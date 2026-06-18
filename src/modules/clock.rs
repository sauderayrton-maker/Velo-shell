//! Clock module: time + date, with a calendar card on click.

use gtk4::prelude::*;

use crate::card;

pub fn build() -> gtk4::Widget {
    let time_label = gtk4::Label::builder().css_classes(vec!["clock-time"]).build();
    let date_label = gtk4::Label::builder().css_classes(vec!["clock-date"]).build();

    let row = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(8).build();
    row.append(&time_label);
    row.append(&date_label);

    let button = gtk4::Button::builder().css_classes(vec!["module", "clock-module", "flat"]).build();
    button.set_child(Some(&row));

    let calendar = gtk4::Calendar::builder().css_classes(vec!["velo-calendar"]).build();
    let card_window = card::build(&calendar);

    button.connect_clicked(move |btn| {
        card::toggle(&card_window, btn.upcast_ref(), || {
            let today = glib::DateTime::now_local().unwrap();
            calendar.select_day(&today);
        });
    });

    update(&time_label, &date_label);
    glib::timeout_add_seconds_local(1, move || {
        update(&time_label, &date_label);
        glib::ControlFlow::Continue
    });

    button.upcast()
}

fn update(time_label: &gtk4::Label, date_label: &gtk4::Label) {
    let now = glib::DateTime::now_local().unwrap();
    time_label.set_label(&now.format("%H:%M").unwrap_or_default());
    date_label.set_label(&now.format("%a %d %b").unwrap_or_default());
}
