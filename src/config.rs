use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Relative paths are resolved against the directory of the config file.
    #[serde(default = "default_state_path")]
    pub state_path: PathBuf,
    #[serde(default = "default_max_age_hours")]
    pub max_age_hours: i64,
    pub channels: HashMap<String, Channel>,
    pub feeds: Vec<Feed>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Channel {
    pub discord_webhook: String,
    pub template: String,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Feed {
    pub url: String,
    pub channel: String,
    /// Used as `{feed_title}` instead of the feed's `<title>` (the one of the whole feed, not of an entry).
    pub title: Option<String>,
}

fn default_state_path() -> PathBuf {
    PathBuf::from("state.json")
}

fn default_max_age_hours() -> i64 {
    24
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let mut config =
            Self::parse(&text).with_context(|| format!("invalid config {}", path.display()))?;
        if let Some(dir) = path.parent() {
            config.state_path = dir.join(&config.state_path);
        }
        Ok(config)
    }

    fn parse(text: &str) -> Result<Self> {
        let config: Self = toml::from_str(text)?;
        let mut urls = HashSet::new();
        for feed in &config.feeds {
            if !config.channels.contains_key(&feed.channel) {
                bail!(
                    "feed {} refers to unknown channel {}",
                    feed.url,
                    feed.channel
                );
            }
            // The state is keyed by feed URL, so the same URL cannot be listed twice.
            if !urls.insert(&feed.url) {
                bail!("feed {} is listed more than once", feed.url);
            }
        }
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHANNELS: &str = r#"
        [channels.news]
        discord_webhook = "https://discord.com/api/webhooks/1/x"
        template = "{title}"
    "#;

    #[test]
    fn parses_config_with_defaults() {
        let text = format!(
            "{CHANNELS}\n[[feeds]]\nurl = \"https://example.com/feed\"\nchannel = \"news\"\n"
        );
        let config = Config::parse(&text).unwrap();
        assert_eq!(config.state_path, PathBuf::from("state.json"));
        assert_eq!(config.max_age_hours, 24);
        assert_eq!(config.feeds.len(), 1);
        assert_eq!(config.channels["news"].username, None);
        assert_eq!(config.feeds[0].title, None);
    }

    #[test]
    fn rejects_unknown_channel() {
        let text = format!(
            "{CHANNELS}\n[[feeds]]\nurl = \"https://example.com/feed\"\nchannel = \"alerts\"\n"
        );
        let err = Config::parse(&text).unwrap_err();
        assert!(err.to_string().contains("unknown channel alerts"), "{err}");
    }

    #[test]
    fn rejects_duplicate_feed_url() {
        let feed = "\n[[feeds]]\nurl = \"https://example.com/feed\"\nchannel = \"news\"\n";
        let text = format!("{CHANNELS}{feed}{feed}");
        let err = Config::parse(&text).unwrap_err();
        assert!(err.to_string().contains("more than once"), "{err}");
    }
}
