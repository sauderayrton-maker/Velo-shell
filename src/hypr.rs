//! Thin wrapper around Hyprland's IPC: JSON queries via `hyprctl`, dispatches
//! for actions like switching workspaces, and a live event stream read from
//! Hyprland's Unix socket.

use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use serde::de::DeserializeOwned;

/// Runs `hyprctl -j <args>` and decodes the JSON response.
pub fn query<T: DeserializeOwned>(args: &[&str]) -> Option<T> {
    let output = Command::new("hyprctl").arg("-j").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    serde_json::from_slice(&output.stdout).ok()
}

/// Fires a `hyprctl dispatch <args>` command without waiting for it to finish.
pub fn dispatch(args: &[&str]) {
    let _ = Command::new("hyprctl").arg("dispatch").args(args).spawn();
}

fn event_socket_path() -> Option<PathBuf> {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").ok()?;
    let signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
    Some(PathBuf::from(runtime_dir).join("hypr").join(signature).join(".socket2.sock"))
}

/// Connects to Hyprland's event socket and streams one line per event
/// (`EVENT>>DATA`). Reconnects with a short backoff if the socket isn't
/// available yet or the connection drops.
pub fn subscribe() -> async_channel::Receiver<String> {
    let (tx, rx) = async_channel::unbounded();

    std::thread::spawn(move || loop {
        let Some(path) = event_socket_path() else {
            std::thread::sleep(Duration::from_secs(2));
            continue;
        };

        match UnixStream::connect(&path) {
            Ok(stream) => {
                for line in BufReader::new(stream).lines() {
                    match line {
                        Ok(line) => {
                            if tx.send_blocking(line).is_err() {
                                return;
                            }
                        }
                        Err(_) => break,
                    }
                }
            }
            Err(_) => std::thread::sleep(Duration::from_secs(2)),
        }
    });

    rx
}
