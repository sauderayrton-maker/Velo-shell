use gtk4::gio;
use gtk4::prelude::*;

mod card;
mod hypr;
mod modules;
mod window;

fn main() -> glib::ExitCode {
    let app = gtk4::Application::builder()
        .application_id("com.velo.Shell")
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    app.connect_activate(|app| {
        window::build_windows(app);
    });

    // Silently ignore any file-open activation attempts.
    app.connect_open(|_, _, _| {});

    app.run()
}
