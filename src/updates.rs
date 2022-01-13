use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use version_compare::Version;
use crate::settings::Settings;

const FEED_URL: &str = "https://gw2scratch.com/releases/arcdps-clears.json";

#[derive(Serialize, Deserialize, Clone)]
pub struct Release {
    #[serde(rename = "version")]
    version: String,
    #[serde(rename = "releaseDate")]
    release_date: String,
    #[serde(rename = "toolSiteUrl")]
    tool_site_url: String,
    #[serde(rename = "changelogUrl")]
    changelog_url: String,
}

#[allow(dead_code)]
impl Release {
    pub fn version(&self) -> &str {
        &self.version
    }
    pub fn release_date(&self) -> &str {
        &self.release_date
    }
    pub fn tool_site_url(&self) -> &str {
        &self.tool_site_url
    }
    pub fn changelog_url(&self) -> &str {
        &self.changelog_url
    }
}

#[derive(Serialize, Deserialize)]
pub struct ReleaseFeed {
    #[serde(rename = "releases")]
    releases: Vec<Release>
}

#[derive(Debug, Clone)]
pub enum UpdateError {
    NoReleaseFound
}

impl Error for UpdateError {}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "no release found in feed")
    }
}

pub fn get_update(settings: &Settings) -> Result<Option<Release>, Box<dyn Error>> {
    let release = get_latest_release()?;
    if is_ignored(settings, &release) {
        Ok(None)
    } else {
        Ok(Some(release))
    }
}

fn get_latest_release() -> Result<Release, Box<dyn Error>> {
    let feed = get_release_feed()?;
    if let Some(first) = feed.releases.into_iter().next() {
        Ok(first)
    } else {
        Err(Box::new(UpdateError::NoReleaseFound))
    }
}

fn get_release_feed() -> Result<ReleaseFeed, Box<dyn Error>> {
    let response = ureq::get(FEED_URL)
        .set("User-Agent", &format!("arcdps-clears v{}", env!("CARGO_PKG_VERSION")))
        .call()?;
    let body = response.into_string()?;
    let feed: ReleaseFeed = serde_json::from_str(&body)?;
    Ok(feed)
}

fn is_ignored(_settings: &Settings, release: &Release) -> bool {
    // We ignore versions that are not newer
    if let Some(current_version) = Version::from(env!("CARGO_PKG_VERSION")) {
        if let Some(release_version) = Version::from(&release.version) {
            if release_version <= current_version {
                return true;
            }
        }
    }
    false
}
