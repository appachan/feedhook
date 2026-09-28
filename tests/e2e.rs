//! Posts to a real Discord webhook. Run with `FEEDHOOK_E2E_WEBHOOK=<url> cargo test --test e2e -- --ignored`.

mod common;

use common::{atom, new_entry, old_entry, run_feedhook, serve_feed, write_config};
use wiremock::MockServer;

#[tokio::test]
#[ignore = "posts to a real Discord webhook"]
async fn posts_new_entry_to_discord() {
    // An unset secret in GitHub Actions becomes an empty string.
    let webhook = std::env::var("FEEDHOOK_E2E_WEBHOOK")
        .ok()
        .filter(|url| !url.is_empty())
        .expect("FEEDHOOK_E2E_WEBHOOK is not set");
    let server = MockServer::start().await;
    // The username makes the posts recognizable as e2e ones in Discord.
    let config = write_config(
        "e2e",
        &format!("{}/feed.xml", server.uri()),
        &webhook,
        "feedhook e2e",
    );

    serve_feed(&server, atom(&[old_entry()])).await;
    run_feedhook(&config);

    server.reset().await;
    serve_feed(&server, atom(&[old_entry(), new_entry()])).await;
    let output = run_feedhook(&config);
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Without this, a run that posts nothing would also pass.
    assert!(stdout.contains("2 entries, 1 to post"), "{stdout}");
}
