//! Network module: Wi-Fi SSID/signal, wired, or offline — read via `nmcli`.
//! Click opens a card with a Wi-Fi toggle and nearby networks.

use std::cell::{Cell, RefCell};
use std::process::Command;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::card;

pub fn build() -> gtk4::Box {
    let icon = gtk4::Image::builder().css_classes(vec!["module-icon"]).build();
    let label = gtk4::Label::builder().css_classes(vec!["module-value"]).ellipsize(gtk4::pango::EllipsizeMode::End).max_width_chars(16).build();

    let root = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(6).css_classes(vec!["module", "network-module"]).build();
    root.append(&icon);
    root.append(&label);

    // ── Card ──
    let title = gtk4::Label::builder().label("NETWORK").css_classes(vec!["panel-title"]).halign(gtk4::Align::Start).hexpand(true).build();
    let wifi_switch = gtk4::Switch::new();

    let header = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(8).build();
    header.append(&title);
    header.append(&wifi_switch);

    let status_label = gtk4::Label::builder().css_classes(vec!["card-subtitle"]).halign(gtk4::Align::Start).ellipsize(gtk4::pango::EllipsizeMode::End).build();

    let list = gtk4::ListBox::builder().css_classes(vec!["card-list"]).selection_mode(gtk4::SelectionMode::None).build();
    let scroller = gtk4::ScrolledWindow::builder()
        .css_classes(vec!["card-scroll"])
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .max_content_height(220)
        .propagate_natural_height(true)
        .build();
    scroller.set_child(Some(&list));

    let card = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).spacing(8).css_classes(vec!["card", "network-card"]).build();
    card.append(&header);
    card.append(&status_label);
    card.append(&scroller);

    let card_window = card::build(&card);

    let networks: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let updating = Rc::new(Cell::new(false));

    wifi_switch.connect_active_notify({
        let updating = updating.clone();
        let wifi_switch_ref = wifi_switch.clone();
        let status_label = status_label.clone();
        let list = list.clone();
        let networks = networks.clone();
        move |sw| {
            if updating.get() {
                return;
            }
            let enable = sw.is_active();
            let _ = Command::new("nmcli").args(["radio", "wifi", if enable { "on" } else { "off" }]).spawn();

            let wifi_switch = wifi_switch_ref.clone();
            let status_label = status_label.clone();
            let list = list.clone();
            let networks = networks.clone();
            let updating = updating.clone();
            glib::timeout_add_seconds_local(1, move || {
                refresh_card(&wifi_switch, &status_label, &list, &networks, &updating);
                glib::ControlFlow::Break
            });
        }
    });

    list.connect_row_activated({
        let networks = networks.clone();
        move |_, row| {
            if let Some(ssid) = networks.borrow().get(row.index() as usize).cloned() {
                let _ = Command::new("nmcli").args(["connection", "up", "id", &ssid]).spawn();
            }
        }
    });

    let click = gtk4::GestureClick::builder().button(1).build();
    click.connect_pressed({
        let root = root.clone();
        let wifi_switch = wifi_switch.clone();
        let status_label = status_label.clone();
        let list = list.clone();
        let networks = networks.clone();
        let updating = updating.clone();
        move |_, _, _, _| {
            card::toggle(&card_window, root.upcast_ref(), || {
                refresh_card(&wifi_switch, &status_label, &list, &networks, &updating);
            });
        }
    });
    root.add_controller(click);

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
            let mut parts = line.splitn(3, ':');
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

/// Repopulates the Wi-Fi card: toggle state, current connection, and the
/// scan list of nearby networks.
fn refresh_card(wifi_switch: &gtk4::Switch, status_label: &gtk4::Label, list: &gtk4::ListBox, networks: &Rc<RefCell<Vec<String>>>, updating: &Rc<Cell<bool>>) {
    let enabled = nmcli(&["radio", "wifi"]).trim() == "enabled";

    updating.set(true);
    wifi_switch.set_active(enabled);
    updating.set(false);

    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    if !enabled {
        status_label.set_label("Wi-Fi is off");
        networks.borrow_mut().clear();
        return;
    }

    let scan = nmcli(&["-t", "-f", "ACTIVE,SIGNAL,SECURITY,SSID", "device", "wifi", "list"]);

    // (ssid, signal, secured, active)
    let mut entries: Vec<(String, u8, bool, bool)> = Vec::new();
    for line in scan.lines() {
        let mut parts = line.splitn(4, ':');
        let (Some(active), Some(signal), Some(security), Some(ssid)) = (parts.next(), parts.next(), parts.next(), parts.next()) else { continue };
        let ssid = unescape(ssid);
        if ssid.is_empty() {
            continue;
        }
        let signal: u8 = signal.parse().unwrap_or(0);
        let secured = !security.is_empty();
        let active = active == "yes";

        if let Some(existing) = entries.iter_mut().find(|(s, ..)| *s == ssid) {
            existing.3 |= active;
            if signal > existing.1 {
                existing.1 = signal;
            }
        } else {
            entries.push((ssid, signal, secured, active));
        }
    }
    entries.sort_by(|a, b| b.1.cmp(&a.1));

    let active_ssid = entries.iter().find(|(_, _, _, active)| *active).map(|(ssid, ..)| ssid.clone());
    status_label.set_label(&match &active_ssid {
        Some(ssid) => format!("Connected to {ssid}"),
        None => "Not connected".to_string(),
    });

    *networks.borrow_mut() = entries.iter().map(|(ssid, ..)| ssid.clone()).collect();

    for (ssid, signal, secured, active) in &entries {
        list.append(&build_network_row(ssid, *signal, *secured, *active));
    }
}

fn build_network_row(ssid: &str, signal: u8, secured: bool, active: bool) -> gtk4::ListBoxRow {
    let icon = gtk4::Image::from_icon_name(signal_icon(signal));
    icon.add_css_class("card-list-icon");

    let label = gtk4::Label::builder().label(ssid).css_classes(vec!["card-list-title"]).halign(gtk4::Align::Start).hexpand(true).ellipsize(gtk4::pango::EllipsizeMode::End).build();

    let content = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(10).css_classes(vec!["card-list-content"]).build();
    content.append(&icon);
    content.append(&label);

    if secured {
        let lock = gtk4::Image::from_icon_name("channel-secure-symbolic");
        lock.add_css_class("card-list-icon");
        content.append(&lock);
    }
    if active {
        let check = gtk4::Image::from_icon_name("object-select-symbolic");
        check.add_css_class("card-list-check");
        content.append(&check);
    }

    let row = gtk4::ListBoxRow::new();
    row.set_child(Some(&content));
    if active {
        row.add_css_class("active");
    }
    row
}
