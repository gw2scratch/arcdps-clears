use std::thread::{JoinHandle, sleep};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::{Mutex, Arc};
use crate::{Data, Settings, friends};
use crate::api::{LiveApi, Gw2Api, ApiError};
use uuid::Uuid;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use chrono::Utc;
use crate::friends::{FriendsApiClient, FriendsApiError, State, KeyUsability};
use log::{info, warn, error};
use itertools::Itertools;

pub struct BackgroundWorkers {
    api_sender: Sender<ApiJob>,
    api_worker_handle: JoinHandle<()>,
    friends_refresher_handle: JoinHandle<()>,
    data_refresher_handle: JoinHandle<()>,
    api_worker_next_wakeup: Arc<Mutex<Instant>>,
}

impl BackgroundWorkers {
    pub fn api_refresher_next_wakeup(&self) -> &Arc<Mutex<Instant>> {
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
    UploadFriendApiSubtoken { key_hash: String },
    UpdateFriendState,
    UpdateFriendClears { account_name: String, subtoken: String },
}

pub fn start_workers(
    data_mutex: &'static Mutex<Data>,
    settings_mutex: &'static Mutex<Option<Settings>>,
    api: LiveApi,
    friends_api: FriendsApiClient,
) -> BackgroundWorkers {
    let api_next_wakeup = Arc::new(Mutex::new(Instant::now()));
    let api_next_wakeup_for_worker = api_next_wakeup.clone();
    let (api_tx, api_rx): (Sender<ApiJob>, Receiver<ApiJob>) = mpsc::channel();

    // Senders for each background thread (moved in to the thread)
    let refresher_api_tx = api_tx.clone();
    let friends_refresher_api_tx = api_tx.clone();
    let self_api_tx = api_tx.clone();

    BackgroundWorkers {
        friends_refresher_handle: thread::spawn(move || {
            loop {
                if data_mutex.lock().unwrap().friends.api_state().is_none() {
                    friends_refresher_api_tx.send(ApiJob::UpdateFriendState);
                }

                if let Some(state) = data_mutex.lock().unwrap().friends.api_state() {
                    for key_state in state.keys() {
                        let valid = key_state.subtoken_expires_at().map(|expiration| {
                            // We want to submit a new subtoken if there's less than two months remaining.
                            expiration - Utc::now() > chrono::Duration::days(60)
                        }).unwrap_or(false);

                        if !valid {
                            friends_refresher_api_tx.send(ApiJob::UploadFriendApiSubtoken { key_hash: key_state.key_hash().to_string() });
                        }
                    }

                    if state.friends().iter().any(|friend| friend.expires_at() - Utc::now() < chrono::Duration::hours(1)) {
                        // A subtoken for a friend is about to expire,
                        // the friend server already has a new one available.
                        friends_refresher_api_tx.send(ApiJob::UpdateFriendState);
                    }
                }

                sleep(Duration::from_secs(10));
            }
        }),
        api_worker_handle: thread::spawn(move || {
            loop {
                // Note that we often copy strings from settings here to avoid locking settings
                // for the duration of API requests.

                match api_rx.recv().unwrap() {
                    ApiJob::UpdateRaids => {
                        if let Ok(raids) = api.get_raids() {
                            data_mutex.lock().unwrap().clears.set_raids(Some(raids))
                        }
                    }
                    ApiJob::UpdateClears(key_uuid) => {
                        let key: Option<String> = copy_api_key(settings_mutex, key_uuid);

                        if let Some(key) = key {
                            if let Ok(state) = api.get_raids_state(&key) {
                                // TODO: Handle invalid API key explicitly
                                data_mutex.lock().unwrap().clears.set_state(key_uuid, Some(state));
                            }
                        }
                    }
                    ApiJob::UpdateAccountData(key_uuid) => {
                        let key: Option<String> = copy_api_key(settings_mutex, key_uuid);

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
                        let key: Option<String> = copy_api_key(settings_mutex, key_uuid);

                        if let Some(key) = key {
                            if let Ok(info) = api.get_token_info(&key) {
                                // TODO: Handle invalid API key explicitly
                                if let Some(key) = settings_mutex.lock().unwrap().as_mut().unwrap().get_key_mut(&key_uuid) {
                                    key.set_token_info(Some(info));
                                }
                            }
                        }
                    }
                    ApiJob::UpdateFriendState => {
                        let keys = copy_friend_capable_api_keys(settings_mutex);
                        if let Some(keys) = keys {
                            match friends_api.get_state(keys) {
                                Ok(state) => {
                                    // TODO: Deduplicate this fragment:
                                    for friend in state.friends() {
                                        // TODO: are accounts deduplicated?
                                        self_api_tx.send(ApiJob::UpdateFriendClears {
                                            account_name: friend.account().to_string(),
                                            subtoken: friend.subtoken().to_string(),
                                        });
                                    }
                                    data_mutex.lock().unwrap().friends.set_api_state(Some(state));
                                }
                                Err(FriendsApiError::UnknownError) => {
                                    // TODO: Better logging
                                    warn!("Friends - failed to get state - unknown error.")
                                }
                                Err(FriendsApiError::JsonDeserializationFailed(_)) => {
                                    warn!("Friends - failed to get state - json deserialization failed.")
                                }
                                Err(FriendsApiError::UreqError(e)) => {
                                    warn!("Friends - failed to get state: {}", e)
                                }
                            }
                        }
                    }
                    ApiJob::UploadFriendApiSubtoken { key_hash } => {
                        // We need to find a key with the specified hash and make a copy to avoid
                        // a lengthy lock on the settings mutex.
                        let matching_key = settings_mutex.lock().unwrap().as_ref()
                            .and_then(|x| x.api_keys().iter()
                                .filter(|x| friends::key_hash(x.key()) == key_hash)
                                .map(|x| x.key().to_string())
                                .next()
                            );

                        if let Some(key) = matching_key {
                            match api.create_subtoken(&key, &friends::SUBTOKEN_PERMISSIONS, &friends::SUBTOKEN_URLS, Utc::now() + chrono::Duration::days(365)) {
                                Ok(subtoken) => {
                                    if let Some(keys) = copy_friend_capable_api_keys(settings_mutex) {
                                        friends_api.add_subtoken(keys, &key, subtoken);
                                    } else {
                                        // Should not happen, failed to get keys from settings
                                        error!("Friends - Failed to get keys from settings after we generated a friend subtoken.");
                                    }
                                }
                                Err(ApiError::UnknownError) => {
                                    warn!("Friends - Failed to get subtoken for from the GW2 API - unknown error.");
                                    // TODO: Add better logging
                                }
                                Err(ApiError::InvalidKey) => {
                                    warn!("Friends - Failed to get subtoken for from the GW2 API - invalid key.");
                                }
                                Err(ApiError::JsonDeserializationFailed(e)) => {
                                    warn!("Friends - Failed to get subtoken for from the GW2 API - json deserialization failed: {}", e);
                                }
                                Err(ApiError::TooManyRequests) => {
                                    warn!("Friends - Failed to get subtoken for from the GW2 API - too many requests, rate limited.");
                                }
                            }
                        } else {
                            warn!("Friends - failed to find a key that is scheduled for a subtoken upload (was it removed in the meantime?).");
                        }
                    }
                    ApiJob::UpdateFriendClears { account_name, subtoken } => {
                        match api.get_raids_state(&subtoken) {
                            Ok(state) => {
                                data_mutex.lock().unwrap().friends.set_clears(account_name, state);
                            }
                            Err(ApiError::UnknownError) => {
                                warn!("Failed to get clears for friend {} - unknown error.", account_name);
                                // TODO: Add better logging
                            }
                            Err(ApiError::InvalidKey) => {
                                warn!("Failed to get clears for friend {} - invalid key.", account_name);
                            }
                            Err(ApiError::JsonDeserializationFailed(e)) => {
                                warn!("Failed to get clears for friend {} - json deserialization failed: {}", account_name, e);
                            }
                            Err(ApiError::TooManyRequests) => {
                                warn!("Failed to get clears for friend {} - too many requests, rate limited.", account_name);
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

                if let Some(state) = data_mutex.lock().unwrap().friends.api_state() {
                    for friend in state.friends() {
                        // TODO: are accounts deduplicated?
                        refresher_api_tx.send(ApiJob::UpdateFriendClears {
                            account_name: friend.account().to_string(),
                            subtoken: friend.subtoken().to_string(),
                        });
                    }
                }

                let sleep_duration = Duration::from_secs(60);
                *api_next_wakeup_for_worker.lock().unwrap() = Instant::now() + sleep_duration;
                sleep(sleep_duration);
            }
        }),
    }
}

fn copy_api_key(settings_mutex: &Mutex<Option<Settings>>, key_uuid: Uuid) -> Option<String> {
    // We usually use this to avoid locking the settings mutex for longer than needed.
    settings_mutex.lock().unwrap().as_ref()
        .and_then(|x| x.get_key(&key_uuid))
        .map(|x| x.key().to_string())
}

fn copy_friend_capable_api_keys(settings_mutex: &Mutex<Option<Settings>>) -> Option<Vec<String>> {
    // Get api keys that are usable with friends, ignore others.
    // Also does deduplication.

    settings_mutex.lock().unwrap().as_ref()
        .map(|x| x.api_keys().iter()
            .filter(|x| friends::get_key_usability(x).is_usable())
            .map(|x| x.key().to_string())
            .unique()
            .collect()
        )
}