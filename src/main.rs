mod config;
mod discord;
mod feed;
mod state;

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Result, ensure};
use chrono::{DateTime, TimeDelta, Utc};
use clap::Parser;

use config::Config;
use state::State;

/// Posts new entries of RSS / Atom feeds to Discord webhooks.
#[derive(Parser)]
#[command(version)]
struct Args {
    #[arg(long, default_value = "config.toml")]
    config: PathBuf,
    /// Print the messages instead of posting them, and do not update the state.
    #[arg(long)]
    dry_run: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::load(&args.config)?;
    let mut state = state::load(&config.state_path)?;
    // Forget feeds removed from the config, so that re-adding one is treated as a first run.
    state.retain(|url, _| config.feeds.iter().any(|feed| feed.url == *url));
    let client = reqwest::Client::builder()
        .user_agent(concat!("feedhook/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(30))
        .build()?;
    let cutoff = Utc::now() - TimeDelta::hours(config.max_age_hours);

    // Keep going on failure so that one broken feed does not block the others.
    let mut failures = 0;
    for feed in &config.feeds {
        if let Err(err) =
            process_feed(&client, &config, feed, &mut state, cutoff, args.dry_run).await
        {
            eprintln!("{}: {err:#}", feed.url);
            failures += 1;
        }
    }
    ensure!(
        failures == 0,
        "{failures} of {} feeds failed",
        config.feeds.len()
    );
    Ok(())
}

async fn process_feed(
    client: &reqwest::Client,
    config: &Config,
    feed: &config::Feed,
    state: &mut State,
    cutoff: DateTime<Utc>,
    dry_run: bool,
) -> Result<()> {
    let channel = &config.channels[&feed.channel];
    let fetched = feed::fetch(client, &feed.url).await?;
    let to_post = feed::entries_to_post(&fetched.entries, state.get(&feed.url), cutoff);
    println!(
        "{}: {} entries, {} to post",
        feed.url,
        fetched.entries.len(),
        to_post.len()
    );

    let mut failed = BTreeSet::new();
    for entry in to_post {
        let content = discord::render(&channel.template, &fetched.title, entry);
        if dry_run {
            println!("--- #{}\n{content}", feed.channel);
        } else if let Err(err) = discord::post(client, channel, &content).await {
            // Leave it unseen so that it is retried on the next run.
            eprintln!("{}: failed to post {}: {err:#}", feed.url, entry.link);
            failed.insert(&entry.id);
        }
    }
    if dry_run {
        return Ok(());
    }

    // Replace rather than extend, so that IDs gone from the feed do not pile up.
    let seen = fetched
        .entries
        .iter()
        .filter(|entry| !failed.contains(&entry.id))
        .map(|entry| entry.id.clone())
        .collect();
    state.insert(feed.url.clone(), seen);
    state::save(&config.state_path, state)?;
    ensure!(failed.is_empty(), "failed to post {} entries", failed.len());
    Ok(())
}
