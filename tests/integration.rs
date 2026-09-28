mod common;

use common::{atom, new_entry, old_entry, run_feedhook, serve_feed, write_config};
use serde_json::{Value, json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn accept_posts(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(204))
        .mount(server)
        .await;
}

async fn posts(server: &MockServer) -> Vec<Value> {
    server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|request| request.url.path() == "/webhook")
        .map(|request| request.body_json().unwrap())
        .collect()
}

#[tokio::test]
async fn posts_only_entries_added_after_the_first_run() {
    let server = MockServer::start().await;
    let config = write_config(
        "posts_only_new",
        &format!("{}/feed.xml", server.uri()),
        &format!("{}/webhook", server.uri()),
    );

    serve_feed(&server, atom(&[old_entry()])).await;
    accept_posts(&server).await;
    run_feedhook(&config);
    assert!(posts(&server).await.is_empty());

    server.reset().await;
    serve_feed(&server, atom(&[old_entry(), new_entry()])).await;
    accept_posts(&server).await;
    run_feedhook(&config);
    assert_eq!(
        posts(&server).await,
        [json!({
            "content": "New entry https://example.com/new",
            "allowed_mentions": { "parse": [] },
        })]
    );
}

#[tokio::test]
async fn retries_after_rate_limit() {
    let server = MockServer::start().await;
    let config = write_config(
        "retries",
        &format!("{}/feed.xml", server.uri()),
        &format!("{}/webhook", server.uri()),
    );
    serve_feed(&server, atom(&[old_entry()])).await;
    run_feedhook(&config);

    server.reset().await;
    serve_feed(&server, atom(&[old_entry(), new_entry()])).await;
    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(429).set_body_json(json!({ "retry_after": 0.01 })))
        .up_to_n_times(1)
        .with_priority(1)
        .mount(&server)
        .await;
    accept_posts(&server).await;
    run_feedhook(&config);
    assert_eq!(posts(&server).await.len(), 2);
}
