use std::time::Duration;

use async_io::Timer;

use crate::futures_provider;

pub async fn run_with_timeout<F, T>(timeout: Duration, future: F) -> Option<T>
where
    F: futures_provider::future::Future<Output = T> + Unpin,
{
    futures_provider::future::or(async { Some(future.await) }, async {
        Timer::after(timeout).await;
        None
    })
    .await
}

pub async fn wait_for_message<F, T>(
    frame_rx: &async_channel::Receiver<T>,
    buffer: &mut Vec<T>,
    condition: F,
) -> Option<T>
where
    F: Fn(&T) -> bool,
{
    loop {
        match frame_rx.recv().await {
            Ok(frame) => {
                if condition(&frame) {
                    return Some(frame);
                } else {
                    buffer.push(frame);
                }
            }
            Err(_) => return None, // Channel closed
        }
    }
}
