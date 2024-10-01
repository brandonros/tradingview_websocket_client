use std::{sync::Arc, time::Duration};

use async_io::Timer;
use async_lock::RwLock;

pub async fn run_with_timeout<F, T>(timeout: Duration, future: F) -> Option<T>
where
    F: futures_lite::future::Future<Output = T> + Unpin,
{
    futures_lite::future::or(async { Some(future.await) }, async {
        Timer::after(timeout).await;
        None
    })
    .await
}

pub async fn wait_for_message<F, T>(
    buffer: Arc<RwLock<Vec<T>>>,
    condition: F,
) -> Option<T>
where
    F: Fn(&T) -> bool,
{
    loop {
        let read_lock = buffer.read().await;
        if let Some(index) = read_lock.iter().position(|e| condition(e)) {
            // Drop the read lock before acquiring a write lock
            drop(read_lock);

            // Acquire write lock
            let mut write_lock = buffer.write().await;

            // Remove the item from the buffer
            return Some(write_lock.remove(index));
        } else {
            drop(read_lock);
            Timer::after(Duration::from_millis(1)).await;
        }
    }
}
