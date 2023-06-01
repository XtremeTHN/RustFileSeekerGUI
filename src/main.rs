use adw::{ApplicationWindow, EntryRow, HeaderBar, prelude::*};
use gtk::{Application, ListBox, Box as GtkBox, Orientation, ProgressBar, Label, Button, TreeView, glib, Notebook};
use log::{info, error};
use std::path::PathBuf;
use std::thread;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
mod setup; mod finder; mod stated;

use glib::Sender;

use finder::Finder;


enum SendTypes {
    VectorValue(Vec<PathBuf>),
    Error(String),
    Bool(bool),
}

struct Callbacks {
    gtk_box: GtkBox,
    nb: Notebook,
    path: String,
    exts: String,
}

fn append_text_column(tree: &gtk::TreeView, title: &str, col: i32) {
    let column = gtk::TreeViewColumn::builder()
        .title(title)
        .build();
    let cell = gtk::CellRendererText::new();

    column.pack_start(&cell, true);
    column.add_attribute(&cell, "text", col);
    tree.append_column(&column);
}

#[derive(Clone)]
struct ProgressAnimate {
    should_exit: Arc<AtomicBool>,
    sender: Sender<SendTypes>,
}
impl ProgressAnimate {
    fn new(sender: Sender<SendTypes>) -> Self {
        ProgressAnimate { should_exit: Arc::new(AtomicBool::new(false)), sender: sender }
    }

    fn animate_progress(&mut self, path: String, extensions: String) {
        let self_clone = self.clone();
        
        let _find_thread = {
            let should_exit = Arc::clone(&self.should_exit);
            thread::spawn(move || {
                while !should_exit.load(Ordering::Relaxed) {
                    self_clone.sender.send(SendTypes::Bool(true)).unwrap_or_else(|err| {
                        error!("main::ProgressAnimate: Error while sending continue signal to the main thread!. Error: {}", err);
                        info!("main::ProgressAnimate: Create a new issue in the github page");
                    });
                    thread::sleep(std::time::Duration::from_millis(700));
                
                    if should_exit.load(Ordering::Relaxed) {
                        self_clone.sender.send(SendTypes::Bool(false)).unwrap_or_else(|err| {
                            error!("main::ProgressAnimate: Error while sending continue signal to the main thread!. Error: {}", err);
                            info!("main::ProgressAnimate: Create a new issue in the github page");
                        });
                        break;
                    }
                }
            });
        };
        let files = find(path, extensions);
        if let Err(err) = files.clone() {
            self.should_exit.store(true, Ordering::Relaxed);
            self.sender.send(SendTypes::Error(err)).unwrap_or_else(|err| {
                error!("main::ProgressAnimate: Error while sending continue signal to the main thread!. Error: {}", err);
                info!("main::ProgressAnimate: Create a new issue in the github page");
            });
            ()
        } else {
            self.should_exit.store(true, Ordering::Relaxed);
            self.sender.send(SendTypes::VectorValue(files.unwrap())).unwrap_or_else(|err| {
                error!("main::ProgressAnimate: Error while sending continue signal to the main thread!. Error: {}", err);
                info!("main::ProgressAnimate: Create a new issue in the github page");
            });
            ()
        }
    }
}

fn find(path: String, extensions: String) -> Result<Vec<PathBuf>, String> {
    let mut find_obj = Finder::new(path, extensions);
    if let Err(_) = find_obj.find() {
        Err(String::from("No files founded in the specified dir"))
    } else {
        Ok(find_obj.get_all())
    }
}

impl Callbacks {
    fn new(gtk_box: GtkBox, path: String, exts: String, nb: Notebook) -> Self {
        Callbacks { gtk_box: gtk_box, path: path, exts: exts, nb: nb}
    }


    fn find_btt_callback(&self, transient_for: ApplicationWindow) {
        let prog = ProgressBar::new();
        self.gtk_box.append(&prog);
        info!("main::Callbacks::find_bt_callback: Creating glib channel...");
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        prog.pulse();

        let mut prog_cb = ProgressAnimate::new(sender.clone());
        let path = self.path.clone();
        let exts = self.exts.clone();

        info!("main::Callbacks::find_bt_callback: Finding files in another thread...");
        // Inicia el hilo secundario que actualiza el widget
        thread::spawn(move || {
            prog_cb.animate_progress(path, exts);
        });

        let box_clone = self.gtk_box.clone();
        let prog_clone = prog.clone();

        let nb_clone = self.nb.clone();
        let _self_clone = self.clone();
        let tf = transient_for.clone();
        receiver.attach(None, move |msg| {

            // Actualiza el widget con los datos recibidos del hilo secundario
            match msg {
                SendTypes::Bool(_) => {
                    prog.pulse();
                    glib::Continue(true)
                }
                SendTypes::VectorValue(vec_files) => {
                    info!("main::Callbacks::find_bt_callback: Recivied list of files from another thread");
                    if let Ok(nb_box) = build_page2(vec_files, nb_clone.clone(), tf.clone()) {
                        let placeholder_label = Label::builder()
                            .use_markup(true)
                            .label("<b>Input set</b>")
                            .build();

                        nb_clone.append_page(&nb_box, Some(&placeholder_label));
                        nb_clone.next_page();
                    }
                    box_clone.remove(&prog_clone);
                    glib::Continue(false)
                }
                SendTypes::Error(err) => {
                    info!("main::Callbacks::find_bt_callback: Recivied error from another thread. Error: {}", err);
                    let msg_diag = adw::MessageDialog::builder()
                        .heading("Empty!")
                        .body(&err)
                        .transient_for(&tf.clone())
                        .build();
                    msg_diag.add_response("Ok", "Ok");
                    msg_diag.present();
                    box_clone.remove(&prog_clone);
                    glib::Continue(false)
                }
            }
        });
    }
}

