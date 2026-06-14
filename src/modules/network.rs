//! Network module: Wi-Fi SSID/signal, wired, or offline — read via `nmcli`.

use std::process::Command;

use gtk4::prelude::*;

pub fn build() -> gtk4::Box {
    let icon = gtk4::Image::builder().css_classes(vec!["module-icon"]).build();
    let label = gtk4::Label::builder().css_classes(vec!["module-value"]).ellipsize(gtk4::pango::EllipsizeMode::End).max_width_chars(16).build();

    let root = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(6).css_classes(vec!["module", "network-module"]).build();
    root.append(&icon);
    root.append(&label);

    update(&icon, &label);
    glib::timeout_add_seconds_local(5, move || {
        update(&icon, &label);
        glib::ControlFlow::Continue
    });

    root
}

enum Status {
    Wifi { ssid: String, signal: u8 },
    Wired,
    Offline,
}

fn status() -> Status {
    let devices = nmcli(&["-t", "-f", "TYPE,STATE", "device", "status"]);

    let mut wifi_connected = false;
    let mut wired_connected = false;
    for line in devices.lines() {
        let mut parts = line.split(':');
        let (Some(kind), Some(state)) = (parts.next(), parts.next()) else { continue };
        if state != "connected" {
            continue;
        }
        match kind {
            "wifi" => wifi_connected = true,
            "ethernet" => wired_connected = true,
            _ => {}
        }
    }

    if wifi_connected {
        let wifi = nmcli(&["-t", "-f", "ACTIVE,SIGNAL,SSID", "device", "wifi"]);
        for line in wifi.lines() {
            let mut parts = line.split(':');
            let (Some(active), Some(signal), Some(ssid)) = (parts.next(), parts.next(), parts.next()) else { continue };
            if active == "yes" {
                return Status::Wifi { ssid: unescape(ssid), signal: signal.parse().unwrap_or(0) };
            }
        }
        return Status::Wifi { ssid: String::new(), signal: 0 };
    }

    if wired_connected {
        return Status::Wired;
    }

    Status::Offline
}

/// `nmcli -t` escapes literal `:` in field values as `\:`.
fn unescape(field: &str) -> String {
    field.replace("\\:", ":")
}

fn nmcli(args: &[&str]) -> String {
    Command::new("nmcli").args(args).output().ok().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).into_owned()).unwrap_or_default()
}

fn update(icon: &gtk4::Image, label: &gtk4::Label) {
    match status() {
        Status::Wifi { ssid, signal } => {
            icon.set_icon_name(Some(signal_icon(signal)));
            label.set_label(if ssid.is_empty() { "Wi-Fi" } else { &ssid });
        }
        Status::Wired => {
            icon.set_icon_name(Some("network-wired-symbolic"));
            label.set_label("Wired");
        }
        Status::Offline => {
            icon.set_icon_name(Some("network-wireless-offline-symbolic"));
            label.set_label("Offline");
        }
    }
}

fn signal_icon(signal: u8) -> &'static str {
    match signal {
        80..=100 => "network-wireless-signal-excellent-symbolic",
        60..=79 => "network-wireless-signal-good-symbolic",
        40..=59 => "network-wireless-signal-ok-symbolic",
        20..=39 => "network-wireless-signal-weak-symbolic",
        _ => "network-wireless-signal-none-symbolic",
    }
}
