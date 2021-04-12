use std::thread::{JoinHandle, sleep};
use std::thread;
use std::time::Duration;
use std::sync::Mutex;
use crate::{Data, Settings};
use std::ops::Deref;
use crate::api::{LiveApi, Gw2Api};

pub struct BackgroundWorkers {
    api_worker_handle: JoinHandle<()>,
}

pub fn start_workers(
    data_mutex: &'static Mutex<Data>,
    settings_mutex: &'static Mutex<Option<Settings>>,
    api: LiveApi,
) -> BackgroundWorkers {
    BackgroundWorkers {
        api_worker_handle: thread::spawn(move || {
            loop {
                eprintln!("api worker tick, updates");
                let mut api_key: Option<String> = None;

                if let Some(settings) = settings_mutex.lock().unwrap().deref() {
                    if let Some(main_key) = settings.main_api_key() {
                        api_key = Some(main_key.key().to_string())
                    }
                }

                // TODO: Separate api source from data and only lock data when saving it!
                if let Some(main_key) = api_key {
                    // Do not refresh raid data once we have it once
                    if data_mutex.lock().unwrap().clears.raids().is_none() {
                        if let Ok(raids) = api.get_raids() {
                            data_mutex.lock().unwrap().clears.set_raids(Some(raids))
                        }
                    }
                    if let Ok(state) = api.get_raids_state(&main_key) {
                        // TODO: Handle invalid API key somehow
                        data_mutex.lock().unwrap().clears.set_state(Some(state));
                    }
                }

                sleep(Duration::from_secs(60));
            }
        })
    }
}