fn build_page2(files: Vec<PathBuf>, nb: Notebook, transient_for: ApplicationWindow) -> Result<GtkBox, ()> {
    let config = setup::load_conf();

    let scrolled_window = gtk::ScrolledWindow::builder()
            .margin_top(12)
            .margin_end(12)
            .margin_bottom(12)
            .margin_start(12)
            .build();

    let page2_box = GtkBox::builder()
        .margin_bottom(20)
        .margin_end(20)
        .margin_start(20)
        .margin_top(20)
        .orientation(Orientation::Vertical)
        .build();

    let label = Label::builder()
        .use_markup(true)
        .label("<b>Results</b>")
        .build();

    let return_button = Button::builder()
        .label("Return to the main page")
        .build();

    return_button.connect_clicked(move |_| {
        nb.prev_page();
        nb.remove_page(Some(1));
    });

    let stated_obj = stated::Stated::new(config);

    if let Err(err) = stated_obj.stat_and_insert(files) {
        
        let msg_diag = adw::MessageDialog::new(Some(&transient_for), Some("Failed"), Some(&err));
        msg_diag.add_response("Ok", "Ok");
        msg_diag.present();
        return Err(());
    };

    let list_store = stated_obj.get_liststore();
    let treeview = TreeView::builder()
        .model(&list_store)
        .vexpand(true)
        .build();

    let cols = vec!["Name", "Extension", "Path", "Total Size", "Date of modification"];

    for (pos, col) in cols.iter().enumerate() {
        append_text_column(&treeview, col, pos as i32);
    };

    scrolled_window.set_child(Some(&treeview));

    page2_box.append(&label);
    page2_box.append(&scrolled_window);
    page2_box.append(&return_button);

    return Ok(page2_box);
}

fn main() {
    setup::setup();
    info!("main: Log initialized!");
    info!("main: Initializing gtk window");

    let app = Application::builder()
        .application_id("com.placeholder.finder")
        .build();

    info!("main: Loading settings...");
    let configs: setup::YamlConfiguration = setup::load_conf();

    let conf_clone = configs.clone();
    app.connect_startup(move |_| {
        info!("Application::connect_startup: Initializing adwaita...");
        let theme = conf_clone.interface_configurations.color_scheme.clone();
        if conf_clone.interface_configurations.enable_adw {
            if let Err(err) = adw::init() {
                error!("{}", err)
            };
            info!("Application::connect_startup: Applying settings theme");
            let adw_settings = adw::StyleManager::default();
            let another_settings = gtk::Settings::default().unwrap();
            if theme == String::from("dark") {
                adw_settings.set_color_scheme(adw::ColorScheme::ForceDark);
                another_settings.set_gtk_error_bell(false);
            } else if theme == String::from("light") {
                adw_settings.set_color_scheme(adw::ColorScheme::ForceLight);
                another_settings.set_gtk_error_bell(false);
            }
        } else {
            info!("Application::connect_startup: Adwaita is disabled in the configuration");
            info!("Application::connect_startup: Applying settings theme");
            let gtk_settings = gtk::Settings::default().unwrap();
            if theme == String::from("dark") {
                gtk_settings.set_gtk_application_prefer_dark_theme(true);
                gtk_settings.set_gtk_error_bell(false);
            } else if theme == String::from("light") {
                gtk_settings.set_gtk_application_prefer_dark_theme(false);
                gtk_settings.set_gtk_error_bell(false);
            }
        }
        

    });

    app.connect_activate(|app| {
        info!("Application::connect_activate: Creating window");
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

        // START PAGE 1
        info!("Application::connect_activate: Creating main page...");
            let main_box = GtkBox::new(Orientation::Vertical, 0);
            let widgets_box = GtkBox::builder()
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

            let btt_box = GtkBox::builder()
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
            let mb = main_box.clone();
            let nb = notebook.clone();
            let app_window_transient = window.clone();
            find_btt.connect_clicked(move |_| {
                let cbs = Callbacks::new(mb.clone(), der_clone.text().to_string(), eer.text().to_string(), nb.clone());
                cbs.find_btt_callback(app_window_transient.clone());
            });

            btt_box.append(&browse_path_btt);
            btt_box.append(&find_btt);
            
            input_list_box.append(&exts_entry_row);
            input_list_box.append(&directory_entry_row);

            widgets_box.append(&placeholder_label);
            widgets_box.append(&input_list_box);
            widgets_box.append(&btt_box);
        info!("Application::connect_activate: Success!");
        // ENF OF PAGE 1
        
        main_box.append(&header);

        let placeholder_label = Label::new(Some("Main Page"));

        notebook.append_page(&widgets_box, Some(&placeholder_label));

        // THE START OF THE PAGE 2 IS IN THE CALLBACK STRUCT
        // START OF PAGE 2

            // ^
            // |
            // |
            // PAGE 2 STARTS AT LINE 166 IN THE FUNCTION build_page2()

        // END OF PAGE 2

        let _placeholder_label = Label::new(Some("Main Page"));

        main_box.append(&notebook);

        window.set_content(Some(&main_box));
        window.show();
    });

    info!("Running app...");
    app.run();
}

