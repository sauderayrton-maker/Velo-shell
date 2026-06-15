use gtk4::gdk;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use crate::modules::{battery, bluetooth, clock, media, network, power, system, volume, workspaces};

pub fn build_window(app: &gtk4::Application) -> gtk4::ApplicationWindow {
    load_css();

    let window = gtk4::ApplicationWindow::builder().application(app).title("Velo Shell").decorated(false).build();

    window.init_layer_shell();
    window.set_layer(Layer::Top);
    window.set_namespace(Some("velo-shell"));
    for edge in [Edge::Left, Edge::Right, Edge::Top] {
        window.set_anchor(edge, true);
    }
    window.set_anchor(Edge::Bottom, false);
    window.auto_exclusive_zone_enable();

    // ── Left: workspaces + active window title ──
    let left = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(10).halign(gtk4::Align::Start).css_classes(vec!["bar-section"]).build();
    left.append(&workspaces::build());

    // ── Centre: clock ──
    let center = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).halign(gtk4::Align::Center).css_classes(vec!["bar-section"]).build();
    center.append(&clock::build());

    // ── Right: system stats, network, bluetooth, media, volume, battery, power ──
    let right = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(10).halign(gtk4::Align::End).css_classes(vec!["bar-section"]).build();
    right.append(&system::build());
    right.append(&network::build());
    right.append(&bluetooth::build());
    right.append(&media::build());
    right.append(&volume::build());
    right.append(&battery::build());
    right.append(&power::build());

    let bar = gtk4::CenterBox::builder().css_classes(vec!["bar"]).build();
    bar.set_start_widget(Some(&left));
    bar.set_center_widget(Some(&center));
    bar.set_end_widget(Some(&right));

    window.set_child(Some(&bar));

    window
}

fn load_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_string(include_str!("style.css"));
    if let Some(display) = gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(&display, &provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }
}
