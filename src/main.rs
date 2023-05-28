use adw::{ApplicationWindow, ActionRow, EntryRow, HeaderBar, prelude::*};
use gtk::{Application, ListBox, Box, Orientation, Entry, ProgressBar, Label, Button, glib};
use log::{info, error, debug};
use std::thread;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
mod setup; mod finder;

use glib::Sender;
use glib::Receiver;
use glib::MainContext;
use glib::idle_add;

use finder::Finder;

struct Callbacks {
    window: ApplicationWindow,
    gtk_box: Box,
    path: String,
    exts: String,
}

#[derive(Clone)]
struct ProgressAnimate {
    flag: Arc<bool>,
    sender: Sender<bool>,
}
impl ProgressAnimate {
    fn new(sender: Sender<bool>) -> Self {
        ProgressAnimate { flag: Arc::new(true), sender: sender }
    }

    fn animate_progress(&mut self, path: String, extensions: String) {
        let mut self_clone = self.clone();
        
        let should_exit = Arc::new(AtomicBool::new(false));
        
        let find_thread = {
            let should_exit = Arc::clone(&should_exit);
            thread::spawn(move || {
                while !should_exit.load(Ordering::Relaxed) {
                    self_clone.sender.send(true);
                    thread::sleep(std::time::Duration::from_millis(700));
                
                    if should_exit.load(Ordering::Relaxed) {
                        self_clone.sender.send(false);
                        break;
                    }
                }
            });
        };
        find(path, extensions);
        
        should_exit.store(true, Ordering::Relaxed);
    }
}

fn find(path: String, extensions: String) {
    debug!("{} {}", path, extensions);
    let mut find_obj = Finder::new(path, extensions);
    println!("{:?}", find_obj.find());
}

impl Callbacks {
    fn new(window: ApplicationWindow, gtk_box: Box, path: String, exts: String) -> Self {
        Callbacks { window: window, gtk_box: gtk_box, path: path, exts: exts }
    }


    fn find_btt_callback(&self) {
        let prog = ProgressBar::new();
        self.gtk_box.append(&prog);
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        prog.pulse();

        let mut prog_cb = ProgressAnimate::new(sender.clone());
        let path = self.path.clone();
        let exts = self.exts.clone();

        // Inicia el hilo secundario que actualiza el widget
        thread::spawn(move || {
            prog_cb.animate_progress(path, exts);
        });

        let box_clone = self.gtk_box.clone();
        let prog_clone = prog.clone();
        receiver.attach(None, move |msg| {

            // Actualiza el widget con los datos recibidos del hilo secundario
            if msg {
                // La tarea está en progreso, utiliza pulse() para mostrar una animación de progreso continuo
                prog.pulse();
            } else {
                // La tarea ha finalizado, establece la barra de progreso en 100% (1.0)
                box_clone.remove(&prog_clone);
            }
            glib::Continue(msg)
            
        });
        
    }
}

fn main() {
    setup::setup();
    info!("[root] Log initialized!");
    info!("[root] Initializing gtk window");

    let app = Application::builder()
        .application_id("com.placeholder.finder")
        .build();

    app.connect_startup(|_| {
        info!("[adw startup] adw::init()");
        if let Err(err) = adw::init() {
            error!("[adw startup] {}", err)
        };
    });

    app.connect_activate(|app| {
        info!("[adw activation] Creating window");
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(600)
            .default_height(300)
            .title("FinderGUI")
            .build();

        let notebook = gtk::Notebook::builder()
            .show_tabs(false)
            .show_border(false)
            .build();

        let main_box = Box::new(Orientation::Vertical, 0);
        let widgets_box = Box::builder()
            .margin_bottom(20)
            .margin_end(20)
            .margin_start(20)
            .margin_top(20)
            .orientation(Orientation::Vertical)
            .build();
        

        let header = HeaderBar::builder()
            .title_widget(&adw::WindowTitle::new("FinderGUI", "The newest Finder GUI"))
            .build();

        let placeholder_label = Label::builder()
            .use_markup(true)
            .label("<b>Input set</b>")
            .build();

        let input_list_box = ListBox::builder()
            .margin_top(22)
            .margin_end(22)
            .margin_bottom(10)
            .margin_start(22)
            // the content class makes the list look nicer
            .css_classes(vec![String::from("content")])
            .build();

        // let exts_row = ActionRow::builder()
        //     .activatable(true)
        //     .selectable(false)
        //     .title("Extensions")
        //     .subtitle("Choose from a list the ext category, or create new category by writing and separating the extensions with this character |")
        //     .build();
        let exts_entry_row = EntryRow::builder()
            .activatable(true)
            .selectable(false)
            .title("Extensions (Separated by | )")
            .build();

        let directory_entry_row = EntryRow::builder()
            .activatable(true)
            .selectable(false)
            .title("Find path")
            .build();

        let btt_box = Box::builder()
            .margin_bottom(26)
            .margin_end(26)
            .margin_start(26)
            .margin_top(26)
            .homogeneous(true)
            .spacing(3)
            .build();

        let browse_path_btt = Button::builder()
            .label("Select a specific path")
            .hexpand_set(true)
            .vexpand_set(true)
            .build();
        let find_btt = Button::builder()
            .label("Find")
            .hexpand_set(true)
            .vexpand_set(true)
            .build();

        let der_clone = directory_entry_row.clone();
        let eer = exts_entry_row.clone();
        let win = window.clone();
        let mb = main_box.clone();
        find_btt.connect_clicked(move |_| {
            let cbs = Callbacks::new(win.clone(), mb.clone(), der_clone.text().to_string(), eer.text().to_string());
            cbs.find_btt_callback();
        });

        btt_box.append(&browse_path_btt);
        btt_box.append(&find_btt);
        
        input_list_box.append(&exts_entry_row);
        input_list_box.append(&directory_entry_row);

        widgets_box.append(&placeholder_label);
        widgets_box.append(&input_list_box);
        widgets_box.append(&btt_box);
        
        main_box.append(&header);

        let placeholder_label = Label::new(Some("Main Page"));

        notebook.append_page(&widgets_box, Some(&placeholder_label));
        main_box.append(&notebook);

        window.set_content(Some(&main_box));
        window.show();
    });

    app.run();
}

