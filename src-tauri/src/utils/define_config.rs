use device_query::Keycode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use tauri::{AppHandle, Runtime};
use tauri_plugin_global_shortcut::{Code, Modifiers};
use tauri_plugin_store::StoreExt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub theme: String,
    pub auto_start: bool,
    pub auto_play: bool,

    pub pronunciation: String,
    pub pronunciation_volume: u32,

    pub point_key: String,
    pub pinned_key: String,
    pub hide_win_key: String,

    pub select_text_modifiers: Modifiers,
    pub select_text_code: Code,

    pub search_win_modifiers: Modifiers,
    pub search_win_code: Code,

    pub first_setup_completed: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "light".to_string(),
            auto_start: false,
            auto_play: false,
            pronunciation: "us".to_string(),
            pronunciation_volume: 50,
            point_key: "F2".to_string(),
            pinned_key: "F1".to_string(),
            hide_win_key: "Tab".to_string(),
            select_text_modifiers: if cfg!(target_os = "windows") {
                Modifiers::ALT
            } else {
                Modifiers::SUPER
            },
            select_text_code: Code::Digit1,
            search_win_modifiers: if cfg!(target_os = "windows") {
                Modifiers::ALT
            } else {
                Modifiers::SUPER
            },
            search_win_code: Code::Digit2,
            first_setup_completed: false,
        }
    }
}

// 定义一个常数作为 store 的文件名
const CONFIG_FILE: &str = "config.json";

pub fn keycode_from_str(s: &str) -> Option<Keycode> {
    Some(match s {
        "F1" => Keycode::F1,
        "F2" => Keycode::F2,
        "F3" => Keycode::F3,
        "F4" => Keycode::F4,
        "F5" => Keycode::F5,
        "F6" => Keycode::F6,
        "F7" => Keycode::F7,
        "F8" => Keycode::F8,
        "F9" => Keycode::F9,
        "F10" => Keycode::F10,
        "F11" => Keycode::F11,
        "F12" => Keycode::F12,
        "Tab" => Keycode::Tab,
        "Escape" | "Esc" => Keycode::Escape,
        _ => return None,
    })
}

/// 递归地将 `default` 中缺失的键合并到 `stored` 中。
/// 返回 `true` 表示发生了合并（即有新字段被加入）。
fn merge_missing_fields(stored: &mut Value, default: &Value) -> bool {
    match (stored, default) {
        (Value::Object(stored_map), Value::Object(default_map)) => {
            let mut changed = false;
            // 由于不能同时借用 stored_map 的可变和不可变引用，先收集需要插入的键值对
            let mut to_insert: BTreeMap<String, Value> = BTreeMap::new();
            for (key, default_val) in default_map {
                if !stored_map.contains_key(key) {
                    to_insert.insert(key.clone(), default_val.clone());
                }
            }
            if !to_insert.is_empty() {
                changed = true;
                for (key, val) in to_insert {
                    stored_map.insert(key, val);
                }
            }
            changed
        }
        _ => false,
    }
}

/// 将存储中的旧配置与当前 `AppConfig` 的默认值做合并，
/// 补全新增的配置字段（保留已有值），如果发生了合并则自动重新保存。
pub fn get_config_from_store<R: Runtime>(app: &AppHandle<R>) -> Result<AppConfig, String> {
    let store = app
        .store(CONFIG_FILE)
        .map_err(|e| format!("Failed to open store: {}", e))?;

    // 当前 AppConfig 默认值的 JSON 结构（代表完整的配置 schema）
    let default_value = serde_json::to_value(AppConfig::default())
        .map_err(|e| format!("Serialize default: {}", e))?;

    if let Some(value) = store.get("config") {
        let mut stored_value = value.clone();

        // 合并缺失的字段
        let changed = merge_missing_fields(&mut stored_value, &default_value);

        let config: AppConfig = serde_json::from_value(stored_value.clone())
            .map_err(|e| format!("Failed to parse config JSON: {}", e))?;

        // 如果合并了新的字段，重新保存以持久化
        if changed {
            store.set("config".to_string(), stored_value);
            store
                .save()
                .map_err(|e| format!("Failed to save merged config: {}", e))?;
        }

        Ok(config)
    } else {
        Ok(AppConfig::default())
    }
}

/// 将配置保存回 Store
pub fn save_config_to_store<R: Runtime>(
    app: &AppHandle<R>,
    config: &AppConfig,
) -> Result<(), String> {
    let store = app.store(CONFIG_FILE).map_err(|e| e.to_string())?;

    let value = serde_json::to_value(config).map_err(|e| e.to_string())?;

    store.set("config".to_string(), value);

    store.save().map_err(|e| e.to_string())?;

    Ok(())
}
