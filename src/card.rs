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

use gtk4::graphene;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

const ANIM_MS: u64 = 180;

/// Wraps `content` in a hidden card window anchored to the top-right area of
/// the screen. The horizontal position is updated at open time by `toggle`.
pub fn build(content: &impl IsA<gtk4::Widget>) -> gtk4::Window {
    let window = gtk4::Window::builder().decorated(false).resizable(false).css_classes(vec!["velo-card-window"]).build();
    window.set_child(Some(content));

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_namespace(Some("velo-shell-card"));
    window.set_keyboard_mode(KeyboardMode::OnDemand);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Right, true);

    window.set_visible(false);
    window
}

/// Hides `window` if visible, otherwise positions it under `trigger`, runs
/// `on_open`, and shows it.
pub fn toggle(window: &gtk4::Window, trigger: &gtk4::Widget, on_open: impl FnOnce()) {
    if window.is_visible() {
        hide_animated(window);
    } else {
        position_under(window, trigger);
        on_open();
        show_animated(window);
    }
}

/// Computes the right margin so the card's right edge aligns with the
/// trigger widget's right edge, making the card appear directly below it.
fn position_under(card: &gtk4::Window, trigger: &gtk4::Widget) {
    let Some(native) = trigger.native() else { return };
    // Bar window spans full width anchored left+right+top, so native coords
    // equal screen coords directly.
    let Some(pt) = trigger.compute_point(&native, &graphene::Point::zero()) else { return };
    let trigger_right = (pt.x() as i32 + trigger.width()).max(0);

    let Some(surface) = native.surface() else { return };
    let screen_width = surface
        .display()
        .monitor_at_surface(&surface)
        .map(|m| m.geometry().width())
        .unwrap_or(1920);

    card.set_margin(Edge::Right, (screen_width - trigger_right).max(0));
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
