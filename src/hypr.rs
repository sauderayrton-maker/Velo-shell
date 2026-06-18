//! niri IPC: JSON queries via `niri msg`, action dispatches, and a live
//! event stream. Replaces the old hyprctl shim — same public surface,
//! compositor backend swapped to niri.

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::Duration;

use serde::de::DeserializeOwned;

/// Run `niri msg -j <args>` and decode the JSON response.
pub fn query<T: DeserializeOwned>(args: &[&str]) -> Option<T> {
    let output = Command::new("niri").arg("msg").arg("-j").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    serde_json::from_slice(&output.stdout).ok()
}

/// Fire `niri msg action <args>` without waiting for it to finish.
pub fn action(args: &[&str]) {
    let _ = Command::new("niri").arg("msg").arg("action").args(args).spawn();
}

/// Stream niri events as raw JSON strings (one object per line).
/// Reconnects automatically if niri restarts or the stream drops.
pub fn subscribe() -> async_channel::Receiver<String> {
    let (tx, rx) = async_channel::unbounded();

    std::thread::spawn(move || loop {
        match Command::new("niri").arg("msg").arg("event-stream").stdout(Stdio::piped()).spawn() {
            Ok(mut child) => {
                let stdout = child.stdout.take().unwrap();
                for line in BufReader::new(stdout).lines() {
                    match line {
                        Ok(line) => {
                            if tx.send_blocking(line).is_err() {
                                return;
                            }
                        }
                        Err(_) => break,
                    }
                }
                let _ = child.wait();
            }
            Err(_) => {}
        }
        std::thread::sleep(Duration::from_secs(1));
    });

    rx
}
