use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use chrono::{DateTime, TimeDelta, Utc};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub struct TestEntry {
    pub id: &'static str,
    pub title: &'static str,
    pub published: DateTime<Utc>,
}

pub fn old_entry() -> TestEntry {
    TestEntry {
        id: "old",
        title: "Old entry",
        published: Utc::now() - TimeDelta::days(2),
    }
}

pub fn new_entry() -> TestEntry {
    TestEntry {
        id: "new",
        title: "New entry",
        published: Utc::now(),
    }
}

pub fn atom(entries: &[TestEntry]) -> String {
    let entries: String = entries
        .iter()
        .map(|entry| {
            format!(
                "<entry><id>{}</id><title>{}</title><link href=\"https://example.com/{}\"/><published>{}</published></entry>",
                entry.id,
                entry.title,
                entry.id,
                entry.published.to_rfc3339()
            )
        })
        .collect();
    format!(
        "<?xml version=\"1.0\"?><feed xmlns=\"http://www.w3.org/2005/Atom\"><title>Test feed</title>{entries}</feed>"
    )
}

pub async fn serve_feed(server: &MockServer, feed: String) {
    Mock::given(method("GET"))
        .and(path("/feed.xml"))
        .respond_with(ResponseTemplate::new(200).set_body_string(feed))
        .mount(server)
        .await;
}

/// Creates an empty directory for the test and writes `config.toml` into it.
/// `username` is shown as the author of the posts in Discord.
pub fn write_config(test_name: &str, feed_url: &str, webhook_url: &str, username: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(test_name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let config = dir.join("config.toml");
    fs::write(
        &config,
        format!(
            r#"
[channels.test]
discord_webhook = "{webhook_url}"
template = "{{title}} {{link}}"
username = "{username}"

[[feeds]]
url = "{feed_url}"
channel = "test"
"#
        ),
    )
    .unwrap();
    config
}

pub fn run_feedhook(config: &Path) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_feedhook"))
        .arg("--config")
        .arg(config)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "feedhook failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
