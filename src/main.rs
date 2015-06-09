extern crate gdk;
extern crate gtk;
extern crate pango;
extern crate libgurbani;

use libgurbani::{QueryParams, Scripture};
use pango::FontDescription;

use gtk::signal::{self, TreeViewSignals};
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
    let search_results: TreeView  = builder.get_object("search_results").unwrap();
    let results_store: ListStore = builder.get_object("search_results_store").unwrap();

    let fullscreen_window: Window = builder.get_object("fullscreen_window").unwrap();
    let gurmukhi_label: Label = builder.get_object("gurmukhi").unwrap();
    let translation_label: Label = builder.get_object("translation").unwrap();
    let transliteration_label: Label = builder.get_object("transliteration").unwrap();

    let gurmukhi_font = FontDescription::from_string("gurbaniwebthick normal 12");
    gurmukhi_label.override_font(&gurmukhi_font);
    translation_label.override_font(&gurmukhi_font);
    transliteration_label.override_font(&gurmukhi_font);
    search_entry.override_font(&gurmukhi_font);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        signal::Inhibit(true)
    });

    search_results.connect_row_activated(move |view: TreeView, path, _| {
        create_fullscreen_window(&fullscreen_window);

        let model = view.get_model().unwrap();
        let mut iter = TreeIter::new();
        if model.get_iter(&mut iter, &path) {
            let s = model.get_value(&iter, 0);
            gurmukhi_label.set_text(&s.get_string().unwrap());
            translation_label.set_text(&s.get_string().unwrap());
            transliteration_label.set_text(&s.get_string().unwrap());
        }
    });
    search_button.connect_clicked(move |_| search(&search_entry, &results_store));

    window.show_all();
}

fn search(_: &Entry, store: &ListStore) {
    let conn = libgurbani::connect();
    let params = QueryParams::new().scripture(Scripture::SGGS).page(1);
    let results = libgurbani::query(&conn, params);

    let mut iter = gtk::TreeIter::new();
    for res in results.iter() {
        store.append(&mut iter);
        store.set_string(&iter, 0, &res.gurmukhi);
    }
}

fn create_fullscreen_window(window: &Window) {
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
