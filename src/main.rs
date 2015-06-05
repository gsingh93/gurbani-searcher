extern crate gdk;
extern crate gtk;
extern crate pango;
extern crate libgurbani;

use libgurbani::{QueryParams, Scripture};
use pango::FontDescription;

use gtk::signal;
use gtk::widgets::*;
use gtk::traits::*;

const UI_FILE: &'static str = "resources/gui.ui";

fn main() {
    gtk::init().ok().expect("Gtk initialization failed");
    init_gui();
    gtk::main();
}

fn init_gui() {
    let builder = Builder::new_from_file(UI_FILE).unwrap();

    let window: Window = builder.get_object("window").unwrap();
    let search_button: Button = builder.get_object("search_button").unwrap();
    let search_entry: Entry  = builder.get_object("search_entry").unwrap();
    let results_store: ListStore = builder.get_object("search_results_store").unwrap();

    search_entry.override_font(&FontDescription::from_string("gurbaniwebthick normal 12").unwrap());

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        signal::Inhibit(true)
    });

    search_button.connect_clicked(move |_| search(&search_entry, &results_store));

    window.show_all();
}

fn search(_: &Entry, store: &ListStore) {
    let conn = libgurbani::connect();
    let params = QueryParams { scripture: Some(Scripture::SGGS), page: Some(1) };
    let results = libgurbani::query(&conn, params);

    let mut iter = gtk::TreeIter::new();
    for res in results.iter() {
        store.append(&mut iter);
        store.set_string(&iter, 0, &res.gurmukhi);
    }
}

fn create_fullscreen_window() {
    let window = Window::new(gtk::WindowType::TopLevel).unwrap();

    // This is needed for tiling window managers so the window can be positioned
    window.set_type_hint(gdk::WindowTypeHint::Dialog);

    window.show_all();
    let gdk_window = window.get_window().unwrap();

    // TODO: Move window to other display
    // let screen = gdk_window.get_screen();
    // let n = screen.get_n_monitors();
    // let geometry = screen.get_monitor_geometry(0);

    window.move_(0, 0);
    gdk_window.fullscreen();
}
