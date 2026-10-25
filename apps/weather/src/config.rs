//! `$KIOSK_DATA/config.toml`: where the weather is for. A missing file is
//! created with the defaults; a broken one falls back to them with a note.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
}

impl Default for Config {
    fn default() -> Self {
        Config { name: "Meiendorf, Wandsbek, Hamburg".into(), latitude: 53.65, longitude: 10.15 }
    }
}

pub fn data_dir() -> PathBuf {
    std::env::var_os("KIOSK_DATA").map_or_else(|| PathBuf::from("."), PathBuf::from)
}

/// The config, and a note for the screen when it could not be used as written.
pub fn load(dir: &Path) -> (Config, Option<String>) {
    let path = dir.join("config.toml");
    match std::fs::read_to_string(&path) {
        Ok(text) => match toml::from_str::<Config>(&text) {
            Ok(c) if (-90.0..=90.0).contains(&c.latitude) && (-180.0..=180.0).contains(&c.longitude) => (c, None),
            Ok(_) => (Config::default(), Some("config.toml: Koordinaten ungültig".into())),
            Err(_) => (Config::default(), Some("config.toml: nicht lesbar".into())),
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let c = Config::default();
            if let Ok(text) = toml::to_string_pretty(&c) {
                let _ = std::fs::write(&path, text);
            }
            (c, None)
        }
        Err(_) => (Config::default(), Some("config.toml: nicht lesbar".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("weather-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn missing_file_is_created_with_defaults() {
        let d = dir("missing");
        let (c, note) = load(&d);
        assert_eq!(c, Config::default());
        assert!(note.is_none());
        assert_eq!(load(&d).0, c, "the written file reads back");
    }

    #[test]
    fn reads_a_custom_location() {
        let d = dir("custom");
        std::fs::write(d.join("config.toml"), "name = \"Berlin\"\nlatitude = 52.52\nlongitude = 13.4\n").unwrap();
        let (c, note) = load(&d);
        assert_eq!((c.name.as_str(), c.latitude, c.longitude), ("Berlin", 52.52, 13.4));
        assert!(note.is_none());
    }

    #[test]
    fn broken_or_out_of_range_falls_back_with_a_note() {
        let d = dir("broken");
        std::fs::write(d.join("config.toml"), "latitude = [").unwrap();
        let (c, note) = load(&d);
        assert_eq!(c, Config::default());
        assert!(note.is_some());
        std::fs::write(d.join("config.toml"), "latitude = 123.0\n").unwrap();
        assert!(load(&d).1.is_some());
    }
}
