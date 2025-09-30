use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use ini::Ini;

#[derive(Clone)]
pub struct DesktopApp {
    pub id: String,
    pub name: String,
    pub comment: String,
    pub exec: String,
    pub keywords: String, // TODO: Implement this as list of string
    pub app_type: String,
    pub categories: String,
    pub no_display: bool,
    pub only_show_in: String,
    pub icon: String,
}

impl DesktopApp {
    pub fn new(
        id: String,
        name: String,
        comment: String,
        exec: String,
        keywords: String,
        app_type: String,
        categories: String,
        icon: String,
        no_display: bool,
        only_show_in: String,
    ) -> Self {
        Self {
            id,
            name,
            comment,
            exec,
            keywords,
            app_type,
            categories,
            icon,
            no_display,
            only_show_in,
        }
    }
}

pub fn get_available_apps() -> HashMap<String, DesktopApp> {
    let mut apps = HashMap::new();

    apps.extend(get_system_apps().into_iter());
    apps.extend(get_local_system_apps().into_iter());
    apps.extend(get_user_apps().into_iter());

    apps
}

pub fn get_system_apps() -> HashMap<String, DesktopApp> {
    let path = PathBuf::from("/usr/share/applications");
    read_desktop_files(&path)
}

pub fn get_local_system_apps() -> HashMap<String, DesktopApp> {
    let path = PathBuf::from("/usr/local/share/applications");
    read_desktop_files(&path)
}

pub fn get_user_apps() -> HashMap<String, DesktopApp> {
    let mut path = dirs::home_dir().unwrap();
    path.push(".local/share/applications");
    read_desktop_files(&path)
}

pub fn read_desktop_files(path: &PathBuf) -> HashMap<String, DesktopApp> {
    let mut apps = HashMap::new();
    let dir = Path::new(&path);
    let current_desktop = if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
        Some(desktop)
    } else {
        None
    };

    if dir.exists() && dir.is_dir() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "desktop").unwrap_or(false) {
                    if let Some(app_name) = path.file_stem() {
                        let id = app_name.to_string_lossy().into_owned();
                        if let Some(app_data) = parse_desktop_file(path.to_str().unwrap(), id.clone()) {
                            if should_show(&app_data, &current_desktop) {
                                continue;
                            }

                            apps.insert(id, app_data);
                        } {
                            println!("{} ignored due to error reading desktop file.", path.to_str().unwrap_or("app"));
                        }
                    }
                }
            }
        }
    }
    apps
}

pub fn should_show(app_data: &DesktopApp, current_desktop: &Option<String>) -> bool {
    app_data.app_type != "Application"
        || app_data.no_display
        || !is_in_show_in(&app_data.only_show_in, current_desktop)
}

pub fn is_in_show_in(show_in: &String, current_desktop: &Option<String>) -> bool {
    let only_show_in: Vec<&str> = show_in.split(';').filter(|s| !s.is_empty()).collect();

    if only_show_in.is_empty() {
        return true;
    }

    if let Some(current_desktop) = current_desktop {
        return only_show_in.contains(&current_desktop.as_str());
    }

    true
}

pub fn parse_desktop_file(path: &str, id: String) -> Option<DesktopApp> {
    let file_content = fs::read_to_string(&path).expect(&format!("Cant read file: {}", path));
    let ini = match Ini::load_from_str(&file_content) {
        Ok(v) => v,
        Err(_) => return None,
    };

    if let Some(section) = ini.section(Some("Desktop Entry")) {
        let name = section.get("Name").unwrap_or("Unknown").to_string();
        let comment = section.get("Comment").unwrap_or("").to_string();
        let icon = section.get("Icon").unwrap_or("").to_string();
        let app_type = section.get("Type").unwrap_or("").to_string();
        let categories = section.get("Categories").unwrap_or("Unknown").to_string();
        let keywords = section.get("Keywords").unwrap_or("").to_string();
        let only_show_in = section.get("OnlyShowIn").unwrap_or("").to_string();
        let no_display = section.get("NoDisplay").unwrap_or("").to_string() == "true";

        let exec_raw = section.get("Exec").unwrap_or("").to_string();

        let exec = clean_exec(&exec_raw);

        return Some(DesktopApp::new(
            id,
            name,
            comment,
            exec,
            keywords,
            app_type,
            categories,
            icon,
            no_display,
            only_show_in,
        ));
    }

    None
}

pub fn clean_exec(exec: &str) -> String {
    let placeholders = ["%U", "%u", "%F", "%f", "%i", "%c", "%k"];
    let mut cleaned = exec.to_string();

    for placeholder in placeholders {
        cleaned = cleaned.replace(placeholder, "");
    }

    cleaned.trim().to_string()
}
