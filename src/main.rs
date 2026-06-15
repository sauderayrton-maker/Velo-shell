use gtk4::prelude::*;

mod card;
mod hypr;
mod modules;
mod window;

fn main() -> glib::ExitCode {
    let app = gtk4::Application::builder().application_id("com.velo.Shell").build();

    app.connect_activate(|app| {
        window::build_window(app).present();
    });

    app.run()
}
