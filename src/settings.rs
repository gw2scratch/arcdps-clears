use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Read, BufWriter, Write};
use std::error::Error;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    main_api_key: Option<ApiKey>,
    #[serde(default = "default_short_name")]
    pub short_names: bool,
}

fn default_short_name() -> bool {
    true
}

#[derive(Serialize, Deserialize)]
pub struct ApiKey {
    key: String
}

impl ApiKey {
    pub fn new(str: &str) -> Self {
        ApiKey {
            key: str.to_string()
        }
    }
    pub fn key(&self) -> &str {
        &self.key
    }
}

impl Settings {
    fn default() -> Self {
        Settings {
            main_api_key: None,
            short_names: true
        }
    }

    pub fn main_api_key(&self) -> &Option<ApiKey> {
        &self.main_api_key
    }

    pub fn short_names(&self) -> bool {
        self.short_names
    }

    pub fn set_main_api_key(&mut self, main_api_key: Option<ApiKey>) {
        self.main_api_key = main_api_key;
    }

    pub fn load_from_file(filename: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let settings: Settings = serde_json::from_reader(reader)?;

        Ok(settings)
    }

    #[must_use]
    pub fn save_to_file(&self, filename: &str) -> Result<(), Box<dyn Error>> {
        // We first serialize settings into a temporary file and then move the file
        let tmp_filename = format!("{}.tmp", filename);
        let tmp_file = File::create(&tmp_filename)?;
        let mut writer = BufWriter::new(tmp_file);
        serde_json::to_writer(&mut writer, &self)?;
        writer.flush()?;

        std::fs::rename(tmp_filename, filename)?;
        Ok(())
    }
}

pub fn load_bg(settings_mutex: &'static Mutex<Option<Settings>>, filename: &'static str, continue_with: Option<fn()>) {
    std::thread::spawn(move || {
        if let Ok(settings) = Settings::load_from_file(filename) {
            *settings_mutex.lock().unwrap() = Some(settings);
        } else {
            // TODO: Log failure (unless file doesn't exist) and reset settings
            *settings_mutex.lock().unwrap() = Some(Settings::default());
        }

        if let Some(function) = continue_with {
            function();
        }
    });
}