use crate::utils::word_cache::{CacheStats, WordCache};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use tauri::State;

pub struct CacheState {
    pub cache: Arc<Mutex<Option<WordCache>>>,
    pub generation: Arc<AtomicU64>,
}

pub async fn with_cache<T: Send + 'static>(
    cache: Arc<Mutex<Option<WordCache>>>,
    action: impl FnOnce(&mut WordCache) -> anyhow::Result<T> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut guard = cache.lock().map_err(|_| "缓存锁不可用".to_string())?;
        let cache = guard
            .as_mut()
            .ok_or("缓存不可用，请检查应用数据目录后重启")?;
        action(cache).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn cache_stats(state: State<'_, CacheState>) -> Result<CacheStats, String> {
    with_cache(state.cache.clone(), |c| c.stats()).await
}

#[tauri::command]
pub async fn clear_word_cache(state: State<'_, CacheState>) -> Result<(), String> {
    let generation = state.generation.clone();
    with_cache(state.cache.clone(), move |c| {
        c.clear()?;
        generation.fetch_add(1, Ordering::Relaxed);
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn export_word_cache(state: State<'_, CacheState>) -> Result<bool, String> {
    let Some(file) = rfd::AsyncFileDialog::new()
        .set_title("导出单词缓存")
        .add_filter("SQLite", &["sqlite"])
        .set_file_name("Pointra-cache.sqlite")
        .save_file()
        .await
    else {
        return Ok(false);
    };
    let path = file.path().to_owned();
    with_cache(state.cache.clone(), move |c| c.export(&path)).await?;
    Ok(true)
}

#[tauri::command]
pub async fn import_word_cache(state: State<'_, CacheState>) -> Result<Option<usize>, String> {
    let Some(file) = rfd::AsyncFileDialog::new()
        .set_title("导入单词缓存")
        .add_filter("SQLite", &["sqlite", "db"])
        .pick_file()
        .await
    else {
        return Ok(None);
    };
    let path = file.path().to_owned();
    let generation = state.generation.clone();
    with_cache(state.cache.clone(), move |c| {
        let count = c.import(&path)?;
        generation.fetch_add(1, Ordering::Relaxed);
        Ok(count)
    })
    .await
    .map(Some)
}
