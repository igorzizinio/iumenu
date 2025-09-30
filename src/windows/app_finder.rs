use std::collections::HashMap;
use std::path::PathBuf;
use crate::app::DesktopApp;

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

/// Get all available Windows applications
#[cfg(windows)]
pub fn get_available_apps() -> HashMap<String, DesktopApp> {
    let mut apps = HashMap::new();

    apps.extend(get_registry_apps().into_iter());
    apps.extend(get_start_menu_apps().into_iter());
    apps.extend(get_user_start_menu_apps().into_iter());

    apps
}

/// Get applications from Windows Registry (Uninstall entries)
#[cfg(windows)]
fn get_registry_apps() -> HashMap<String, DesktopApp> {
    let mut apps = HashMap::new();

    // Check HKEY_LOCAL_MACHINE
    if let Ok(hklm) = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall")
    {
        for subkey_name in hklm.enum_keys().filter_map(|k| k.ok()) {
            if let Ok(app) = parse_registry_app(&hklm, &subkey_name) {
                apps.insert(app.id.clone(), app);
            }
        }
    }

    // Check HKEY_CURRENT_USER
    if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall")
    {
        for subkey_name in hkcu.enum_keys().filter_map(|k| k.ok()) {
            if let Ok(app) = parse_registry_app(&hkcu, &subkey_name) {
                apps.insert(app.id.clone(), app);
            }
        }
    }

    apps
}

/// Parse a registry entry into a DesktopApp
#[cfg(windows)]
fn parse_registry_app(parent_key: &RegKey, subkey_name: &str) -> Result<DesktopApp, Box<dyn std::error::Error>> {
    let subkey = parent_key.open_subkey(subkey_name)?;

    let display_name: String = subkey.get_value("DisplayName")?;
    let display_icon: String = subkey.get_value("DisplayIcon").unwrap_or_default();
    let install_location: String = subkey.get_value("InstallLocation").unwrap_or_default();
    let publisher: String = subkey.get_value("Publisher").unwrap_or_default();
    
    // Try to get the executable path from various registry keys
    let exec = if !display_icon.is_empty() && display_icon.to_lowercase().ends_with(".exe") {
        // DisplayIcon sometimes contains the exe path
        clean_windows_path(&display_icon)
    } else if let Ok(uninstall_string) = subkey.get_value::<String, _>("UninstallString") {
        // Extract exe from uninstall string
        clean_windows_path(&uninstall_string)
    } else if let Ok(display_icon_val) = subkey.get_value::<String, _>("DisplayIcon") {
        // Try DisplayIcon as fallback
        clean_windows_path(&display_icon_val)
    } else if !install_location.is_empty() {
        install_location.clone()
    } else {
        String::new()
    };

    // Skip entries without a display name or executable
    if display_name.is_empty() || exec.is_empty() {
        return Err("Missing required fields".into());
    }

    // Extract icon path from DisplayIcon
    let icon = if !display_icon.is_empty() {
        clean_icon_path(&display_icon)
    } else {
        String::new()
    };

    Ok(DesktopApp::new(
        subkey_name.to_string(),
        display_name,
        publisher, // Using publisher as comment
        exec,
        String::new(), // keywords
        "Application".to_string(),
        String::new(), // categories
        icon,
        false, // no_display
        String::new(), // only_show_in
    ))
}

/// Get applications from Start Menu (all users)
#[cfg(windows)]
fn get_start_menu_apps() -> HashMap<String, DesktopApp> {
    let mut apps = HashMap::new();
    
    if let Ok(program_data) = std::env::var("ProgramData") {
        let start_menu = PathBuf::from(program_data)
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs");
        
        apps.extend(scan_lnk_files(&start_menu).into_iter());
    }

    apps
}

/// Get applications from User Start Menu
#[cfg(windows)]
fn get_user_start_menu_apps() -> HashMap<String, DesktopApp> {
    let mut apps = HashMap::new();
    
    if let Ok(appdata) = std::env::var("APPDATA") {
        let start_menu = PathBuf::from(appdata)
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs");
        
        apps.extend(scan_lnk_files(&start_menu).into_iter());
    }

    apps
}

/// Recursively scan directory for .lnk files
#[cfg(windows)]
fn scan_lnk_files(path: &PathBuf) -> HashMap<String, DesktopApp> {
    let mut apps = HashMap::new();

    if !path.exists() || !path.is_dir() {
        return apps;
    }

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            
            if entry_path.is_dir() {
                // Recursively scan subdirectories
                apps.extend(scan_lnk_files(&entry_path).into_iter());
            } else if entry_path.extension().and_then(|s| s.to_str()) == Some("lnk") {
                if let Some(app) = parse_lnk_file(&entry_path) {
                    apps.insert(app.id.clone(), app);
                }
            }
        }
    }

    apps
}

/// Parse a .lnk shortcut file into a DesktopApp
#[cfg(windows)]
fn parse_lnk_file(path: &PathBuf) -> Option<DesktopApp> {
    use lnk::ShellLink;

    let lnk = ShellLink::open(path).ok()?;
    
    let name = path.file_stem()?.to_string_lossy().to_string();
    let exec = lnk.link_info()
        .and_then(|li| li.local_base_path())
        .unwrap_or_default()
        .to_string();
    
    if exec.is_empty() {
        return None;
    }

    let icon = lnk.icon_location().unwrap_or_default().to_string();
    let description = lnk.name().unwrap_or_default().to_string();
    let working_dir = lnk.working_dir().unwrap_or_default().to_string();

    Some(DesktopApp::new(
        path.to_string_lossy().to_string(),
        name,
        description,
        exec,
        String::new(), // keywords
        "Application".to_string(),
        String::new(), // categories
        icon,
        false, // no_display
        String::new(), // only_show_in
    ))
}

/// Clean up Windows path strings (remove quotes, extract exe path)
#[cfg(windows)]
fn clean_windows_path(path: &str) -> String {
    let path = path.trim();
    
    // Remove quotes
    let path = path.trim_matches('"');
    
    // If it contains arguments, try to extract just the exe path
    if let Some(exe_end) = path.find(".exe") {
        path[..exe_end + 4].to_string()
    } else {
        path.to_string()
    }
}

/// Extract icon path from DisplayIcon registry value
#[cfg(windows)]
fn clean_icon_path(icon: &str) -> String {
    let icon = icon.trim().trim_matches('"');
    
    // DisplayIcon format can be "path.exe,0" or just "path.ico"
    if let Some(comma_pos) = icon.find(',') {
        icon[..comma_pos].to_string()
    } else {
        icon.to_string()
    }
}

#[cfg(not(windows))]
pub fn get_available_apps() -> HashMap<String, DesktopApp> {
    HashMap::new()
}
