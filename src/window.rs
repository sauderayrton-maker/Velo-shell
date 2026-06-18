use gtk4::gdk;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use crate::modules::{battery, bluetooth, clock, media, network, power, system, volume, workspaces};

pub fn build_windows(app: &gtk4::Application) {
    load_css();
    build_spacer(app).present();
    build_left(app).present();
    build_center(app).present();
    build_right(app).present();
}

/// Invisible full-width window whose only job is to hold the exclusive zone
/// so niri keeps all tiled windows below the pill strip.
/// top_margin(8) + pill_height(38) + gap(8) = 54px reserved from top edge.
fn build_spacer(app: &gtk4::Application) -> gtk4::ApplicationWindow {
    let window = shell_window(app);
    window.set_layer(Layer::Top);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_anchor(Edge::Top, true);
    window.set_exclusive_zone(54);

    let spacer = gtk4::Box::builder()
        .css_classes(vec!["bar-spacer"])
        .build();
    window.set_default_size(-1, 54);
    window.set_child(Some(&spacer));
    window
}

fn shell_window(app: &gtk4::Application) -> gtk4::ApplicationWindow {
    let w = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("Velo Shell")
        .decorated(false)
        .build();
    w.init_layer_shell();
    // Overlay: positions relative to raw screen geometry, not the usable area
    // after exclusive zones, so the spacer's zone doesn't push the pills down.
    w.set_layer(Layer::Overlay);
    w.set_namespace(Some("velo-shell"));
    w
}

fn pill(spacing: i32) -> gtk4::Box {
    gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(spacing)
        .css_classes(vec!["bar-pill"])
        .build()
}

fn build_left(app: &gtk4::Application) -> gtk4::ApplicationWindow {
    let window = shell_window(app);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Top, true);
    window.set_margin(Edge::Top, 8);
    window.set_margin(Edge::Left, 12);

    let p = pill(0);
    p.append(&workspaces::build());
    window.set_child(Some(&p));
    window
}

fn build_center(app: &gtk4::Application) -> gtk4::ApplicationWindow {
    let window = shell_window(app);
    // No horizontal anchors → niri centers the surface.
    window.set_anchor(Edge::Top, true);
    window.set_margin(Edge::Top, 8);

    let p = pill(0);
    p.append(&clock::build());
    window.set_child(Some(&p));
    window
}

fn build_right(app: &gtk4::Application) -> gtk4::ApplicationWindow {
    let window = shell_window(app);
    window.set_anchor(Edge::Right, true);
    window.set_anchor(Edge::Top, true);
    window.set_margin(Edge::Top, 8);
    window.set_margin(Edge::Right, 12);

    let p = pill(2);
    p.append(&system::build());
    p.append(&network::build());
    p.append(&bluetooth::build());
    p.append(&media::build());
    p.append(&volume::build());
    p.append(&battery::build());
    p.append(&power::build());
    window.set_child(Some(&p));
    window
}

fn load_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_string(include_str!("style.css"));
    if let Some(display) = gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(&display, &provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }
}
