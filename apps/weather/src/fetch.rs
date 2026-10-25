//! The network thread. The UI never blocks on HTTP: this thread fetches, sends
//! the result down a channel, pokes an eventfd so the display backend wakes
//! up, and the UI drains the channel on its next frame.

use std::os::fd::RawFd;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use crate::api;
use crate::config::Config;
use crate::model::Forecast;

const REFRESH: Duration = Duration::from_secs(15 * 60);
const BACKOFF_MIN: Duration = Duration::from_secs(5);
const BACKOFF_MAX: Duration = Duration::from_secs(5 * 60);

pub struct Fetcher {
    pub rx: Receiver<Result<Forecast, String>>,
    pub wake: RawFd,
}

/// The wait after `failures` failures in a row: 5 s, 10 s, 20 s ... 5 min.
fn backoff(failures: u32) -> Duration {
    BACKOFF_MIN.saturating_mul(1 << failures.saturating_sub(1).min(6)).min(BACKOFF_MAX)
}

pub fn spawn(config: Config) -> Fetcher {
    // SAFETY: plain eventfd(2); the fd lives for the whole process.
    let wake = unsafe { libc::eventfd(0, libc::EFD_CLOEXEC | libc::EFD_NONBLOCK) };
    let (tx, rx) = channel();
    std::thread::spawn(move || run(config, tx, wake));
    Fetcher { rx, wake }
}

fn run(config: Config, tx: Sender<Result<Forecast, String>>, wake: RawFd) {
    let mut failures = 0u32;
    loop {
        let result = api::fetch(&config).map_err(|e| format!("{e:#}"));
        failures = if result.is_ok() { 0 } else { failures + 1 };
        let wait = if failures == 0 { REFRESH } else { backoff(failures) };
        if tx.send(result).is_err() {
            return;
        }
        if wake >= 0 {
            let one = 1u64.to_ne_bytes();
            // SAFETY: writes 8 bytes from a live buffer to our own eventfd.
            unsafe { libc::write(wake, one.as_ptr().cast(), one.len()) };
        }
        std::thread::sleep(wait);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_doubles_then_caps() {
        let secs: Vec<u64> = (1..=8).map(|n| backoff(n).as_secs()).collect();
        assert_eq!(secs, [5, 10, 20, 40, 80, 160, 300, 300]);
    }
}
