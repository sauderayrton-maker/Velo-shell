//! Workspace pills and the active window's title, kept in sync with
//! Hyprland over its event socket.

use gtk4::prelude::*;
use serde::Deserialize;

use crate::hypr;

#[derive(Deserialize)]
struct Workspace {
    id: i32,
    windows: i32,
}

#[derive(Deserialize, Default)]
struct ActiveWindow {
    #[serde(default)]
    title: String,
}

/// Always show at least this many workspace pills, even if some are empty.
const MIN_WORKSPACES: i32 = 10;

pub fn build() -> gtk4::Box {
    let root = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(10).css_classes(vec!["module"]).build();

    let pills = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(4).css_classes(vec!["workspaces"]).build();

    let title = gtk4::Label::builder()
        .css_classes(vec!["window-title"])
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .max_width_chars(48)
        .halign(gtk4::Align::Start)
        .build();

    root.append(&pills);
    root.append(&title);

    refresh(&pills, &title);

    let rx = hypr::subscribe();
    let pills = pills.clone();
    let title = title.clone();
    glib::spawn_future_local(async move {
        while let Ok(line) = rx.recv().await {
            if is_relevant(&line) {
                refresh(&pills, &title);
            }
        }
    });

    root
}

fn is_relevant(line: &str) -> bool {
    const EVENTS: &[&str] = &[
        "workspace>>",
        "focusedmon>>",
        "activewindow>>",
        "createworkspace>>",
        "destroyworkspace>>",
        "moveworkspace>>",
        "urgent>>",
    ];
    EVENTS.iter().any(|prefix| line.starts_with(prefix))
}

fn refresh(pills: &gtk4::Box, title: &gtk4::Label) {
    let workspaces: Vec<Workspace> = hypr::query(&["workspaces"]).unwrap_or_default();
    let active_id: i32 = hypr::query::<Workspace>(&["activeworkspace"]).map_or(-1, |w| w.id);

    let max_id = workspaces.iter().map(|w| w.id).filter(|&id| id > 0).max().unwrap_or(0).max(MIN_WORKSPACES);

    while let Some(child) = pills.first_child() {
        pills.remove(&child);
    }

    for id in 1..=max_id {
        let occupied = workspaces.iter().any(|w| w.id == id && w.windows > 0);

        let button = gtk4::Button::builder().css_classes(vec!["workspace-pill"]).build();
        button.set_child(Some(&gtk4::Label::new(Some(&id.to_string()))));
        if id == active_id {
            button.add_css_class("active");
        } else if occupied {
            button.add_css_class("occupied");
        }
        button.connect_clicked(move |_| hypr::dispatch(&["workspace", &id.to_string()]));

        pills.append(&button);
    }

    let active_window: ActiveWindow = hypr::query(&["activewindow"]).unwrap_or_default();
    title.set_label(if active_window.title.is_empty() { "Desktop" } else { &active_window.title });
}
