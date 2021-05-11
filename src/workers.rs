use std::thread::{JoinHandle, sleep};
use std::thread;
use std::time::{Duration, SystemTime, Instant};
use std::sync::{Mutex, Arc};
use crate::{Data, Settings};
use std::ops::Deref;
use crate::api::{LiveApi, Gw2Api};
use crate::translations::Translation;

pub struct BackgroundWorkers {
    api_worker_handle: JoinHandle<()>,
    api_worker_next_wakeup: Arc<Mutex<Instant>>
}

impl BackgroundWorkers {
    pub fn api_worker_next_wakeup(&self) -> &Arc<Mutex<Instant>> {
        &self.api_worker_next_wakeup
    }
}

pub fn start_workers(
    data_mutex: &'static Mutex<Data>,
    settings_mutex: &'static Mutex<Option<Settings>>,
    api: LiveApi,
) -> BackgroundWorkers {
    let api_next_wakeup = Arc::new(Mutex::new(Instant::now()));
    let api_next_wakeup_for_worker = api_next_wakeup.clone();
    BackgroundWorkers {
        api_worker_handle: thread::spawn(move || {
            loop {
                let mut api_key: Option<String> = None;

                if let Some(settings) = settings_mutex.lock().unwrap().deref() {
                    if let Some(main_key) = settings.main_api_key() {
                        api_key = Some(main_key.key().to_string())
                    }
                }

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

                let sleep_duration = Duration::from_secs(60);
                *api_next_wakeup_for_worker.lock().unwrap() = Instant::now() + sleep_duration;
                sleep(sleep_duration);
            }
        }),
        api_worker_next_wakeup: api_next_wakeup
    }
}