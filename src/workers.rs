use std::thread::{JoinHandle, sleep};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::{Mutex, Arc};
use crate::{Data, Settings};
use crate::api::{LiveApi, Gw2Api};
use uuid::Uuid;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

pub struct BackgroundWorkers {
    api_sender: Sender<ApiJob>,
    api_worker_handle: JoinHandle<()>,
    data_refresher_handle: JoinHandle<()>,
    api_worker_next_wakeup: Arc<Mutex<Instant>>,
}

impl BackgroundWorkers {
    pub fn api_worker_next_wakeup(&self) -> &Arc<Mutex<Instant>> {
        &self.api_worker_next_wakeup
    }

    pub fn api_sender(&self) -> Sender<ApiJob> {
        self.api_sender.clone()
    }
}

pub enum ApiJob {
    UpdateRaids,
    UpdateClears(Uuid),
    UpdateAccountData(Uuid),
    UpdateTokenInfo(Uuid),
}

pub fn start_workers(
    data_mutex: &'static Mutex<Data>,
    settings_mutex: &'static Mutex<Option<Settings>>,
    api: LiveApi,
) -> BackgroundWorkers {
    let api_next_wakeup = Arc::new(Mutex::new(Instant::now()));
    let api_next_wakeup_for_worker = api_next_wakeup.clone();
    let (api_tx, api_rx): (Sender<ApiJob>, Receiver<ApiJob>) = mpsc::channel();
    let refresher_api_tx = api_tx.clone();
    BackgroundWorkers {
        api_worker_handle: thread::spawn(move || {
            loop {
                match api_rx.recv().unwrap() {
                    ApiJob::UpdateRaids => {
                        if let Ok(raids) = api.get_raids() {
                            data_mutex.lock().unwrap().clears.set_raids(Some(raids))
                        }
                    }
                    ApiJob::UpdateClears(key_uuid) => {
                        // We copy the key to a String here to avoid locking settings
                        // for the duration of the API request.
                        let key: Option<String> = settings_mutex.lock().unwrap().as_ref()
                            .and_then(|x| x.get_key(&key_uuid))
                            .map(|x| x.key().to_string());

                        if let Some(key) = key {
                            if let Ok(state) = api.get_raids_state(&key) {
                                // TODO: Handle invalid API key explicitly
                                data_mutex.lock().unwrap().clears.set_state(key_uuid, Some(state));
                            }
                        }
                    }
                    ApiJob::UpdateAccountData(key_uuid) => {
                        // We copy the key to a String here to avoid locking settings
                        // for the duration of the API request.
                        let key: Option<String> = settings_mutex.lock().unwrap().as_ref()
                            .and_then(|x| x.get_key(&key_uuid))
                            .map(|x| x.key().to_string());

                        if let Some(key) = key {
                            if let Ok(data) = api.get_account_data(&key) {
                                // TODO: Handle invalid API key explicitly
                                if let Some(key) = settings_mutex.lock().unwrap().as_mut().unwrap().get_key_mut(&key_uuid) {
                                    key.set_account_data(Some(data));
                                }
                            }
                        }
                    }
                    ApiJob::UpdateTokenInfo(key_uuid) => {
                        // We copy the key to a String here to avoid locking settings
                        // for the duration of the API request.
                        let key: Option<String> = settings_mutex.lock().unwrap().as_ref()
                            .and_then(|x| x.get_key(&key_uuid))
                            .map(|x| x.key().to_string());

                        if let Some(key) = key {
                            if let Ok(info) = api.get_token_info(&key) {
                                // TODO: Handle invalid API key explicitly
                                if let Some(key) = settings_mutex.lock().unwrap().as_mut().unwrap().get_key_mut(&key_uuid) {
                                    key.set_token_info(Some(info));
                                }
                            }
                        }
                    }
                }
            }
        }),
        api_worker_next_wakeup: api_next_wakeup,
        api_sender: api_tx,
        data_refresher_handle: thread::spawn(move || {
            loop {
                if data_mutex.lock().unwrap().clears.raids().is_none() {
                    refresher_api_tx.send(ApiJob::UpdateRaids);
                }

                if let Some(settings) = settings_mutex.lock().unwrap().as_ref() {
                    for key in settings.api_keys() {
                        if key.data().account_data().is_none() {
                            refresher_api_tx.send(ApiJob::UpdateAccountData(*key.id()));
                        }
                        if key.data().token_info().is_none() {
                            refresher_api_tx.send(ApiJob::UpdateTokenInfo(*key.id()));
                        }
                        if key.show_key_in_clears() {
                            refresher_api_tx.send(ApiJob::UpdateClears(*key.id()));
                        }
                    }
                }

                let sleep_duration = Duration::from_secs(60);
                *api_next_wakeup_for_worker.lock().unwrap() = Instant::now() + sleep_duration;
                sleep(sleep_duration);
            }
        }),
    }
}