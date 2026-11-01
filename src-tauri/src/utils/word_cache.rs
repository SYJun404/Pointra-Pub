//! Local, portable dictionary cache. All access runs on blocking worker threads.
use super::trans_server::{DictResponse, TranslationSource};
use lru::LruCache;
use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use serde::Serialize;
use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const MEMORY_LIMIT: usize = 8 * 1024 * 1024;
pub const DISK_LIMIT: u64 = 100 * 1024 * 1024;
// Leave room for SQLite pages, indices and fragmentation.
const PAYLOAD_LIMIT: i64 = 70 * 1024 * 1024;
const ENTRY_LIMIT: usize = 256 * 1024;
const APPLICATION_ID: i64 = 0x504F494E;
// SQLite 表结构版本：3 = 小写单词作为唯一 key，数据库约束禁止写入大写键。
const SCHEMA_VERSION: i64 = 3;
// 结果版本（不是应用版本号）：1 = 搜狗英译中词卡，保存完整 DictResponse JSON。
// 后续更换来源或调整结果结构时分配新数字，读取、导入时仅接受支持的版本。
const RESULT_VERSION: i64 = 1;

pub struct WordCache {
    db: Connection,
    path: PathBuf,
    memory: LruCache<String, String>,
    memory_bytes: usize,
}

#[derive(Serialize)]
pub struct CacheStats {
    entries: i64,
    disk_bytes: u64,
    memory_bytes: usize,
    memory_limit: usize,
    disk_limit: u64,
}

pub fn cache_key(word: &str) -> String {
    // 大小写不参与缓存身份；保留词形、标点和短语内部空格。
    word.trim().to_ascii_lowercase()
}

