use std::process;

use action::click_app;
use freedesktop::desktop_entry::get_available_apps;

use gtk::{prelude::*, Label, ListBox, ListBoxRow, ScrolledWindow, SearchEntry};

mod action;
mod args;
mod config;
mod freedesktop;
mod ipc;
mod style;
mod util;

const APP_ID: &str = "com.igorunderplayer.IUMenu";

fn main() {
    let args = args::parse_arguments();

    // Client mode: send toggle command to server
    if args.toggle {
        let rt = tokio::runtime::Runtime::new().unwrap();
        match rt.block_on(ipc::send_toggle_command()) {
            Ok(_) => {
                println!("Toggle command sent successfully");
                process::exit(0);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    }

    // Server mode: run the GTK application with IPC
    gtk::init().expect("Failed to initialize GTK.");

    let app = gtk::Application::builder().application_id(APP_ID).build();

    app.connect_activate(move |app| {
        let config = config::load_from_file(&args.config);

        let sys_apps = get_available_apps();
        let mut entries: Vec<String> = sys_apps.iter().map(|(id, _)| id.clone()).collect();
        entries.sort_by(|a, b| {
            let app_a = &sys_apps[a];
            let app_b = &sys_apps[b];
            app_a.name.to_lowercase().cmp(&app_b.name.to_lowercase())
        });

        let window = gtk::ApplicationWindow::new(app);
        let window_config = config.window.unwrap_or(config::WindowConfig::default());

        window.set_title(Some("IUMenu"));
        window.set_default_size(window_config.width, window_config.height);
        window.set_decorated(false);
        window.set_resizable(false);
        window.set_size_request(window_config.width, window_config.height);
        window.set_modal(true);

        let style_config = config.style.unwrap_or(config::StyleConfig::default());

        window.set_opacity(style_config.opacity.unwrap_or(1.0));

        if let Some(path) = style_config.path {
            style::apply_custom_css(path.to_str().unwrap());
        }

        let search_entry = SearchEntry::new();
        search_entry.set_vexpand(false);
        search_entry.set_hexpand(true);
        search_entry.set_height_request(72);

        search_entry.add_css_class("search-entry");

        let main_grid = gtk::Box::new(gtk::Orientation::Vertical, 0);

        main_grid.set_hexpand(true);
        main_grid.set_vexpand(true);

        main_grid.append(&search_entry);

        let scroll = ScrolledWindow::new();
        scroll.set_max_content_height(328);
        scroll.set_vexpand(true);

        let list_box = ListBox::new();
        list_box.set_vexpand(true);

        list_box.add_css_class("list-box");

        let rows: Vec<ListBoxRow> = entries
            .iter()
            .map(|app_id| {
                let app = &sys_apps[app_id];
                let row = gtk::ListBoxRow::new();

                let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                row_box.set_margin_start(8);
                row_box.set_margin_end(8);

                row.add_css_class("entry");
                row.set_height_request(48);

                let label = Label::new(Some(&app.name));
                let icon = gtk::Image::from_icon_name(&app.icon);
                icon.add_css_class("entry-icon");

                unsafe {
                    row.set_data("app-id", app.id.to_owned());
                }

                row_box.append(&icon);
                row_box.append(&label);
                row.set_child(Some(&row_box));

                list_box.append(&row);

                row
            })
            .collect();

        list_box.select_row(Some(&rows[0]));

        let last_active: String = String::default();
        list_box.connect_row_activated({
            let app = app.clone();
            let sys_apps = sys_apps.clone();
            let window = window.clone();
            let search_entry = search_entry.clone();
            let server_mode = args.server;
            let last_active = std::sync::Arc::new(std::sync::Mutex::new(last_active));
            move |_list_box, row| {
                println!("escolheu algo");
                let id;

                unsafe {
                    id = row.data::<String>("app-id").unwrap().as_ref().to_owned();
                }

                let mut last_active = last_active.lock().unwrap();

                if *last_active == id {
                    let app_data = sys_apps.get(&id).unwrap();
                    click_app(app_data);

                    if server_mode {
                        search_entry.set_text("");
                        window.set_visible(false);
                    } else {
                        app.quit();
                    }
                }

                *last_active = id.clone();
            }
        });

        search_entry.connect_changed({
            let rows = rows.clone();
            let sys_apps = sys_apps.clone();
            let list_box = list_box.clone();
            move |entry| {
                let query = entry.text().to_lowercase();

                for row in &rows {
                    let id: String;
                    unsafe {
                        id = row.data::<String>("app-id").unwrap().as_ref().to_owned();
                    }

                    if let Some(app) = sys_apps.get(&id) {
                        row.set_visible(
                            app.name.to_lowercase().contains(&query)
                                || app.keywords.to_lowercase().contains(&query),
                        );
                    }
                }

                list_box.select_row(rows.iter().find(|p| p.is_visible()));
            }
        });

        let controller = gtk::EventControllerKey::new();

        controller.set_propagation_phase(gtk::PropagationPhase::Capture);

        controller.connect_key_pressed({
            let sys_apps = sys_apps.clone();
            let list_box = list_box.clone();
            let rows = rows.clone();
            let app = app.clone();
            let window = window.clone();
            let search_entry = search_entry.clone();
            let server_mode = args.server;
            move |_, keyval, keycode, _state| {
                println!("Key pressed: {:?}, Keycode: {:?}", keyval, keycode);
                let visible_rows: Vec<ListBoxRow> = rows
                    .clone()
                    .into_iter()
                    .filter_map(|widget| widget.downcast::<ListBoxRow>().ok())
                    .filter(|p| p.is_visible())
                    .collect();

                let current_index = visible_rows
                    .iter()
                    .position(|row| list_box.selected_row() == Some(row.clone()));

                match keyval {
                    gtk::gdk::Key::Escape => {
                        if server_mode {
                            search_entry.set_text("");
                            window.set_visible(false);
                        } else {
                            app.quit();
                        }
                        gtk::glib::Propagation::Proceed
                    }
                    gtk::gdk::Key::Return => {
                        println!("enter");
                        if let Some(row) = list_box.selected_row() {
                            println!("escolheu algo");
                            let id;

                            unsafe {
                                id = row.data::<String>("app-id").unwrap().as_ref().to_owned();
                            }

                            let app_data = sys_apps.get(&id).unwrap();
                            click_app(app_data);

                            if server_mode {
                                search_entry.set_text("");
                                window.set_visible(false);
                            } else {
                                app.quit();
                            }
                        }
                        gtk::glib::Propagation::Stop
                    }
                    gtk::gdk::Key::Down => {
                        println!("key press down");
                        if let Some(index) = current_index {
                            if index + 1 < visible_rows.len() {
                                let next_row = &visible_rows[index + 1];
                                list_box.select_row(Some(next_row));
                                next_row.activate();
                                search_entry.grab_focus();
                            }
                        } else if !visible_rows.is_empty() {
                            let next_row = &visible_rows[0];
                            list_box.select_row(Some(next_row));
                            next_row.activate();
                            search_entry.grab_focus();
                        }
                        gtk::glib::Propagation::Stop
                    }
                    gtk::gdk::Key::Up => {
                        if let Some(index) = current_index {
                            if index > 0 {
                                let next_row = &visible_rows[index - 1];
                                list_box.select_row(Some(next_row));
                                next_row.activate();
                                search_entry.grab_focus();
                            }
                        } else if !visible_rows.is_empty() {
                            let next_row = &visible_rows[0];
                            list_box.select_row(Some(next_row));
                            next_row.activate();
                            search_entry.grab_focus();
                        }
                        gtk::glib::Propagation::Stop
                    }
                    _ => gtk::glib::Propagation::Proceed,
                }
            }
        });

        scroll.set_child(Some(&list_box));
        main_grid.append(&scroll);

        search_entry.add_controller(controller);
        window.set_child(Some(&main_grid));

        // Don't show window initially in server mode
        if !args.server {
            window.present();
            search_entry.grab_focus();
        } else {
            // Setup IPC server for server mode
            use std::sync::mpsc;

            let (tx, rx) = mpsc::channel::<()>();
            let window_weak = window.downgrade();
            let search_entry_weak = search_entry.downgrade();

            // Handle toggle messages on GTK main thread
            gtk::glib::spawn_future_local(async move {
                loop {
                    // Check for messages from IPC server
                    if let Ok(_) = rx.try_recv() {
                        if let (Some(window), Some(search_entry)) =
                            (window_weak.upgrade(), search_entry_weak.upgrade())
                        {
                            if window.is_visible() {
                                search_entry.set_text("");
                                window.set_visible(false);
                            } else {
                                window.present();
                                search_entry.grab_focus();
                            }
                        }
                    }
                    // Small delay to avoid busy-waiting
                    gtk::glib::timeout_future(std::time::Duration::from_millis(50)).await;
                }
            });

            // Start IPC server in a separate thread
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    if let Err(e) = ipc::start_ipc_server(move || {
                        let _ = tx.send(());
                    })
                    .await
                    {
                        eprintln!("Failed to start IPC server: {}", e);
                    }

                    // Keep the runtime alive
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                });
            });
        }

        window.connect_hide({
            let server_mode = args.server;
            move |window| {
                if !server_mode {
                    window.close();
                    process::exit(0);
                }
                // In server mode, just hide the window instead of closing
            }
        });

        window.connect_close_request({
            let server_mode = args.server;
            move |window| {
                if server_mode {
                    // In server mode, hide instead of closing
                    window.set_visible(false);
                    gtk::glib::Propagation::Stop
                } else {
                    // In normal mode, allow close
                    gtk::glib::Propagation::Proceed
                }
            }
        });
    });

    let gtk_args: Vec<String> = vec![];
    app.run_with_args(&gtk_args);
}
