//! Volume module: shows the default sink's level. Scroll to adjust, click
//! to open a card with a slider, mute toggle, and output device list.

use std::cell::{Cell, RefCell};
use std::process::Command;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::card;

const STEP: i32 = 5;

pub fn build() -> gtk4::Box {
    let icon = gtk4::Image::builder().css_classes(vec!["module-icon"]).build();
    let label = gtk4::Label::builder().css_classes(vec!["module-value"]).build();

    let root = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(6).css_classes(vec!["module", "volume-module"]).build();
    root.append(&icon);
    root.append(&label);

    // ── Card ──
    let title = gtk4::Label::builder().label("SOUND").css_classes(vec!["panel-title"]).halign(gtk4::Align::Start).hexpand(true).build();
    let header = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(8).build();
    header.append(&title);

    let scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
    scale.set_draw_value(false);
    scale.set_hexpand(true);
    scale.add_css_class("volume-scale");

    let mute_icon = gtk4::Image::new();
    let mute_button = gtk4::Button::builder().css_classes(vec!["flat", "card-icon-button"]).build();
    mute_button.set_child(Some(&mute_icon));

    let slider_row = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(10).build();
    slider_row.append(&scale);
    slider_row.append(&mute_button);

    let outputs_title = gtk4::Label::builder().label("OUTPUT").css_classes(vec!["panel-title", "card-section-title"]).halign(gtk4::Align::Start).build();

    let list = gtk4::ListBox::builder().css_classes(vec!["card-list"]).selection_mode(gtk4::SelectionMode::None).build();
    let scroller = gtk4::ScrolledWindow::builder()
        .css_classes(vec!["card-scroll"])
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .max_content_height(160)
        .propagate_natural_height(true)
        .build();
    scroller.set_child(Some(&list));

    let card = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).spacing(8).css_classes(vec!["card", "volume-card"]).build();
    card.append(&header);
    card.append(&slider_row);
    card.append(&outputs_title);
    card.append(&scroller);

    let card_window = card::build(&card);

    let sinks: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let updating = Rc::new(Cell::new(false));

    scale.connect_value_changed({
        let updating = updating.clone();
        move |scale| {
            if updating.get() {
                return;
            }
            set_volume(scale.value().round() as i32);
        }
    });

    mute_button.connect_clicked({
        let icon = icon.clone();
        let label = label.clone();
        let mute_icon = mute_icon.clone();
        move |_| {
            toggle_mute();
            refresh(&icon, &label);
            update_mute_icon(&mute_icon);
        }
    });

    list.connect_row_activated({
        let sinks = sinks.clone();
        let icon = icon.clone();
        let label = label.clone();
        let list = list.clone();
        move |_, row| {
            if let Some(name) = sinks.borrow().get(row.index() as usize).cloned() {
                switch_sink(&name);
                refresh(&icon, &label);
                refresh_sinks(&list, &sinks);
            }
        }
    });

    let click = gtk4::GestureClick::builder().button(1).build();
    click.connect_pressed({
        let scale = scale.clone();
        let mute_icon = mute_icon.clone();
        let updating = updating.clone();
        let list = list.clone();
        let sinks = sinks.clone();
        move |_, _, _, _| {
            card::toggle(&card_window, || {
                updating.set(true);
                scale.set_value(get_volume() as f64);
                updating.set(false);
                update_mute_icon(&mute_icon);
                refresh_sinks(&list, &sinks);
            });
        }
    });
    root.add_controller(click);

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
    update_mute_icon(&mute_icon);
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

fn update_mute_icon(icon: &gtk4::Image) {
    icon.set_icon_name(Some(if is_muted() { "audio-volume-muted-symbolic" } else { "audio-volume-high-symbolic" }));
}

/// Repopulates the output device list, highlighting the default sink.
fn refresh_sinks(list: &gtk4::ListBox, sinks: &Rc<RefCell<Vec<String>>>) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    let default = pactl(&["get-default-sink"]).trim().to_string();
    let all = list_sinks();

    for (name, description) in &all {
        list.append(&build_sink_row(description, *name == default));
    }
    *sinks.borrow_mut() = all.into_iter().map(|(name, _)| name).collect();
}

fn build_sink_row(description: &str, active: bool) -> gtk4::ListBoxRow {
    let icon = gtk4::Image::from_icon_name("audio-speakers-symbolic");
    icon.add_css_class("card-list-icon");

    let label = gtk4::Label::builder().label(description).css_classes(vec!["card-list-title"]).halign(gtk4::Align::Start).hexpand(true).ellipsize(gtk4::pango::EllipsizeMode::End).build();

    let content = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(10).css_classes(vec!["card-list-content"]).build();
    content.append(&icon);
    content.append(&label);

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

/// Lists `(sink_name, description)` pairs from `pactl list sinks`.
fn list_sinks() -> Vec<(String, String)> {
    let mut sinks = Vec::new();
    let mut current_name: Option<String> = None;

    for line in pactl(&["list", "sinks"]).lines() {
        let line = line.trim();
        if let Some(name) = line.strip_prefix("Name: ") {
            current_name = Some(name.to_string());
        } else if let Some(description) = line.strip_prefix("Description: ") {
            if let Some(name) = current_name.take() {
                sinks.push((name, description.to_string()));
            }
        }
    }

    sinks
}

/// Makes `name` the default sink and moves any active streams onto it.
fn switch_sink(name: &str) {
    let _ = Command::new("pactl").args(["set-default-sink", name]).spawn();
    for line in pactl(&["list", "short", "sink-inputs"]).lines() {
        if let Some(id) = line.split_whitespace().next() {
            let _ = Command::new("pactl").args(["move-sink-input", id, name]).spawn();
        }
    }
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
