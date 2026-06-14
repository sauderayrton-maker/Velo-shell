//! Power button: a small menu for locking, logging out, suspending,
//! restarting or shutting down.

use std::process::Command;

use gtk4::prelude::*;

use crate::hypr;

enum Action {
    Lock,
    Logout,
    Suspend,
    Reboot,
    Shutdown,
}

const ACTIONS: &[(&str, &str, bool, Action)] = &[
    ("system-lock-screen-symbolic", "Lock", false, Action::Lock),
    ("weather-clear-night-symbolic", "Suspend", false, Action::Suspend),
    ("system-log-out-symbolic", "Log Out", false, Action::Logout),
    ("system-reboot-symbolic", "Restart", false, Action::Reboot),
    ("system-shutdown-symbolic", "Shut Down", true, Action::Shutdown),
];

pub fn build() -> gtk4::Button {
    let button = gtk4::Button::builder().css_classes(vec!["module", "power-button", "flat"]).build();
    button.set_child(Some(&gtk4::Image::from_icon_name("system-shutdown-symbolic")));

    let list = gtk4::ListBox::builder().css_classes(vec!["power-menu-list"]).selection_mode(gtk4::SelectionMode::None).build();
    for (icon, title, danger, _) in ACTIONS {
        list.append(&build_row(icon, title, *danger));
    }

    let popover = gtk4::Popover::builder().css_classes(vec!["velo-popover"]).autohide(true).position(gtk4::PositionType::Bottom).build();
    popover.set_child(Some(&list));
    popover.set_parent(&button);

    list.connect_row_activated({
        let popover = popover.clone();
        move |_, row| {
            popover.popdown();
            if let Some((_, _, _, action)) = ACTIONS.get(row.index() as usize) {
                run(action);
            }
        }
    });

    button.connect_clicked(move |_| popover.popup());

    button
}

fn build_row(icon_name: &str, title: &str, danger: bool) -> gtk4::ListBoxRow {
    let icon = gtk4::Image::from_icon_name(icon_name);
    icon.set_pixel_size(16);
    icon.add_css_class("power-menu-icon");

    let label = gtk4::Label::builder().label(title).css_classes(vec!["power-menu-title"]).halign(gtk4::Align::Start).build();

    let content = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(12).css_classes(vec!["power-menu-row"]).build();
    content.append(&icon);
    content.append(&label);

    let row = gtk4::ListBoxRow::new();
    row.set_child(Some(&content));
    if danger {
        row.add_css_class("danger");
    }
    row
}

fn run(action: &Action) {
    match action {
        Action::Lock => spawn("loginctl", &["lock-session"]),
        Action::Suspend => spawn("systemctl", &["suspend"]),
        Action::Logout => hypr::dispatch(&["exit"]),
        Action::Reboot => spawn("systemctl", &["reboot"]),
        Action::Shutdown => spawn("systemctl", &["poweroff"]),
    }
}

fn spawn(cmd: &str, args: &[&str]) {
    let _ = Command::new(cmd).args(args).spawn();
}
