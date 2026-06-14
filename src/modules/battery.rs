//! Battery module: reads `/sys/class/power_supply/BAT*`. Hides itself
//! entirely on desktops with no battery.

use std::fs;
use std::path::{Path, PathBuf};

use gtk4::prelude::*;

pub fn build() -> gtk4::Box {
    let icon = gtk4::Image::builder().css_classes(vec!["module-icon"]).build();
    let label = gtk4::Label::builder().css_classes(vec!["module-value"]).build();

    let root = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(6).css_classes(vec!["module", "battery-module"]).build();
    root.append(&icon);
    root.append(&label);

    match find_battery() {
        Some(path) => {
            update(&path, &icon, &label, &root);
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
