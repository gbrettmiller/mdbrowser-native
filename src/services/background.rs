//! Runs blocking work on its own thread, so iced's executor threads never stall
//! on disk, child processes or timers.

use std::future::Future;
use std::time::Duration;

use iced::futures::channel::oneshot;

pub fn run_blocking<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> impl Future<Output = T> {
    let (sender, receiver) = oneshot::channel();
    std::thread::spawn(move || {
        let _ = sender.send(work());
    });
    async move { receiver.await.expect("background thread finished its work") }
}

/// Completes after `duration` without blocking the executor.
pub fn after(duration: Duration) -> impl Future<Output = ()> {
    run_blocking(move || std::thread::sleep(duration))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn returns_the_work_result() {
        assert_eq!(futures::executor::block_on(run_blocking(|| 2 + 2)), 4);
    }

    #[test]
    fn work_runs_off_the_calling_thread() {
        let caller = std::thread::current().id();
        let worker = futures::executor::block_on(run_blocking(|| std::thread::current().id()));
        assert_ne!(worker, caller);
    }

    #[test]
    fn after_waits_at_least_the_duration() {
        let started = Instant::now();
        futures::executor::block_on(after(Duration::from_millis(20)));
        assert!(started.elapsed() >= Duration::from_millis(20));
    }
}
