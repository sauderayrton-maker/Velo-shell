//! Desktops — niri's dynamic workspaces rendered as a morphing instrument
//! row, plus the focused window's title.
//!
//! Unlike the old Hyprland-era approach (re-shelling `niri msg` and rebuilding
//! every pill on each event), this keeps an in-memory model fed straight from
//! niri's event stream. niri emits the full state on connect and deltas after,
//! so the hot path spawns no subprocesses and updates land instantly. Pills are
//! diffed in place so their CSS transitions animate as desktops appear, fill,
//! and focus.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use gtk4::prelude::*;
use serde::Deserialize;
use serde_json::Value;

use crate::hypr;

#[derive(Deserialize, Clone)]
struct WorkspaceInfo {
    id: u64,
    idx: usize,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    output: Option<String>,
    #[serde(default)]
    is_active: bool,
    #[serde(default)]
    is_focused: bool,
}

#[derive(Deserialize, Clone)]
struct WindowInfo {
    id: u64,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    workspace_id: Option<u64>,
    #[serde(default)]
    is_focused: bool,
}

/// The slice of niri state the desktop row cares about.
#[derive(Default)]
struct State {
    workspaces: Vec<WorkspaceInfo>,
    windows: HashMap<u64, WindowInfo>,
    focused: Option<u64>,
}

/// One reusable pill widget kept across renders so CSS transitions animate.
struct Pill {
    button: gtk4::Button,
    label: gtk4::Label,
    target: Rc<Cell<usize>>,
}

pub fn build() -> gtk4::Box {
    let root = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(10)
        .css_classes(vec!["module"])
        .build();

    let pills = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(0)
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

    // Seed from a one-shot query so the bar paints immediately, then let the
    // event stream own every subsequent update.
    let state = Rc::new(RefCell::new(seed_state()));
    let cache: Rc<RefCell<Vec<Pill>>> = Rc::new(RefCell::new(Vec::new()));
    render(&pills, &title, &state.borrow(), &cache);

    let rx = hypr::subscribe();
    glib::spawn_future_local(async move {
        while let Ok(line) = rx.recv().await {
            if apply(&mut state.borrow_mut(), &line) {
                render(&pills, &title, &state.borrow(), &cache);
            }
        }
    });

    root
}

/// Pulls the current workspaces and windows once, for a flash-free first paint.
fn seed_state() -> State {
    let windows: Vec<WindowInfo> = hypr::query(&["windows"]).unwrap_or_default();
    State {
        workspaces: hypr::query(&["workspaces"]).unwrap_or_default(),
        focused: windows.iter().find(|w| w.is_focused).map(|w| w.id),
        windows: windows.into_iter().map(|w| (w.id, w)).collect(),
    }
}

/// Folds one event-stream line into `state`. Returns whether the desktop row
/// needs re-rendering. Parsed defensively via `Value` so unknown niri events
/// (and future fields) are simply ignored.
fn apply(state: &mut State, line: &str) -> bool {
    let Ok(value) = serde_json::from_str::<Value>(line) else { return false };
    let Some((event, payload)) = value.as_object().and_then(|o| o.iter().next()) else { return false };

    match event.as_str() {
        "WorkspacesChanged" => {
            if let Ok(ws) = serde_json::from_value::<Vec<WorkspaceInfo>>(payload["workspaces"].clone()) {
                state.workspaces = ws;
                return true;
            }
        }
        "WorkspaceActivated" => {
            if let Some(id) = payload["id"].as_u64() {
                let focused = payload["focused"].as_bool().unwrap_or(true);
                let output = state.workspaces.iter().find(|w| w.id == id).and_then(|w| w.output.clone());
                for w in &mut state.workspaces {
                    // Only one workspace is active per output…
                    if w.output == output {
                        w.is_active = w.id == id;
                    }
                    // …but focus is global.
                    if focused {
                        w.is_focused = w.id == id;
                    }
                }
                return true;
            }
        }
        "WindowsChanged" => {
            if let Ok(wins) = serde_json::from_value::<Vec<WindowInfo>>(payload["windows"].clone()) {
                state.focused = wins.iter().find(|w| w.is_focused).map(|w| w.id);
                state.windows = wins.into_iter().map(|w| (w.id, w)).collect();
                return true;
            }
        }
        "WindowOpenedOrChanged" => {
            if let Ok(window) = serde_json::from_value::<WindowInfo>(payload["window"].clone()) {
                if window.is_focused {
                    state.focused = Some(window.id);
                }
                state.windows.insert(window.id, window);
                return true;
            }
        }
        "WindowClosed" => {
            if let Some(id) = payload["id"].as_u64() {
                state.windows.remove(&id);
                if state.focused == Some(id) {
                    state.focused = None;
                }
                return true;
            }
        }
        "WindowFocusChanged" => {
            state.focused = payload["id"].as_u64();
            return true;
        }
        _ => {}
    }
    false
}

fn render(container: &gtk4::Box, title: &gtk4::Label, state: &State, cache: &Rc<RefCell<Vec<Pill>>>) {
    let mut sorted: Vec<&WorkspaceInfo> = state.workspaces.iter().collect();
    sorted.sort_by_key(|w| w.idx);

    let mut pills = cache.borrow_mut();

    // Reconcile pill count, reusing widgets so transitions survive across renders.
    while pills.len() < sorted.len() {
        pills.push(new_pill(container));
    }
    while pills.len() > sorted.len() {
        if let Some(pill) = pills.pop() {
            container.remove(&pill.button);
        }
    }

    for (pill, ws) in pills.iter().zip(&sorted) {
        let occupied = state.windows.values().any(|w| w.workspace_id == Some(ws.id));
        let active = ws.is_active;

        // Active and occupied desktops carry their label; empties collapse to a dot.
        let text = if active || occupied { ws.name.clone().unwrap_or_else(|| ws.idx.to_string()) } else { String::new() };
        if pill.label.text() != text {
            pill.label.set_text(&text);
        }
        pill.target.set(ws.idx);

        set_class(&pill.button, "active", active);
        set_class(&pill.button, "occupied", occupied && !active);
        set_class(&pill.button, "empty", !occupied && !active);
        set_class(&pill.button, "focused", ws.is_focused);
    }

    let text = state
        .focused
        .and_then(|id| state.windows.get(&id))
        .and_then(|w| w.title.clone())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "Desktop".to_string());
    if title.text() != text {
        title.set_text(&text);
    }
}

fn new_pill(container: &gtk4::Box) -> Pill {
    let label = gtk4::Label::new(None);
    let button = gtk4::Button::builder().css_classes(vec!["workspace-pill"]).build();
    button.set_child(Some(&label));

    let target = Rc::new(Cell::new(0usize));
    button.connect_clicked({
        let target = target.clone();
        move |_| hypr::action(&["focus-workspace", &target.get().to_string()])
    });

    container.append(&button);
    Pill { button, label, target }
}

fn set_class(widget: &impl IsA<gtk4::Widget>, class: &str, on: bool) {
    if on {
        if !widget.has_css_class(class) {
            widget.add_css_class(class);
        }
    } else if widget.has_css_class(class) {
        widget.remove_css_class(class);
    }
}