fn valid_key(key: &str) -> bool {
    !key.is_empty() && key.is_ascii() && key.trim() == key
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

impl WordCache {
    pub fn open(path: PathBuf) -> anyhow::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Connection::open(&path)?;
        db.busy_timeout(std::time::Duration::from_secs(3))?;
        db.execute_batch(
            "PRAGMA page_size=4096; PRAGMA journal_mode=DELETE;
            PRAGMA synchronous=NORMAL; PRAGMA cache_size=-2048; PRAGMA trusted_schema=OFF;",
        )?;
        let version: i64 = db.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        anyhow::ensure!(
            version == 0 || version == SCHEMA_VERSION,
            "不支持的缓存表结构，请使用新版缓存文件"
        );
        db.execute_batch("CREATE TABLE IF NOT EXISTS words (
            key TEXT PRIMARY KEY CHECK(key = lower(key) COLLATE BINARY),
            -- 1 = 搜狗英译中词卡完整 JSON；非应用版本号。
            version INTEGER NOT NULL CHECK(version > 0), payload TEXT NOT NULL,
            updated_at INTEGER NOT NULL, accessed_at INTEGER NOT NULL,
            bytes INTEGER NOT NULL) WITHOUT ROWID;
            CREATE INDEX IF NOT EXISTS words_access ON words(accessed_at);
            CREATE TABLE IF NOT EXISTS usage (id INTEGER PRIMARY KEY CHECK(id=1), bytes INTEGER NOT NULL);
            INSERT OR IGNORE INTO usage SELECT 1, coalesce(sum(bytes),0) FROM words;
            CREATE TRIGGER IF NOT EXISTS words_insert AFTER INSERT ON words BEGIN
                UPDATE usage SET bytes=bytes+new.bytes WHERE id=1; END;
            CREATE TRIGGER IF NOT EXISTS words_delete AFTER DELETE ON words BEGIN
                UPDATE usage SET bytes=bytes-old.bytes WHERE id=1; END;
            CREATE TRIGGER IF NOT EXISTS words_update AFTER UPDATE OF bytes ON words BEGIN
                UPDATE usage SET bytes=bytes-old.bytes+new.bytes WHERE id=1; END;")?;
        db.pragma_update(None, "application_id", APPLICATION_ID)?;
        db.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        let page_size: i64 = db.query_row("PRAGMA page_size", [], |r| r.get(0))?;
        db.pragma_update(None, "max_page_count", DISK_LIMIT as i64 / page_size)?;
        Ok(Self {
            db,
            path,
            memory: LruCache::unbounded(),
            memory_bytes: 0,
        })
    }

    // Account for owned string capacities plus a conservative allowance for each LRU node.
    fn cost(key: &String, value: &String) -> usize {
        key.capacity() + value.capacity() + 256
    }

    fn remember(&mut self, key: String, value: String) {
        if let Some((k, v)) = self.memory.pop_entry(&key) {
            self.memory_bytes -= Self::cost(&k, &v);
        }
        let cost = Self::cost(&key, &value);
        if cost > MEMORY_LIMIT {
            return;
        }
        while self.memory_bytes + cost > MEMORY_LIMIT {
            if let Some((k, v)) = self.memory.pop_lru() {
                self.memory_bytes -= Self::cost(&k, &v);
            }
        }
        self.memory_bytes += cost;
        self.memory.put(key, value);
    }

    pub fn get(&mut self, key: &str) -> anyhow::Result<Option<(DictResponse, TranslationSource)>> {
        let key = cache_key(key);
        if !valid_key(&key) {
            return Ok(None);
        }
        if let Some(value) = self.memory.get(&key) {
            return Ok(Some((
                serde_json::from_str(value)?,
                TranslationSource::Memory,
            )));
        }
        let payload: Option<String> = self
            .db
            .query_row(
                "SELECT payload FROM words WHERE key=?1 AND version=?2",
                params![key, RESULT_VERSION],
                |r| r.get(0),
            )
            .optional()?;
        let Some(payload) = payload else {
            return Ok(None);
        };
        let data: DictResponse = serde_json::from_str(&payload)?;
        if !data.cacheable() {
            return Ok(None);
        }
        // Disk recency is updated on promotion, rather than every memory hit.
        self.db.execute(
            "UPDATE words SET accessed_at=?2 WHERE key=?1",
            params![key, now()],
        )?;
        self.remember(key.to_owned(), payload);
        Ok(Some((data, TranslationSource::Disk)))
    }

    fn prune(db: &Connection) -> anyhow::Result<()> {
        loop {
            let bytes: i64 =
                db.query_row("SELECT bytes FROM usage WHERE id=1", [], |r| r.get(0))?;
            let (pages, free, page_size): (i64, i64, i64) = db.query_row(
                "SELECT page_count, freelist_count, page_size FROM pragma_page_count(), pragma_freelist_count(), pragma_page_size()",
                [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
            if bytes <= PAYLOAD_LIMIT && (pages - free) * page_size < DISK_LIMIT as i64 * 9 / 10 {
                break;
            }
            let removed = db.execute("DELETE FROM words WHERE key IN (SELECT key FROM words ORDER BY accessed_at LIMIT 128)", [])?;
            if removed == 0 {
                break;
            }
        }
        Ok(())
    }

    pub fn put(&mut self, key: &str, data: &DictResponse) -> anyhow::Result<()> {
        let key = cache_key(key);
        if !data.cacheable() || !valid_key(&key) {
            return Ok(());
        }
        let payload = serde_json::to_string(data)?;
        if payload.len() + key.len() > ENTRY_LIMIT {
            return Ok(());
        }
        let tx = self.db.transaction()?;
        // Prune before insertion too, retaining headroom below the physical page limit.
        Self::prune(&tx)?;
        tx.execute(
            "INSERT INTO words (key,version,payload,updated_at,accessed_at,bytes) VALUES (?1,?5,?2,?3,?3,?4)
            ON CONFLICT(key) DO UPDATE SET version=excluded.version, payload=excluded.payload, updated_at=excluded.updated_at,
            accessed_at=excluded.accessed_at, bytes=excluded.bytes",
            params![key, payload, now(), (key.len() + payload.len() + 8) as i64, RESULT_VERSION],
        )?;
        Self::prune(&tx)?;
        tx.commit()?;
        self.remember(key.to_owned(), payload);
        Ok(())
    }

    pub fn stats(&self) -> anyhow::Result<CacheStats> {
        Ok(CacheStats {
            entries: self
                .db
                .query_row("SELECT count(*) FROM words", [], |r| r.get(0))?,
            disk_bytes: std::fs::metadata(&self.path)?.len(),
            memory_bytes: self.memory_bytes,
            memory_limit: MEMORY_LIMIT,
            disk_limit: DISK_LIMIT,
        })
    }

    pub fn clear(&mut self) -> anyhow::Result<()> {
        self.db.execute("DELETE FROM words", [])?;
        self.memory.clear();
        self.memory_bytes = 0;
        self.db.execute_batch("VACUUM")?;
        Ok(())
    }

    pub fn export(&self, destination: &Path) -> anyhow::Result<()> {
        anyhow::ensure!(
            destination != self.path
                && destination.canonicalize().ok() != self.path.canonicalize().ok(),
            "不能覆盖正在使用的缓存"
        );
        // Write beside the destination, then atomically replace it only on success.
        let temp = destination.with_file_name(format!(".pointra-{}.sqlite", uuid::Uuid::new_v4()));
        let result = (|| -> anyhow::Result<()> {
            self.db
                .execute("VACUUM INTO ?1", [temp.to_string_lossy().as_ref()])?;
            std::fs::rename(&temp, destination)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(temp);
        }
        result
    }

    pub fn import(&mut self, source: &Path) -> anyhow::Result<usize> {
        anyhow::ensure!(
            std::fs::metadata(source)?.len() <= DISK_LIMIT,
            "导入文件超过 100 MB"
        );
        anyhow::ensure!(
            source.canonicalize()? != self.path.canonicalize()?,
            "不能导入正在使用的缓存，请选择导出文件"
        );
        let input = Connection::open_with_flags(source, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        input.execute_batch("PRAGMA trusted_schema=OFF; PRAGMA query_only=ON;")?;
        let id: i64 = input.query_row("PRAGMA application_id", [], |r| r.get(0))?;
        let version: i64 = input.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        anyhow::ensure!(
            id == APPLICATION_ID && version == SCHEMA_VERSION,
            "不是兼容的 Pointra 缓存文件"
        );
        let check: String = input.query_row("PRAGMA quick_check", [], |r| r.get(0))?;
        anyhow::ensure!(check == "ok", "缓存文件损坏");
        let kind: String = input.query_row(
            "SELECT type FROM sqlite_schema WHERE name='words'",
            [],
            |r| r.get(0),
        )?;
        anyhow::ensure!(kind == "table", "无效的缓存数据表");
        let mut statement = input.prepare("SELECT key,payload,updated_at,version FROM words")?;
        let mut rows = statement.query([])?;
        let tx = self.db.transaction()?;
        let mut imported = 0;
        while let Some(row) = rows.next()? {
            anyhow::ensure!(
                row.get_ref(0)?.as_str()?.len() + row.get_ref(1)?.as_str()?.len() <= ENTRY_LIMIT,
                "缓存条目过大"
            );
            let raw_key: String = row.get(0)?;
            let key = cache_key(&raw_key);
            let payload: String = row.get(1)?;
            let updated: i64 = row.get(2)?;
            anyhow::ensure!(key.len() + payload.len() <= ENTRY_LIMIT, "缓存条目过大");
            let result_version: i64 = row.get(3)?;
            anyhow::ensure!(result_version == RESULT_VERSION, "不支持的查询结果版本");
            let data: DictResponse = serde_json::from_str(&payload)?;
            anyhow::ensure!(
                data.cacheable()
                    && valid_key(&key)
                    && updated >= 0
                    && updated <= now() + 86_400_000,
                "无效的缓存条目"
            );
            imported += tx.execute("INSERT INTO words (key,version,payload,updated_at,accessed_at,bytes) VALUES (?1,?6,?2,?3,?4,?5)
                ON CONFLICT(key) DO UPDATE SET version=excluded.version, payload=excluded.payload, updated_at=excluded.updated_at,
                accessed_at=excluded.accessed_at, bytes=excluded.bytes
                WHERE excluded.updated_at > words.updated_at",
                params![key, payload, updated, now(), (key.len() + payload.len() + 8) as i64, result_version])?;
            Self::prune(&tx)?;
        }
        tx.commit()?;
        self.memory.clear();
        self.memory_bytes = 0;
        Ok(imported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data(show: serde_json::Value) -> DictResponse {
        serde_json::from_value(
            serde_json::json!({"voice":{},"translate":{"errorCode":"0"},"wordCard":{"show":show}}),
        )
        .unwrap()
    }
    #[test]
    fn persistence_filter_and_portable_merge() {
        let root = std::env::temp_dir().join(format!("pointra-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("a.sqlite");
        let key = cache_key("Domain");
        {
            let mut a = WordCache::open(path.clone()).unwrap();
            a.put(&key, &data(serde_json::json!(true))).unwrap();
            for show in [
                serde_json::json!(false),
                serde_json::json!("true"),
                serde_json::Value::Null,
            ] {
                a.put(&cache_key("excluded"), &data(show)).unwrap();
            }
            assert_eq!(a.stats().unwrap().entries, 1);
            a.export(&root.join("export.sqlite")).unwrap();
        }
        let mut a = WordCache::open(path).unwrap();
        assert!(a.get(&key).unwrap().is_some());
        let mut b = WordCache::open(root.join("b.sqlite")).unwrap();
        assert_eq!(b.import(&root.join("export.sqlite")).unwrap(), 1);
        assert_eq!(b.import(&root.join("export.sqlite")).unwrap(), 0);
        assert!(b.get(&key).unwrap().is_some());
        b.clear().unwrap();
        assert!(b.get(&key).unwrap().is_none());
        drop(a);
        drop(b);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn memory_is_bounded_and_keys_ignore_case() {
        assert_eq!(cache_key("Domain"), cache_key("domain"));
        assert_eq!(cache_key(" domain "), cache_key("domain"));
        let root = std::env::temp_dir().join(format!("pointra-test-{}", uuid::Uuid::new_v4()));
        let mut cache = WordCache::open(root.join("test.sqlite")).unwrap();
        for i in 0..100 {
            cache.remember(i.to_string(), "x".repeat(200_000));
        }
        assert!(cache.memory_bytes <= MEMORY_LIMIT);
        assert!(!cache.memory.contains("0"));
        assert!(cache.memory.contains("99"));
        drop(cache);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn disk_budget_evicts_old_entries() {
        let root = std::env::temp_dir().join(format!("pointra-test-{}", uuid::Uuid::new_v4()));
        let mut cache = WordCache::open(root.join("test.sqlite")).unwrap();
        let large: DictResponse = serde_json::from_value(serde_json::json!({
            "voice":{}, "translate":{"dit":"x".repeat(240_000)}, "wordCard":{"show":true}
        }))
        .unwrap();
        cache.put(&cache_key("oldest"), &large).unwrap();
        cache
            .db
            .execute("UPDATE words SET accessed_at=0", [])
            .unwrap();
        for i in 0..460 {
            cache.put(&cache_key(&format!("word{i}")), &large).unwrap();
        }
        assert!(cache.stats().unwrap().disk_bytes <= DISK_LIMIT);
        assert!(cache.stats().unwrap().entries < 461);
        let total: i64 = cache
            .db
            .query_row("SELECT bytes FROM usage", [], |r| r.get(0))
            .unwrap();
        assert!(total <= PAYLOAD_LIMIT);
        // Memory is independently bounded; clear it to verify disk eviction.
        cache.memory.clear();
        cache.memory_bytes = 0;
        assert!(cache.get(&cache_key("oldest")).unwrap().is_none());
        assert!(cache.get(&cache_key("word459")).unwrap().is_some());
        drop(cache);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn import_rolls_back_invalid_rows_and_keeps_newer_results() {
        let root = std::env::temp_dir().join(format!("pointra-test-{}", uuid::Uuid::new_v4()));
        let mut source = WordCache::open(root.join("source.sqlite")).unwrap();
        source
            .put(&cache_key("alpha"), &data(serde_json::json!(true)))
            .unwrap();
        source
            .put(&cache_key("zulu"), &data(serde_json::json!(true)))
            .unwrap();
        // Invalid payload is encountered after the valid row.
        source
            .db
            .execute(
                "UPDATE words SET payload='{}' WHERE key=?1",
                [cache_key("zulu")],
            )
            .unwrap();
        source.export(&root.join("invalid.sqlite")).unwrap();
        let mut target = WordCache::open(root.join("target.sqlite")).unwrap();
        target
            .put(&cache_key("keep"), &data(serde_json::json!(true)))
            .unwrap();
        assert!(target.import(&root.join("invalid.sqlite")).is_err());
        assert_eq!(target.stats().unwrap().entries, 1);
        assert!(target.get(&cache_key("alpha")).unwrap().is_none());
        source
            .db
            .execute("DELETE FROM words WHERE key=?1", [cache_key("zulu")])
            .unwrap();
        source
            .db
            .execute("UPDATE words SET updated_at=1", [])
            .unwrap();
        source.export(&root.join("valid.sqlite")).unwrap();
        target
            .put(&cache_key("alpha"), &data(serde_json::json!(true)))
            .unwrap();
        assert_eq!(target.import(&root.join("valid.sqlite")).unwrap(), 0);
        source
            .db
            .execute("UPDATE words SET updated_at=?1", [now() + 1000])
            .unwrap();
        source.export(&root.join("valid.sqlite")).unwrap();
        assert_eq!(target.import(&root.join("valid.sqlite")).unwrap(), 1);
        assert!(target.export(&root.join("target.sqlite")).is_err());
        assert!(target.import(&root.join("target.sqlite")).is_err());
        drop(source);
        drop(target);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn plain_word_keys_and_numeric_result_version() {
        let root = std::env::temp_dir().join(format!("pointra-test-{}", uuid::Uuid::new_v4()));
        let mut source = WordCache::open(root.join("source.sqlite")).unwrap();
        let data = data(serde_json::json!(true));
        assert_eq!(cache_key(" Domain "), "domain");
        source.put("domain name", &data).unwrap();
        source.put("领域", &data).unwrap();
        assert_eq!(source.stats().unwrap().entries, 1);
        assert!(source.get("领域").unwrap().is_none());
        let (key, version, kind): (String, i64, String) = source
            .db
            .query_row("SELECT key,version,typeof(version) FROM words", [], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })
            .unwrap();
        assert_eq!(key, "domain name");
        assert_eq!(version, RESULT_VERSION);
        assert_eq!(kind, "integer");
        source
            .db
            .execute("UPDATE words SET version=999", [])
            .unwrap();
        source.export(&root.join("unsupported.sqlite")).unwrap();
        let mut target = WordCache::open(root.join("target.sqlite")).unwrap();
        assert!(target.import(&root.join("unsupported.sqlite")).is_err());
        assert_eq!(target.stats().unwrap().entries, 0);
        source.memory.clear();
        source.memory_bytes = 0;
        assert!(source.get("domain name").unwrap().is_none());
        drop(source);
        drop(target);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn response_source_matches_each_read_path() {
        use super::super::trans_server::TransResult;
        let root = std::env::temp_dir().join(format!("pointra-test-{}", uuid::Uuid::new_v4()));
        let path = root.join("test.sqlite");
        let mut cache = WordCache::open(path.clone()).unwrap();
        cache.put("domain", &data(serde_json::json!(true))).unwrap();
        let (_, source) = cache.get("domain").unwrap().unwrap();
        assert_eq!(source, TranslationSource::Memory);
        drop(cache);
        let mut cache = WordCache::open(path).unwrap();
        let (_, source) = cache.get("domain").unwrap().unwrap();
        assert_eq!(source, TranslationSource::Disk);
        let (_, source) = cache.get("domain").unwrap().unwrap();
        assert_eq!(source, TranslationSource::Memory);
        for (source, expected) in [
            (TranslationSource::Memory, "memory"),
            (TranslationSource::Disk, "disk"),
            (TranslationSource::Network, "network"),
        ] {
            let response = TransResult::success(data(serde_json::json!(true)), source);
            let json = serde_json::to_value(response).unwrap();
            assert_eq!(json["source"], expected);
            assert_eq!(json["status"], 200);
            assert!(json["data"].get("source").is_none());
        }
        let failure = serde_json::to_value(TransResult::fail("timeout".into())).unwrap();
        assert_eq!(failure["source"], "network");
        drop(cache);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn case_variants_share_memory_disk_and_imported_record() {
        let root = std::env::temp_dir().join(format!("pointra-test-{}", uuid::Uuid::new_v4()));
        let path = root.join("source.sqlite");
        let mut source = WordCache::open(path.clone()).unwrap();
        let response: DictResponse = serde_json::from_value(serde_json::json!({
            "voice":{}, "translate":{"text":"Domain", "orig_text":"Domain", "dit":"领域"},
            "wordCard":{"show":true,"title":"domain"}
        }))
        .unwrap();
        source.put("Domain", &response).unwrap();
        source.put("domain", &response).unwrap();
        source.put("DOMAIN", &response).unwrap();
        assert_eq!(source.stats().unwrap().entries, 1);
        assert_eq!(source.memory.len(), 1);
        assert_eq!(
            source.get("Domain").unwrap().unwrap().1,
            TranslationSource::Memory
        );
        let stored: String = source
            .db
            .query_row("SELECT key FROM words", [], |r| r.get(0))
            .unwrap();
        assert_eq!(stored, "domain");
        assert!(source
            .db
            .execute("UPDATE words SET key='Domain'", [])
            .is_err());
        source.export(&root.join("export.sqlite")).unwrap();
        drop(source);
        let mut reopened = WordCache::open(path).unwrap();
        let (data, origin) = reopened.get("DOMAIN").unwrap().unwrap();
        assert_eq!(origin, TranslationSource::Disk);
        let displayed = serde_json::to_value(data.with_query_text("DOMAIN")).unwrap();
        assert_eq!(displayed["translate"]["text"], "DOMAIN");
        assert_eq!(displayed["translate"]["orig_text"], "DOMAIN");
        assert_eq!(displayed["wordCard"]["title"], "domain");
        let mut target = WordCache::open(root.join("target.sqlite")).unwrap();
        target.put("DOMAIN", &response).unwrap();
        assert_eq!(target.import(&root.join("export.sqlite")).unwrap(), 0);
        assert_eq!(target.stats().unwrap().entries, 1);
        assert!(target.get("domain").unwrap().is_some());
        assert_ne!(cache_key("domain"), cache_key("domains"));
        assert_eq!(cache_key(" Domain Name "), "domain name");
        drop(reopened);
        drop(target);
        std::fs::remove_dir_all(root).unwrap();
    }
}
