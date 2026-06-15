//! Shared "card" popup used by modules like network, bluetooth, volume,
//! media, and battery.
//!
//! These are plain layer-shell windows rather than `GtkPopover` — on this
//! GTK4 / gtk4-layer-shell combination, popping a `GtkPopover` over a
//! layer-shell surface aborts the process with a
//! `cairo_surface_set_device_scale` assertion failure.

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

/// Wraps `content` in a hidden card window anchored to the screen's
/// top-right corner, just below the bar.
pub fn build(content: &impl IsA<gtk4::Widget>) -> gtk4::Window {
    let window = gtk4::Window::builder().decorated(false).resizable(false).css_classes(vec!["velo-popover"]).build();
    window.set_child(Some(content));

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_namespace(Some("velo-shell-card"));
    window.set_keyboard_mode(KeyboardMode::OnDemand);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Right, true);
    window.set_margin(Edge::Top, 48);
    window.set_margin(Edge::Right, 12);

    // Autohide once the card loses keyboard focus.
    window.connect_notify_local(Some("is-active"), |win, _| {
        if win.is_visible() && !win.is_active() {
            win.set_visible(false);
        }
    });

    window.set_visible(false);
    window
}

/// Hides `window` if visible, otherwise runs `on_open` and shows it.
pub fn toggle(window: &gtk4::Window, on_open: impl FnOnce()) {
    if window.is_visible() {
        window.set_visible(false);
    } else {
        on_open();
        window.set_visible(true);
    }
}
