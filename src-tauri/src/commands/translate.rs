use crate::utils::trans_server::{fetch_translation, TransResult};
use crate::AppState;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn fetch_trans_res(
    state: State<'_, AppState>,
    app: AppHandle,
    word: String,
) -> Result<TransResult, ()> {
    match fetch_translation(&word, &state, &app).await {
        Ok(data) => Ok(TransResult::success(data)),
        Err(err_msg) => Ok(TransResult::fail(err_msg)),
    }
}
