//! Media module: now-playing track via MPRIS (`playerctl`). Hidden
//! entirely when no player is active. Click opens a card with transport
//! controls; scroll skips tracks.

use std::process::Command;

use gtk4::prelude::*;

use crate::card;

pub fn build() -> gtk4::Box {
    let icon = gtk4::Image::builder().css_classes(vec!["module-icon"]).build();
    let label = gtk4::Label::builder().css_classes(vec!["module-value"]).ellipsize(gtk4::pango::EllipsizeMode::End).max_width_chars(28).build();

    let root = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(6).css_classes(vec!["module", "media-module"]).build();
    root.append(&icon);
    root.append(&label);

    // ── Card ──
    let title_label = gtk4::Label::builder().css_classes(vec!["media-track-title"]).halign(gtk4::Align::Start).ellipsize(gtk4::pango::EllipsizeMode::End).max_width_chars(32).build();
    let artist_label = gtk4::Label::builder().css_classes(vec!["media-track-artist"]).halign(gtk4::Align::Start).ellipsize(gtk4::pango::EllipsizeMode::End).max_width_chars(32).build();

    let (prev_button, _) = icon_button("media-skip-backward-symbolic");
    let (play_button, play_icon) = icon_button("media-playback-start-symbolic");
    let (next_button, _) = icon_button("media-skip-forward-symbolic");

    let controls = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(6).halign(gtk4::Align::Center).css_classes(vec!["media-controls"]).build();
    controls.append(&prev_button);
    controls.append(&play_button);
    controls.append(&next_button);

    let card = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).spacing(4).css_classes(vec!["card", "media-card"]).build();
    card.append(&gtk4::Label::builder().label("MEDIA").css_classes(vec!["panel-title"]).halign(gtk4::Align::Start).build());
    card.append(&title_label);
    card.append(&artist_label);
    card.append(&controls);

    let card_window = card::build(&card);

    prev_button.connect_clicked(|_| playerctl(&["previous"]));
    next_button.connect_clicked(|_| playerctl(&["next"]));
    play_button.connect_clicked(|_| playerctl(&["play-pause"]));

    let click = gtk4::GestureClick::builder().button(1).build();
    click.connect_pressed({
        let title_label = title_label.clone();
        let artist_label = artist_label.clone();
        let play_icon = play_icon.clone();
        move |_, _, _, _| {
            card::toggle(&card_window, || {
                refresh_card(&title_label, &artist_label, &play_icon);
            });
        }
    });
    root.add_controller(click);

    let scroll = gtk4::EventControllerScroll::new(gtk4::EventControllerScrollFlags::VERTICAL);
    scroll.connect_scroll(|_, _, dy| {
        playerctl(if dy < 0.0 { &["next"] } else { &["previous"] });
        glib::Propagation::Stop
    });
    root.add_controller(scroll);

    update(&root, &icon, &label, &play_icon);
    glib::timeout_add_seconds_local(1, {
        let root = root.clone();
        move || {
            update(&root, &icon, &label, &play_icon);
            glib::ControlFlow::Continue
        }
    });

    root
}

fn icon_button(icon_name: &str) -> (gtk4::Button, gtk4::Image) {
    let icon = gtk4::Image::from_icon_name(icon_name);
    let button = gtk4::Button::builder().css_classes(vec!["flat", "media-btn"]).build();
    button.set_child(Some(&icon));
    (button, icon)
}

fn playerctl(args: &[&str]) {
    let _ = Command::new("playerctl").args(args).spawn();
}

fn playerctl_output(args: &[&str]) -> Option<String> {
    let output = Command::new("playerctl").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// `(status, artist, title)` for the active player, if any.
fn now_playing() -> Option<(String, String, String)> {
    let raw = playerctl_output(&["metadata", "--format", "{{status}}|{{artist}}|{{title}}"])?;
    let mut parts = raw.splitn(3, '|');
    let status = parts.next()?.to_string();
    let artist = parts.next().unwrap_or_default().to_string();
    let title = parts.next().unwrap_or_default().to_string();
    Some((status, artist, title))
}

fn set_play_icon(icon: &gtk4::Image, playing: bool) {
    icon.set_icon_name(Some(if playing { "media-playback-pause-symbolic" } else { "media-playback-start-symbolic" }));
}

fn update(root: &gtk4::Box, icon: &gtk4::Image, label: &gtk4::Label, play_icon: &gtk4::Image) {
    let Some((status, artist, title)) = now_playing() else {
        root.set_visible(false);
        return;
    };
    root.set_visible(true);

    icon.set_icon_name(Some("audio-x-generic-symbolic"));
    label.set_label(&if artist.is_empty() { title } else { format!("{artist} — {title}") });

    set_play_icon(play_icon, status == "Playing");
}

fn refresh_card(title_label: &gtk4::Label, artist_label: &gtk4::Label, play_icon: &gtk4::Image) {
    match now_playing() {
        Some((status, artist, title)) => {
            title_label.set_label(if title.is_empty() { "Nothing playing" } else { &title });
            artist_label.set_visible(!artist.is_empty());
            artist_label.set_label(&artist);
            set_play_icon(play_icon, status == "Playing");
        }
        None => {
            title_label.set_label("Nothing playing");
            artist_label.set_visible(false);
        }
    }
}
