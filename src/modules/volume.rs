//! Volume module: shows the default sink's level, scroll to adjust,
//! left-click to mute, right-click for a slider popover.

use std::process::Command;

use gtk4::prelude::*;

const STEP: i32 = 5;

pub fn build() -> gtk4::Box {
    let icon = gtk4::Image::builder().css_classes(vec!["module-icon"]).build();
    let label = gtk4::Label::builder().css_classes(vec!["module-value"]).build();

    let root = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(6).css_classes(vec!["module", "volume-module"]).build();
    root.append(&icon);
    root.append(&label);

    let scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
    scale.set_draw_value(false);
    scale.set_size_request(140, -1);
    scale.add_css_class("volume-scale");

    let popover_box = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).spacing(8).css_classes(vec!["volume-popover-box"]).build();
    popover_box.append(&gtk4::Label::builder().label("VOLUME").css_classes(vec!["panel-title"]).halign(gtk4::Align::Start).build());
    popover_box.append(&scale);

    let popover = gtk4::Popover::builder().css_classes(vec!["velo-popover"]).autohide(true).position(gtk4::PositionType::Bottom).build();
    popover.set_child(Some(&popover_box));
    popover.set_parent(&root);

    scale.connect_value_changed(|scale| set_volume(scale.value().round() as i32));

    let left_click = gtk4::GestureClick::builder().button(1).build();
    left_click.connect_pressed({
        let icon = icon.clone();
        let label = label.clone();
        move |_, _, _, _| {
            toggle_mute();
            refresh(&icon, &label);
        }
    });
    root.add_controller(left_click);

    let right_click = gtk4::GestureClick::builder().button(3).build();
    right_click.connect_pressed({
        let scale = scale.clone();
        move |_, _, _, _| {
            scale.set_value(get_volume() as f64);
            popover.popup();
        }
    });
    root.add_controller(right_click);

    let scroll = gtk4::EventControllerScroll::new(gtk4::EventControllerScrollFlags::VERTICAL);
    scroll.connect_scroll({
        let icon = icon.clone();
        let label = label.clone();
        move |_, _, dy| {
            adjust_volume(if dy < 0.0 { STEP } else { -STEP });
            refresh(&icon, &label);
            glib::Propagation::Stop
        }
    });
    root.add_controller(scroll);

    refresh(&icon, &label);
    glib::timeout_add_seconds_local(2, {
        let icon = icon.clone();
        let label = label.clone();
        move || {
            refresh(&icon, &label);
            glib::ControlFlow::Continue
        }
    });

    root
}

fn refresh(icon: &gtk4::Image, label: &gtk4::Label) {
    let volume = get_volume();
    let muted = is_muted();

    icon.set_icon_name(Some(icon_name(volume, muted)));
    label.set_label(if muted { "Muted".to_string() } else { format!("{volume}%") }.as_str());
}

fn icon_name(volume: u32, muted: bool) -> &'static str {
    if muted || volume == 0 {
        "audio-volume-muted-symbolic"
    } else if volume < 33 {
        "audio-volume-low-symbolic"
    } else if volume < 66 {
        "audio-volume-medium-symbolic"
    } else {
        "audio-volume-high-symbolic"
    }
}

fn get_volume() -> u32 {
    let output = pactl(&["get-sink-volume", "@DEFAULT_SINK@"]);
    output
        .split_whitespace()
        .find_map(|tok| tok.strip_suffix('%'))
        .and_then(|pct| pct.parse().ok())
        .unwrap_or(0)
}

fn is_muted() -> bool {
    pactl(&["get-sink-mute", "@DEFAULT_SINK@"]).contains("yes")
}

fn set_volume(percent: i32) {
    let percent = percent.clamp(0, 100);
    let _ = Command::new("pactl").args(["set-sink-volume", "@DEFAULT_SINK@", &format!("{percent}%")]).status();
}

fn adjust_volume(delta: i32) {
    let current = get_volume() as i32;
    set_volume(current + delta);
}

fn toggle_mute() {
    let _ = Command::new("pactl").args(["set-sink-mute", "@DEFAULT_SINK@", "toggle"]).status();
}

fn pactl(args: &[&str]) -> String {
    Command::new("pactl").args(args).output().ok().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).into_owned()).unwrap_or_default()
}
