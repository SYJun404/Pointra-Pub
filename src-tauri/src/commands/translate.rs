use super::cache::{with_cache, CacheState};
use crate::utils::trans_server::{fetch_translation, TransResult, TranslationSource};
use crate::utils::word_cache::cache_key;
use crate::AppState;
use std::collections::HashMap;
use std::sync::{atomic::Ordering, Mutex};
use tauri::{AppHandle, Manager, State};

#[derive(Default)]
pub struct TranslationTasks {
    tasks: Mutex<(u64, HashMap<u64, tokio::task::AbortHandle>)>,
}

impl TranslationTasks {
    fn spawn<F>(&self, future: F) -> (u64, tokio::task::JoinHandle<F::Output>)
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        // Register under the same lock used by cancellation, with no await gap.
        let mut tasks = self.tasks.lock().unwrap();
        tasks.0 += 1;
        let id = tasks.0;
        let task = tokio::spawn(future);
        tasks.1.insert(id, task.abort_handle());
        (id, task)
    }

    fn finish(&self, id: u64) {
        self.tasks.lock().unwrap().1.remove(&id);
    }

    pub fn abort_all(&self) {
        for (_, task) in self.tasks.lock().unwrap().1.drain() {
            task.abort();
        }
    }
}

#[tauri::command]
pub async fn fetch_trans_res(
    app: AppHandle,
    tasks: State<'_, TranslationTasks>,
    word: String,
) -> Result<TransResult, ()> {
    let worker_app = app.clone();
    let (id, task) = tasks.spawn(async move {
        let state = worker_app.state::<AppState>();
        let cache = worker_app.state::<CacheState>();
        translate(&state, &worker_app, &cache, word).await
    });
    let result = task.await;
    tasks.finish(id);
    // Aborting drops the HTTP future, including cookie refresh/retries.
    result.map_err(|_| ())?
}

async fn translate(
    state: &AppState,
    app: &AppHandle,
    cache: &CacheState,
    word: String,
) -> Result<TransResult, ()> {
    let word = word.trim().to_owned();
    let generation = cache.generation.load(Ordering::Relaxed);
    let key = cache_key(&word);
    let lookup_key = key.clone();
    match with_cache(cache.cache.clone(), move |c| c.get(&lookup_key)).await {
        Ok(Some((data, source))) => {
            return Ok(TransResult::success(data.with_query_text(&word), source));
        }
        Ok(None) => {}
        Err(error) => eprintln!("[WordCache] lookup: {error}"),
    }
    let started = std::time::Instant::now();
    // Cookie refresh and retries share one budget; a stale idle connection must not
    // leave the cursor card loading for an unbounded amount of time.
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(15),
        fetch_translation(&word, state, app),
    )
    .await;
    if std::env::var_os("POINTRA_TRACE_CURSOR").is_some() {
        eprintln!("[Translation] total={:?}", started.elapsed());
    }
    match result {
        Ok(Ok(data)) => {
            if data.cacheable() {
                let saved = data.clone();
                let generation_counter = cache.generation.clone();
                let cache = cache.cache.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(error) = with_cache(cache, move |c| {
                        // An older request must not repopulate a cleared/imported cache.
                        if generation_counter.load(Ordering::Relaxed) == generation {
                            c.put(&key, &saved)?;
                        }
                        Ok(())
                    })
                    .await
                    {
                        eprintln!("[WordCache] save: {error}");
                    }
                });
            }
            Ok(TransResult::success(data, TranslationSource::Network))
        }
        Ok(Err(err_msg)) => Ok(TransResult::fail(err_msg)),
        Err(_) => Ok(TransResult::fail("翻译请求超时，请重试".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hide_aborts_pending_request_and_next_request_can_finish() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let tasks = TranslationTasks::default();
            let (started_tx, started_rx) = tokio::sync::oneshot::channel();
            let (dropped_tx, dropped_rx) = tokio::sync::oneshot::channel();
            struct OnDrop(Option<tokio::sync::oneshot::Sender<()>>);
            impl Drop for OnDrop {
                fn drop(&mut self) {
                    let _ = self.0.take().unwrap().send(());
                }
            }
            let (old_id, old_task) = tasks.spawn(async move {
                let _guard = OnDrop(Some(dropped_tx));
                started_tx.send(()).unwrap();
                std::future::pending::<()>().await;
            });
            started_rx.await.unwrap();
            tasks.abort_all();
            let (new_id, new_task) = tasks.spawn(async { 42 });
            assert!(old_task.await.unwrap_err().is_cancelled());
            dropped_rx.await.unwrap();
            // Finishing the aborted command must not remove the new request.
            tasks.finish(old_id);
            assert!(tasks.tasks.lock().unwrap().1.contains_key(&new_id));
            assert_eq!(new_task.await.unwrap(), 42);
            tasks.finish(new_id);
            assert!(tasks.tasks.lock().unwrap().1.is_empty());
        });
    }
}
