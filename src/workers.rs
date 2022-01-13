use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;
use std::thread::sleep;
use std::thread;
use std::time::{Duration, Instant};

use chrono::Utc;
use itertools::Itertools;
use log::{error, warn};
use uuid::Uuid;

use crate::{Data, friends, Settings};
use crate::api::{ApiError, Gw2Api, LiveApi};
use crate::clears::RaidClearState;
use crate::friends::{FriendRequestMetadata, FriendsApiClient, FriendsApiError};

pub struct BackgroundWorkers {
    api_sender: Sender<ApiJob>,
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
    ShareKeyWithFriend { key_uuid: Uuid, friend_account_name: String },
    UnshareKeyWithFriend { key_uuid: Uuid, friend_account_name: String },
    SetKeyPublicFriend { key_uuid: Uuid, public: bool },
    SetAllKeysPublicFriend { public: bool },
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

    // Friends refresher
    thread::spawn(move || {
        let send_job = |job: ApiJob| {
            if let Err(_) = friends_refresher_api_tx.send(job) {
                warn!("Failed to send request to API worker.");
            }
        };

        loop {
            let enabled = settings_mutex.lock().unwrap().as_ref().unwrap().friends.enabled;

            if enabled {
                if data_mutex.lock().unwrap().friends.api_state().is_none() {
                    send_job(ApiJob::UpdateFriendState);
                }

                if let Some(state) = data_mutex.lock().unwrap().friends.api_state() {
                    for key_state in state.keys() {
                        let valid = key_state.subtoken_expires_at().map(|expiration| {
                            // We want to submit a new subtoken if there's less than two months remaining.
                            expiration - Utc::now() > chrono::Duration::days(60)
                        }).unwrap_or(false);

                        if !valid {
                            send_job(ApiJob::UploadFriendApiSubtoken { key_hash: key_state.key_hash().to_string() });
                        }
                    }

                    let token_about_to_expire = state.friends().iter()
                        .any(|friend| {
                            if let Some(subtoken) = friend.subtoken() {
                                subtoken.expires_at() - Utc::now() < chrono::Duration::hours(1)
                            } else {
                                false
                            }
                        });
                    if token_about_to_expire {
                        // A subtoken for a friend is about to expire,
                        // the friend server already has a new one available.
                        send_job(ApiJob::UpdateFriendState);
                    }
                }
            }

            sleep(Duration::from_secs(10));
        }
    });

