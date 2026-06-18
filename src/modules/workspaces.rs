//! Workspace pills and the active window title, kept in sync with niri
//! over its event stream.

use gtk4::prelude::*;
use serde::Deserialize;

use crate::hypr;

#[derive(Deserialize)]
struct Workspace {
    id: u64,
    idx: usize,
    #[serde(default)]
    name: Option<String>,
    is_focused: bool,
}

#[derive(Deserialize)]
struct Window {
    workspace_id: Option<u64>,
}

#[derive(Deserialize)]
struct FocusedWindow {
    title: String,
}

pub fn build() -> gtk4::Box {
    let root = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(10)
        .css_classes(vec!["module"])
        .build();

    let pills = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .css_classes(vec!["workspaces"])
        .build();

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
    line.contains("WorkspacesChanged")
        || line.contains("WorkspaceActivated")
        || line.contains("WorkspaceActiveWindowChanged")
        || line.contains("WindowFocusChanged")
        || line.contains("WindowOpenedOrChanged")
        || line.contains("WindowClosed")
}

fn refresh(pills: &gtk4::Box, title: &gtk4::Label) {
    let workspaces: Vec<Workspace> = hypr::query(&["workspaces"]).unwrap_or_default();
    let windows: Vec<Window> = hypr::query(&["windows"]).unwrap_or_default();

    while let Some(child) = pills.first_child() {
        pills.remove(&child);
    }

    // Sort by idx so pills always appear in order regardless of niri's reply order.
    let mut sorted = workspaces;
    sorted.sort_by_key(|w| w.idx);

    for ws in &sorted {
        let occupied = windows.iter().any(|w| w.workspace_id == Some(ws.id));
        let label_text = ws.name.clone().unwrap_or_else(|| ws.idx.to_string());

        let button = gtk4::Button::builder().css_classes(vec!["workspace-pill"]).build();
        button.set_child(Some(&gtk4::Label::new(Some(&label_text))));

        if ws.is_focused {
            button.add_css_class("active");
        } else if occupied {
            button.add_css_class("occupied");
        }

        let idx = ws.idx;
        button.connect_clicked(move |_| {
            hypr::action(&["focus-workspace", &idx.to_string()]);
        });

        pills.append(&button);
    }

    let focused: Option<FocusedWindow> = hypr::query(&["focused-window"]);
    let text = focused.as_ref().map(|w| w.title.as_str()).unwrap_or("");
    title.set_label(if text.is_empty() { "Desktop" } else { text });
}
