use gtk4::gdk;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use crate::modules::{battery, bluetooth, clock, media, network, power, system, volume, workspaces};

pub fn build_windows(app: &gtk4::Application) {
    load_css();
    build_left(app).present();
    build_center(app).present();
    build_right(app).present();
}

fn shell_window(app: &gtk4::Application) -> gtk4::ApplicationWindow {
    let w = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("Velo Shell")
        .decorated(false)
        .build();
    w.init_layer_shell();
    w.set_layer(Layer::Top);
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
    // Only one pill owns the exclusive zone — this reserves top_margin + pill
    // height so niri knows to place windows below the floating strip.
    window.auto_exclusive_zone_enable();

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
