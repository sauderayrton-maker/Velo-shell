//! Bluetooth module: power state + paired devices via `bluetoothctl`.
//! Click opens a card with a power toggle and a connect/disconnect list.

use std::cell::{Cell, RefCell};
use std::process::Command;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::card;

pub fn build() -> gtk4::Box {
    let icon = gtk4::Image::builder().css_classes(vec!["module-icon"]).build();
    let label = gtk4::Label::builder().css_classes(vec!["module-value"]).ellipsize(gtk4::pango::EllipsizeMode::End).max_width_chars(14).build();

    let root = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(6).css_classes(vec!["module", "bluetooth-module"]).build();
    root.append(&icon);
    root.append(&label);

    // ── Card ──
    let title = gtk4::Label::builder().label("BLUETOOTH").css_classes(vec!["panel-title"]).halign(gtk4::Align::Start).hexpand(true).build();
    let power_switch = gtk4::Switch::new();

    let header = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(8).build();
    header.append(&title);
    header.append(&power_switch);

    let status_label = gtk4::Label::builder().css_classes(vec!["card-subtitle"]).halign(gtk4::Align::Start).ellipsize(gtk4::pango::EllipsizeMode::End).build();

    let list = gtk4::ListBox::builder().css_classes(vec!["card-list"]).selection_mode(gtk4::SelectionMode::None).build();
    let scroller = gtk4::ScrolledWindow::builder()
        .css_classes(vec!["card-scroll"])
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .max_content_height(220)
        .propagate_natural_height(true)
        .build();
    scroller.set_child(Some(&list));

    let card = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).spacing(8).css_classes(vec!["card", "bluetooth-card"]).build();
    card.append(&header);
    card.append(&status_label);
    card.append(&scroller);

    let card_window = card::build(&card);

    let devices: Rc<RefCell<Vec<(String, String)>>> = Rc::new(RefCell::new(Vec::new())); // (mac, name)
    let updating = Rc::new(Cell::new(false));

    power_switch.connect_active_notify({
        let updating = updating.clone();
        let power_switch_ref = power_switch.clone();
        let status_label = status_label.clone();
        let list = list.clone();
        let devices = devices.clone();
        move |sw| {
            if updating.get() {
                return;
            }
            let on = sw.is_active();
            let _ = Command::new("bluetoothctl").args(["power", if on { "on" } else { "off" }]).spawn();

            let power_switch = power_switch_ref.clone();
            let status_label = status_label.clone();
            let list = list.clone();
            let devices = devices.clone();
            let updating = updating.clone();
            glib::timeout_add_seconds_local(1, move || {
                refresh_card(&power_switch, &status_label, &list, &devices, &updating);
                glib::ControlFlow::Break
            });
        }
    });

    list.connect_row_activated({
        let devices = devices.clone();
        let power_switch = power_switch.clone();
        let status_label = status_label.clone();
        let list = list.clone();
        let updating = updating.clone();
        move |_, row| {
            let Some((mac, _)) = devices.borrow().get(row.index() as usize).cloned() else { return };
            let action = if row.has_css_class("connected") { "disconnect" } else { "connect" };
            let _ = Command::new("bluetoothctl").args([action, &mac]).spawn();

            let power_switch = power_switch.clone();
            let status_label = status_label.clone();
            let list = list.clone();
            let devices = devices.clone();
            let updating = updating.clone();
            glib::timeout_add_seconds_local(2, move || {
                refresh_card(&power_switch, &status_label, &list, &devices, &updating);
                glib::ControlFlow::Break
            });
        }
    });

    let click = gtk4::GestureClick::builder().button(1).build();
    click.connect_pressed({
        let power_switch = power_switch.clone();
        let status_label = status_label.clone();
        let list = list.clone();
        let devices = devices.clone();
        let updating = updating.clone();
        move |_, _, _, _| {
            card::toggle(&card_window, || {
                refresh_card(&power_switch, &status_label, &list, &devices, &updating);
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

fn bluetoothctl(args: &[&str]) -> String {
    Command::new("bluetoothctl").args(args).output().ok().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).into_owned()).unwrap_or_default()
}

fn is_powered() -> bool {
    bluetoothctl(&["show"]).lines().any(|l| l.trim() == "Powered: yes")
}

/// Parses a `Device AA:BB:CC:DD:EE:FF Name` line into `(mac, name)`.
fn parse_device_line(line: &str) -> Option<(String, String)> {
    let rest = line.strip_prefix("Device ")?;
    let (mac, name) = rest.split_once(' ')?;
    Some((mac.to_string(), name.to_string()))
}

fn update(icon: &gtk4::Image, label: &gtk4::Label) {
    if !is_powered() {
        icon.set_icon_name(Some("bluetooth-disabled-symbolic"));
        label.set_label("Off");
        return;
    }

    icon.set_icon_name(Some("bluetooth-active-symbolic"));
    match bluetoothctl(&["devices", "Connected"]).lines().find_map(parse_device_line) {
        Some((_, name)) => label.set_label(&name),
        None => label.set_label("On"),
    }
}

/// Repopulates the Bluetooth card: power state and the paired-device list.
fn refresh_card(power_switch: &gtk4::Switch, status_label: &gtk4::Label, list: &gtk4::ListBox, devices: &Rc<RefCell<Vec<(String, String)>>>, updating: &Rc<Cell<bool>>) {
    let powered = is_powered();

    updating.set(true);
    power_switch.set_active(powered);
    updating.set(false);

    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    if !powered {
        status_label.set_label("Bluetooth is off");
        devices.borrow_mut().clear();
        return;
    }

    let connected: Vec<String> = bluetoothctl(&["devices", "Connected"]).lines().filter_map(parse_device_line).map(|(mac, _)| mac).collect();
    let paired: Vec<(String, String)> = bluetoothctl(&["devices", "Paired"]).lines().filter_map(parse_device_line).collect();

    let connected_names: Vec<&str> = paired.iter().filter(|(mac, _)| connected.contains(mac)).map(|(_, name)| name.as_str()).collect();
    status_label.set_label(&if connected_names.is_empty() { "No devices connected".to_string() } else { format!("Connected: {}", connected_names.join(", ")) });

    for (mac, name) in &paired {
        list.append(&build_device_row(name, connected.contains(mac)));
    }
    *devices.borrow_mut() = paired;
}

fn build_device_row(name: &str, connected: bool) -> gtk4::ListBoxRow {
    let icon = gtk4::Image::from_icon_name(if connected { "bluetooth-active-symbolic" } else { "bluetooth-symbolic" });
    icon.add_css_class("card-list-icon");

    let label = gtk4::Label::builder().label(name).css_classes(vec!["card-list-title"]).halign(gtk4::Align::Start).hexpand(true).ellipsize(gtk4::pango::EllipsizeMode::End).build();

    let content = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(10).css_classes(vec!["card-list-content"]).build();
    content.append(&icon);
    content.append(&label);

    if connected {
        let check = gtk4::Image::from_icon_name("object-select-symbolic");
        check.add_css_class("card-list-check");
        content.append(&check);
    }

    let row = gtk4::ListBoxRow::new();
    row.set_child(Some(&content));
    if connected {
        row.add_css_class("active");
        row.add_css_class("connected");
    }
    row
}