    // API job consumer
    thread::spawn(move || {
        let send_job = |job: ApiJob| {
            if let Err(_) = self_api_tx.send(job) {
                warn!("Failed to send request to API worker.");
            }
        };

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
                        if let Ok(finished_encounters) = api.get_finished_encounters(&key) {
                            if let Ok(last_modified) = api.get_account_last_modified(&key) {
                                // TODO: Handle invalid API key explicitly
                                let state = RaidClearState::new(finished_encounters, Utc::now(), last_modified);
                                data_mutex.lock().unwrap().clears.set_state(key_uuid, Some(state));
                            }
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
                                // We do request a friend state update in here and not in
                                // account data as this is requested after account data
                                // and we need both.
                                send_job(ApiJob::UpdateFriendState);
                            }
                        }
                    }
                }
                ApiJob::UpdateFriendState => {
                    let enabled = settings_mutex.lock().unwrap().as_ref().unwrap().friends.enabled;

                    if enabled {
                        let metadata = copy_friends_metadata(settings_mutex);
                        if let Some(metadata) = metadata {
                            match friends_api.get_state(metadata) {
                                Ok(state) => {
                                    // TODO: Deduplicate this fragment:
                                    for friend in state.friends() {
                                        if let Some(subtoken) = friend.subtoken() {
                                            // TODO: are accounts deduplicated?
                                            send_job(ApiJob::UpdateFriendClears {
                                                account_name: friend.account().to_string(),
                                                subtoken: subtoken.subtoken().to_string(),
                                            });
                                        }
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
                                if let Some(metadata) = copy_friends_metadata(settings_mutex) {
                                    match friends_api.add_subtoken(metadata, &key, subtoken) {
                                        Ok(_) => {}
                                        Err(FriendsApiError::UnknownError) => {
                                            warn!("Friends - Failed to send subtoken to friend server.");
                                        }
                                        Err(FriendsApiError::JsonDeserializationFailed(e)) => {
                                            warn!("Friends - Failed to send subtoken to friend server - json deserialization failed: {}.", e);
                                        }
                                        Err(FriendsApiError::UreqError(e)) => {
                                            warn!("Friends - Failed to send subtoken to friend server - {}.", e);
                                        }
                                    }
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
                    match api.get_finished_encounters(&subtoken) {
                        Ok(finished_encounters) => {
                            match api.get_account_last_modified(&subtoken) {
                                Ok(last_modified) => {
                                    let state = RaidClearState::new(finished_encounters, Utc::now(), last_modified);
                                    data_mutex.lock().unwrap().friends.set_clears(account_name, state);
                                }
                                Err(err) => {
                                    warn!("Failed to get last-modified for friend {} - {:?}.", account_name, err);
                                }
                            }
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
                ApiJob::ShareKeyWithFriend { key_uuid, friend_account_name } => {
                    let metadata = copy_friends_metadata(settings_mutex);
                    let key: Option<String> = copy_api_key(settings_mutex, key_uuid);

                    if let Some(metadata) = metadata {
                        if let Some(key) = key {
                            match friends_api.share(metadata, &key, friend_account_name) {
                                Ok(state) => {
                                    data_mutex.lock().unwrap().friends.set_api_state(Some(state));
                                }
                                // TODO: deduplicate this logging
                                Err(FriendsApiError::UnknownError) => {
                                    warn!("Friends - failed to share key - unknown error.")
                                }
                                Err(FriendsApiError::JsonDeserializationFailed(_)) => {
                                    warn!("Friends - failed to share key - json deserialization failed.")
                                }
                                Err(FriendsApiError::UreqError(e)) => {
                                    warn!("Friends - failed to share key: {}", e)
                                }
                            }
                        }
                    }
                }
                ApiJob::UnshareKeyWithFriend { key_uuid, friend_account_name } => {
                    let metadata = copy_friends_metadata(settings_mutex);
                    let key: Option<String> = copy_api_key(settings_mutex, key_uuid);

                    if let Some(metadata) = metadata {
                        if let Some(key) = key {
                            match friends_api.unshare(metadata, &key, friend_account_name) {
                                Ok(state) => {
                                    data_mutex.lock().unwrap().friends.set_api_state(Some(state));
                                }
                                // TODO: deduplicate this logging
                                Err(FriendsApiError::UnknownError) => {
                                    warn!("Friends - failed to share key - unknown error.")
                                }
                                Err(FriendsApiError::JsonDeserializationFailed(_)) => {
                                    warn!("Friends - failed to share key - json deserialization failed.")
                                }
                                Err(FriendsApiError::UreqError(e)) => {
                                    warn!("Friends - failed to share key: {}", e)
                                }
                            }
                        }
                    }
                }
                ApiJob::SetKeyPublicFriend { key_uuid, public } => {
                    let metadata = copy_friends_metadata(settings_mutex);
                    let key: Option<String> = copy_api_key(settings_mutex, key_uuid);

                    if let Some(metadata) = metadata {
                        if let Some(key) = key {
                            match friends_api.set_public(metadata, &key, public) {
                                Ok(state) => {
                                    data_mutex.lock().unwrap().friends.set_api_state(Some(state));
                                }
                                // TODO: deduplicate this logging
                                Err(FriendsApiError::UnknownError) => {
                                    warn!("Friends - failed to set key public status - unknown error.")
                                }
                                Err(FriendsApiError::JsonDeserializationFailed(_)) => {
                                    warn!("Friends - failed to set key public status - json deserialization failed.")
                                }
                                Err(FriendsApiError::UreqError(e)) => {
                                    warn!("Friends - failed to set key public status: {}", e)
                                }
                            }
                        }
                    }
                }
                ApiJob::SetAllKeysPublicFriend { public } => {
                    let metadata = copy_friends_metadata(settings_mutex);

                    if let Some(metadata) = metadata {
                        for key in &metadata.api_keys {
                            match friends_api.set_public(metadata.clone(), key, public) {
                                Ok(state) => {
                                    data_mutex.lock().unwrap().friends.set_api_state(Some(state));
                                }
                                // TODO: deduplicate this logging
                                Err(FriendsApiError::UnknownError) => {
                                    warn!("Friends - failed to set key public status - unknown error.")
                                }
                                Err(FriendsApiError::JsonDeserializationFailed(_)) => {
                                    warn!("Friends - failed to set key public status - json deserialization failed.")
                                }
                                Err(FriendsApiError::UreqError(e)) => {
                                    warn!("Friends - failed to set key public status: {}", e)
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    // Clears refresher
    thread::spawn(move || {
        let send_job = |job: ApiJob| {
            if let Err(_) = refresher_api_tx.send(job) {
                warn!("Failed to send request to API worker.");
            }
        };

        loop {
            if data_mutex.lock().unwrap().clears.raids().is_none() {
                send_job(ApiJob::UpdateRaids);
            }

            if let Some(settings) = settings_mutex.lock().unwrap().as_ref() {
                for key in settings.api_keys() {
                    if key.data().account_data().is_none() {
                        send_job(ApiJob::UpdateAccountData(*key.id()));
                    }
                    if key.data().token_info().is_none() {
                        send_job(ApiJob::UpdateTokenInfo(*key.id()));
                    }
                    if key.show_key_in_clears() {
                        send_job(ApiJob::UpdateClears(*key.id()));
                    }
                }
            }

            if let Some(state) = data_mutex.lock().unwrap().friends.api_state() {
                for friend in state.friends() {
                    if let Some(subtoken) = friend.subtoken() {
                        // TODO: are accounts deduplicated?
                        send_job(ApiJob::UpdateFriendClears {
                            account_name: friend.account().to_string(),
                            subtoken: subtoken.subtoken().to_string(),
                        });
                    }
                }
            }

            let sleep_minutes = settings_mutex.lock().unwrap().as_ref().expect("Settings should be loaded by now").clears_check_interval_minutes;

            let sleep_duration = Duration::from_secs((60 * sleep_minutes) as u64);
            *api_next_wakeup_for_worker.lock().unwrap() = Instant::now() + sleep_duration;
            sleep(sleep_duration);
        }
    });

    BackgroundWorkers {
        api_worker_next_wakeup: api_next_wakeup,
        api_sender: api_tx,
    }
}

fn copy_api_key(settings_mutex: &Mutex<Option<Settings>>, key_uuid: Uuid) -> Option<String> {
    // We usually use this to avoid locking the settings mutex for longer than needed.
    settings_mutex.lock().unwrap().as_ref()
        .and_then(|x| x.get_key(&key_uuid))
        .map(|x| x.key().to_string())
}

fn copy_friends_metadata(settings_mutex: &Mutex<Option<Settings>>) -> Option<FriendRequestMetadata> {
    // Get api keys that are usable with friends, ignore others.
    // Also does deduplication.
    let api_keys = settings_mutex.lock().unwrap().as_ref()
        .map(|x| x.api_keys().iter()
            .filter(|x| friends::get_key_usability(x).is_usable())
            .map(|x| x.key().to_string())
            .unique()
            .collect()
        );

    let public_friends = settings_mutex.lock().unwrap().as_ref()
        .map(|x| x.friends.list.friends().iter()
            .map(|x| x.account_name().to_string())
            .collect()
        );

    if api_keys.is_none() || public_friends.is_none() {
        None
    } else {
        Some(FriendRequestMetadata {
            api_keys: api_keys.unwrap(),
            public_friends: public_friends.unwrap(),
        })
    }
}