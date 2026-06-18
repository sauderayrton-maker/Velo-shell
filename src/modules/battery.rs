//! Battery module: reads `/sys/class/power_supply/BAT*`. Hides itself
//! entirely on desktops with no battery. Click opens a card with a
//! larger icon, status, and an estimated time remaining.

use std::fs;
use std::path::{Path, PathBuf};

use gtk4::prelude::*;

use crate::card;

pub fn build() -> gtk4::Box {
    let icon = gtk4::Image::builder().css_classes(vec!["module-icon"]).build();
    let label = gtk4::Label::builder().css_classes(vec!["module-value"]).build();

    let root = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(6).css_classes(vec!["module", "battery-module"]).build();
    root.append(&icon);
    root.append(&label);

    // ── Card ──
    let big_icon = gtk4::Image::builder().css_classes(vec!["battery-card-icon"]).build();
    big_icon.set_pixel_size(40);

    let percent_label = gtk4::Label::builder().css_classes(vec!["battery-card-percent"]).halign(gtk4::Align::Start).build();
    let status_label = gtk4::Label::builder().css_classes(vec!["battery-card-status"]).halign(gtk4::Align::Start).build();
    let time_label = gtk4::Label::builder().css_classes(vec!["battery-card-time"]).halign(gtk4::Align::Start).build();

    let text_col = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).spacing(2).valign(gtk4::Align::Center).build();
    text_col.append(&percent_label);
    text_col.append(&status_label);

    let top_row = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(14).build();
    top_row.append(&big_icon);
    top_row.append(&text_col);

    let card = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).spacing(8).css_classes(vec!["card", "battery-card"]).build();
    card.append(&gtk4::Label::builder().label("BATTERY").css_classes(vec!["panel-title"]).halign(gtk4::Align::Start).build());
    card.append(&top_row);
    card.append(&time_label);

    let card_window = card::build(&card);

    match find_battery() {
        Some(path) => {
            update(&path, &icon, &label, &root);

            let click = gtk4::GestureClick::builder().button(1).build();
            click.connect_pressed({
                let root = root.clone();
                let path = path.clone();
                move |_, _, _, _| {
                    card::toggle(&card_window, root.upcast_ref(), || {
                        update_card(&path, &big_icon, &percent_label, &status_label, &time_label);
                    });
                }
            });
            root.add_controller(click);

            let root_for_timer = root.clone();
            glib::timeout_add_seconds_local(15, move || {
                update(&path, &icon, &label, &root_for_timer);
                glib::ControlFlow::Continue
            });
        }
        None => root.set_visible(false),
    }

    root
}

fn find_battery() -> Option<PathBuf> {
    let mut batteries: Vec<PathBuf> = fs::read_dir("/sys/class/power_supply")
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("BAT")))
        .collect();
    batteries.sort();
    batteries.into_iter().next()
}

fn update(path: &Path, icon: &gtk4::Image, label: &gtk4::Label, root: &gtk4::Box) {
    let capacity: u32 = read_trim(&path.join("capacity")).and_then(|s| s.parse().ok()).unwrap_or(0);
    let status = read_trim(&path.join("status")).unwrap_or_default();
    let charging = status.eq_ignore_ascii_case("charging");

    icon.set_icon_name(Some(battery_icon(capacity, charging)));
    label.set_label(&format!("{capacity}%"));

    root.remove_css_class("battery-low");
    if capacity <= 15 && !charging {
        root.add_css_class("battery-low");
    }
}

fn update_card(path: &Path, icon: &gtk4::Image, percent: &gtk4::Label, status_label: &gtk4::Label, time_label: &gtk4::Label) {
    let capacity: u32 = read_trim(&path.join("capacity")).and_then(|s| s.parse().ok()).unwrap_or(0);
    let status = read_trim(&path.join("status")).unwrap_or_default();
    let charging = status.eq_ignore_ascii_case("charging");

    icon.set_icon_name(Some(battery_icon(capacity, charging)));
    percent.set_label(&format!("{capacity}%"));
    status_label.set_label(if status.is_empty() { "Unknown" } else { &status });

    match time_remaining(path, charging) {
        Some(text) => {
            time_label.set_label(&text);
            time_label.set_visible(true);
        }
        None => time_label.set_visible(false),
    }
}

/// Estimates time to empty (discharging) or full (charging) from
/// `energy_now` / `energy_full` / `power_now` (µWh / µW).
fn time_remaining(path: &Path, charging: bool) -> Option<String> {
    let energy_now: f64 = read_trim(&path.join("energy_now"))?.parse().ok()?;
    let energy_full: f64 = read_trim(&path.join("energy_full"))?.parse().ok()?;
    let power_now: f64 = read_trim(&path.join("power_now"))?.parse().ok()?;
    if power_now <= 0.0 {
        return None;
    }

    let remaining = if charging { energy_full - energy_now } else { energy_now };
    let hours = remaining / power_now;
    if !hours.is_finite() || hours <= 0.0 {
        return None;
    }

    let total_minutes = (hours * 60.0).round() as i64;
    let (h, m) = (total_minutes / 60, total_minutes % 60);
    let verb = if charging { "Full in" } else { "Remaining" };
    Some(format!("{verb} {h}h {m}m"))
}

fn read_trim(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn battery_icon(capacity: u32, charging: bool) -> &'static str {
    let level = match capacity {
        0..=10 => "empty",
        11..=35 => "low",
        36..=70 => "good",
        _ => "full",
    };

    match (level, charging) {
        ("empty", true) => "battery-empty-charging-symbolic",
        ("low", true) => "battery-low-charging-symbolic",
        ("good", true) => "battery-good-charging-symbolic",
        (_, true) => "battery-full-charging-symbolic",
        ("empty", false) => "battery-empty-symbolic",
        ("low", false) => "battery-low-symbolic",
        ("good", false) => "battery-good-symbolic",
        (_, false) => "battery-full-symbolic",
    }
}
