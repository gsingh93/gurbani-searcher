extern crate glib;
extern crate gdk;
extern crate gtk;
extern crate gtk_sys as ffi;
extern crate pango;
extern crate libgurbani;
extern crate env_logger;

use std::rc::Rc;

use libgurbani::{DbConnection, QueryParams, Scripture};
use pango::FontDescription;

use glib::{Value, Type};

use gtk::signal::{self, TreeViewSignals};
use gtk::widgets::*;
use gtk::traits::*;

const UI_FILE: &'static str = "resources/gui.ui";

fn main() {
    env_logger::init().unwrap();
    gtk::init().ok().expect("Gtk initialization failed");
    init_gui();
    gtk::main();
}

#[derive(Clone)]
struct Slide {
    gurmukhi: Label,
    translation: Label,
    transliteration: Label
}

impl Slide {
    fn new(gurmukhi: Label, translation: Label, transliteration: Label) -> Self {
        Slide { gurmukhi: gurmukhi, translation: translation, transliteration: transliteration }
    }

    fn set_text(&self, gurmukhi: &str, translation: &str, transliteration: &str) {
        self.gurmukhi.set_text(gurmukhi);
        self.translation.set_text(translation);
        self.transliteration.set_text(transliteration);
    }
}

fn init_gui() {
    let builder = Builder::new_from_file(UI_FILE).unwrap();

    let window: Window = builder.get_object("window").unwrap();
    let container: Box = builder.get_object("main_container").unwrap();

    let search_button: Button = builder.get_object("search_button").unwrap();
    let search_entry: Entry  = builder.get_object("search_entry").unwrap();
    let search_results: TreeView  = builder.get_object("search_results").unwrap();
    let results_scroll_window: ScrolledWindow  =
        builder.get_object("search_results_scroll_window").unwrap();
    let results_store: ListStore = builder.get_object("search_results_store").unwrap();

    let shabad_lines: TreeView = builder.get_object("shabad_lines").unwrap();
    let shabad_store: ListStore = builder.get_object("shabad_store").unwrap();

    let fullscreen_window: Window = builder.get_object("fullscreen_window").unwrap();
    let provider = CssProvider::load_from_path("resources/theme.css");
    unsafe { ffi::gtk_style_context_add_provider(fullscreen_window.get_style_context(),
                                                 provider.pointer,
                                                 800) };

    let slide_box: Box = builder.get_object("slide").unwrap();
    let shabad_lines_scroll_window: ScrolledWindow =
        builder.get_object("shabad_lines_scroll_window").unwrap();

    let gurmukhi_font = FontDescription::from_string("gurbaniwebthick normal");

    let gurmukhi1_label: Label = builder.get_object("gurmukhi1").unwrap();
    let translation1_label: Label = builder.get_object("translation1").unwrap();
    let transliteration1_label: Label = builder.get_object("transliteration1").unwrap();
    gurmukhi1_label.override_font(&gurmukhi_font);
    let slide_fullscreen = Slide::new(gurmukhi1_label, translation1_label, transliteration1_label);

    let gurmukhi_label: Label = builder.get_object("gurmukhi").unwrap();
    let translation_label: Label = builder.get_object("translation").unwrap();
    let transliteration_label: Label = builder.get_object("transliteration").unwrap();
    gurmukhi_label.override_font(&gurmukhi_font);
    let slide = Slide::new(gurmukhi_label, translation_label, transliteration_label);

    search_entry.override_font(&gurmukhi_font);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        signal::Inhibit(true)
    });

    {
        let slide = slide.clone();
        let slide_fullscreen = slide_fullscreen.clone();
        shabad_lines.connect_row_activated(move |view: TreeView, path, _| {
            let model = view.get_model().unwrap();
            let mut iter = TreeIter::new();
            if model.get_iter(&mut iter, &path) {
                let gurmukhi = model.get_value(&iter, 0);
                let transliteration = model.get_value(&iter, 1);
                let translation = model.get_value(&iter, 2);

                let gurmukhi = gurmukhi.get_string().unwrap();
                let translation = translation.get_string().unwrap();
                let transliteration = transliteration.get_string().unwrap();

                slide.set_text(&gurmukhi, &translation, &transliteration);
                slide_fullscreen.set_text(&gurmukhi, &translation, &transliteration);
            }
        });
    }

    let conn = Rc::new(DbConnection::connect());
    {
        let conn = conn.clone();
        search_results.connect_row_activated(move |view: TreeView, path, _| {
            create_fullscreen_window(&fullscreen_window);

            container.remove(&results_scroll_window);
            container.add(&slide_box);

            let model = view.get_model().unwrap();
            let mut iter = TreeIter::new();
            let (hymn, id) = if model.get_iter(&mut iter, &path) {
                (model.get_value(&iter, 1).get::<i64>() as i16, model.get_value(&iter, 2).get())
            } else {
                unreachable!()
            };

            // FIXME: Remove SGGS restriction
            let params = QueryParams::new().scripture(Scripture::SGGS).hymn(hymn);
            let mut stmt = conn.query(params);
            let results = stmt.query();
            let mut iter = TreeIter::new();
            for res in results {
                shabad_store.append(&mut iter);
                let gurmukhi = res.gurmukhi();
                let transliteration = res.transliteration();
                let translation = res.translation();

                shabad_store.set_string(&iter, 0, &gurmukhi);
                shabad_store.set_string(&iter, 1, &transliteration);
                shabad_store.set_string(&iter, 2, &translation);

                if res.id() == id {
                    slide.set_text(&gurmukhi, &translation, &transliteration);
                    slide_fullscreen.set_text(&gurmukhi, &translation, &transliteration);
                }
            }
            container.add(&shabad_lines_scroll_window);
        });
    }
    search_button.connect_clicked(move |_| search(&conn, &search_entry, &results_store));

    window.show_all();
}

fn search<'conn>(conn: &DbConnection, search_entry: &Entry, store: &ListStore) {
    let text = search_entry.get_text().unwrap();
    if text == "" {
        return;
    }
    // FIXME: Remove SGGS restriction
    let params = QueryParams::new().scripture(Scripture::SGGS).gurmukhi(text);
    let mut stmt = conn.query(params);
    let results = stmt.query();
    let mut iter = gtk::TreeIter::new();
    for res in results {
        store.append(&mut iter);
        store.set_string(&iter, 0, &res.gurmukhi());

        let mut hymn_num = Value::new();
        hymn_num.init(Type::I64);
        hymn_num.set(&res.hymn());
        store.set_value(&iter, 1, &hymn_num);

        let mut id = Value::new();
        id.init(Type::I32);
        id.set(&res.id());
        store.set_value(&iter, 2, &id);
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
