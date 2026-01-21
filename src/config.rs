use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use crate::util::expand_path;

#[derive(Deserialize)]
pub struct Config {
    pub window: Option<WindowConfig>,
    pub style: Option<StyleConfig>,
}

#[derive(Deserialize)]
pub struct WindowConfig {
    pub width: i32,
    pub height: i32,
}

#[derive(Deserialize)]
pub struct StyleConfig {
    #[serde(deserialize_with = "deserialize_expanded_path")]
    pub path: Option<PathBuf>,
    pub opacity: Option<f64>,
}

fn deserialize_expanded_path<'de, D>(deserializer: D) -> Result<Option<PathBuf>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt_path: Option<String> = Option::deserialize(deserializer)?;
    if let Some(path) = opt_path {
        let expanded = expand_path(&path).map_err(serde::de::Error::custom)?;
        Ok(Some(expanded))
    } else {
        Ok(None)
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            window: Some(WindowConfig::default()),
            style: Some(StyleConfig::default()),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> WindowConfig {
        WindowConfig {
            width: 800,
            height: 400,
        }
    }
}

impl Default for StyleConfig {
    fn default() -> StyleConfig {
        StyleConfig {
            path: None,
            opacity: Some(1.0),
        }
    }
}

pub fn load_from_file(path: &Option<PathBuf>) -> Config {
    if path.is_none() {
        return Config::default();
    }

    let path = path.as_ref().unwrap();
    let contents = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Could not read config file!");
            std::process::exit(1)
        }
    };

    let data: Config = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(_) => {
            eprintln!("Unable to load config file!");
            std::process::exit(1)
        }
    };

    return data;
}
