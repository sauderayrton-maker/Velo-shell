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
use serde::Deserialize;

use crate::hypr;

const ANIM_MS: u64 = 180;

/// Gap between the bar pill and the card, so it reads as a floating drawer.
const GAP: i32 = 6;
/// Fallback output width if niri can't be reached (logical px).
const FALLBACK_WIDTH: i32 = 1536;

#[derive(Deserialize)]
struct Logical {
    width: i32,
}

#[derive(Deserialize)]
struct Output {
    logical: Logical,
}

/// The focused output's logical width — the same coordinate space
/// layer-shell margins live in. niri is authoritative here; GDK's
/// `monitor_at_surface` is unreliable for layer surfaces and ignores scale,
/// which is what threw cards off on fractional-scale displays.
fn output_width() -> i32 {
    hypr::query::<Output>(&["focused-output"]).map(|o| o.logical.width).filter(|w| *w > 0).unwrap_or(FALLBACK_WIDTH)
}

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
    window.set_anchor(Edge::Left, true);

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

/// Drops the card straight down from `trigger`, horizontally centred on it and
/// clamped to the screen, so it reads as belonging to the icon you clicked.
///
/// The trigger's absolute on-screen rectangle is derived from the pill window's
/// *own* anchors and margins (which we set in `window.rs`) plus the trigger's
/// offset inside that window. Screen width comes from niri, so the math holds at
/// any display scale.
fn position_under(card: &gtk4::Window, trigger: &gtk4::Widget) {
    let Some(native) = trigger.native() else { return };
    let Some(win) = native.dynamic_cast_ref::<gtk4::Window>() else { return };
    let Some(pt) = trigger.compute_point(&native, &graphene::Point::zero()) else { return };

    let width = output_width();
    let win_w = win.width();

    // Left edge of the pill window in output-logical coordinates, from how it
    // was anchored. (All bar windows anchor Top, so the top edge is its margin.)
    let win_left = if win.is_anchor(Edge::Left) {
        win.margin(Edge::Left)
    } else if win.is_anchor(Edge::Right) {
        width - win.margin(Edge::Right) - win_w
    } else {
        (width - win_w) / 2 // centered: niri centers surfaces with no L/R anchor
    };
    let win_top = win.margin(Edge::Top);

    let trigger_center = win_left + pt.x() as i32 + trigger.width() / 2;
    let trigger_bottom = win_top + pt.y() as i32 + trigger.height();

    // Card width — measured natural size, so we can centre it under the trigger.
    let card_w = card.child().map(|c| c.measure(gtk4::Orientation::Horizontal, -1).1).filter(|w| *w > 0).unwrap_or(248);

    let left_margin = (trigger_center - card_w / 2).clamp(GAP, (width - card_w - GAP).max(GAP));
    let top_margin = (trigger_bottom + GAP).max(0);

    // Card is anchored Top+Left; clamp so it always stays on screen.
    card.set_margin(Edge::Left, left_margin);
    card.set_margin(Edge::Top, top_margin);
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
