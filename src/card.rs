//! Shared "card" popup used by modules like network, bluetooth, volume,
//! media, and battery.
//!
//! These are plain layer-shell windows rather than `GtkPopover` — on this
//! GTK4 / gtk4-layer-shell combination, popping a `GtkPopover` over a
//! layer-shell surface aborts the process with a
//! `cairo_surface_set_device_scale` assertion failure.
//!
//! Each card sits flush against the bottom edge of the bar with square top
//! corners, so it reads as a drawer sliding out of the bar rather than a
//! floating box. Opening/closing fades and slides the card via the
//! `card-enter` CSS class (see `.velo-card-window` / `.card` in style.css).

use std::time::Duration;

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

const ANIM_MS: u64 = 180;

/// Wraps `content` in a hidden card window anchored to the screen's
/// top-right corner, flush against the bottom of the bar.
pub fn build(content: &impl IsA<gtk4::Widget>) -> gtk4::Window {
    let window = gtk4::Window::builder().decorated(false).resizable(false).css_classes(vec!["velo-card-window"]).build();
    window.set_child(Some(content));

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_namespace(Some("velo-shell-card"));
    window.set_keyboard_mode(KeyboardMode::OnDemand);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Right, true);
    // The bar reserves its own exclusive zone, so a 0 top margin sits the
    // card flush against its bottom edge.
    window.set_margin(Edge::Right, 12);

    window.set_visible(false);
    window
}

/// Hides `window` if visible, otherwise runs `on_open` and shows it.
pub fn toggle(window: &gtk4::Window, on_open: impl FnOnce()) {
    if window.is_visible() {
        hide_animated(window);
    } else {
        on_open();
        show_animated(window);
    }
}

/// Maps the window with the `card-enter` (collapsed) state applied, then
/// drops it on the next frame so the card eases into place.
fn show_animated(window: &gtk4::Window) {
    set_entering(window, true);
    window.set_visible(true);
    glib::idle_add_local_once({
        let window = window.clone();
        move || set_entering(&window, false)
    });
}

/// Re-applies the collapsed state to ease the card out, then unmaps it once
/// the transition has finished.
fn hide_animated(window: &gtk4::Window) {
    set_entering(window, true);
    glib::timeout_add_local_once(Duration::from_millis(ANIM_MS), {
        let window = window.clone();
        move || {
            window.set_visible(false);
            set_entering(&window, false);
        }
    });
}

fn set_entering(window: &gtk4::Window, entering: bool) {
    if entering {
        window.add_css_class("card-enter");
    } else {
        window.remove_css_class("card-enter");
    }
    if let Some(content) = window.child() {
        if entering {
            content.add_css_class("card-enter");
        } else {
            content.remove_css_class("card-enter");
        }
    }
}
