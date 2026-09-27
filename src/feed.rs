use std::collections::BTreeSet;

use anyhow::Result;
use chrono::{DateTime, Utc};

pub struct Fetched {
    pub title: String,
    pub entries: Vec<Entry>,
}

pub struct Entry {
    pub id: String,
    pub title: String,
    pub link: String,
    /// Plain text converted from the summary (or the content) HTML.
    pub description: String,
    /// The published date, or the updated date if the entry has none.
    pub date: Option<DateTime<Utc>>,
}

pub async fn fetch(client: &reqwest::Client, url: &str) -> Result<Fetched> {
    let body = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    parse(url, &body)
}

fn parse(url: &str, body: &[u8]) -> Result<Fetched> {
    let parser = feed_rs::parser::Builder::new()
        .base_uri(Some(url))
        .id_generator(|links, title, uri| {
            // With base_uri set, feed-rs falls back to a random UUID only when there is neither
            // a link nor a title. Such an ID would change on every run, so leave it empty
            // instead to skip the entry.
            if links.is_empty() && title.is_none() {
                String::new()
            } else {
                feed_rs::parser::generate_id(links, title, uri)
            }
        })
        .build();
    let feed = parser.parse(body)?;

    let entries = feed
        .entries
        .into_iter()
        .filter(|entry| !entry.id.is_empty())
        .map(|entry| {
            let html = entry
                .summary
                .map(|summary| summary.content)
                .or_else(|| entry.content.and_then(|content| content.body))
                .unwrap_or_default();
            Entry {
                id: entry.id,
                title: entry.title.map(|title| title.content).unwrap_or_default(),
                link: article_link(&entry.links),
                description: html_to_text(&html),
                date: entry.published.or(entry.updated),
            }
        })
        .collect();

    Ok(Fetched {
        title: feed.title.map(|title| title.content).unwrap_or_default(),
        entries,
    })
}

/// Picks the link to the article itself.
///
/// Atom entries may also have links such as `replies` or `edit` before it. feed-rs sets the rel
/// of an Atom link without one to `alternate`, while RSS links have no rel.
fn article_link(links: &[feed_rs::model::Link]) -> String {
    links
        .iter()
        .find(|link| link.rel.as_deref().is_none_or(|rel| rel == "alternate"))
        .or(links.first())
        .map(|link| link.href.clone())
        .unwrap_or_default()
}

fn html_to_text(html: &str) -> String {
    // Discord wraps lines by itself, so render without wrapping.
    html2text::config::plain_no_decorate()
        .string_from_read(html.as_bytes(), usize::MAX)
        .map(|text| text.trim().to_string())
        .unwrap_or_else(|_| html.to_string())
}

/// Returns the entries to post, oldest first.
///
/// `seen` is `None` when the feed has never been processed; in that case nothing is posted
/// so that old entries are not flooded into the channel.
pub fn entries_to_post<'a>(
    entries: &'a [Entry],
    seen: Option<&BTreeSet<String>>,
    cutoff: DateTime<Utc>,
) -> Vec<&'a Entry> {
    let Some(seen) = seen else {
        return Vec::new();
    };
    let mut new_entries: Vec<&Entry> = entries
        .iter()
        .filter(|entry| !seen.contains(&entry.id))
        .filter(|entry| entry.date.is_some_and(|date| date >= cutoff))
        .collect();
    new_entries.sort_by_key(|entry| entry.date);
    new_entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 9, 27, hour, 0, 0).unwrap()
    }

    mod entries_to_post {
        use super::*;

        fn entry(id: &str, date: Option<DateTime<Utc>>) -> Entry {
            Entry {
                id: id.to_string(),
                title: String::new(),
                link: String::new(),
                description: String::new(),
                date,
            }
        }

        fn ids(entries: Vec<&Entry>) -> Vec<&str> {
            entries.iter().map(|entry| entry.id.as_str()).collect()
        }

        #[test]
        fn posts_nothing_on_first_run() {
            let entries = vec![entry("a", Some(at(12)))];
            assert!(entries_to_post(&entries, None, at(0)).is_empty());
        }

        #[test]
        fn posts_unseen_recent_entries_oldest_first() {
            let entries = vec![
                entry("newer", Some(at(12))),
                entry("seen", Some(at(11))),
                entry("older", Some(at(10))),
            ];
            let seen = BTreeSet::from(["seen".to_string()]);
            assert_eq!(
                ids(entries_to_post(&entries, Some(&seen), at(0))),
                ["older", "newer"]
            );
        }

        #[test]
        fn skips_entries_older_than_cutoff_or_without_date() {
            let entries = vec![
                entry("old", Some(at(5))),
                entry("undated", None),
                entry("recent", Some(at(6))),
            ];
            assert_eq!(
                ids(entries_to_post(&entries, Some(&BTreeSet::new()), at(6))),
                ["recent"]
            );
        }
    }

    mod parse {
        use super::*;

        #[test]
        fn parses_rss_and_converts_description_to_text() {
            let rss = r#"<?xml version="1.0"?>
            <rss version="2.0"><channel><title>Status</title>
              <item>
                <title>Incident</title>
                <link>https://example.com/incidents/1</link>
                <guid>incident-1</guid>
                <pubDate>Sun, 27 Sep 2026 12:00:00 GMT</pubDate>
                <description>&lt;p&gt;Investigating &lt;b&gt;elevated&lt;/b&gt; errors.&lt;/p&gt;</description>
              </item>
            </channel></rss>"#;
            let fetched = parse("https://example.com/rss", rss.as_bytes()).unwrap();
            assert_eq!(fetched.title, "Status");
            let entry = &fetched.entries[0];
            assert_eq!(entry.id, "incident-1");
            assert_eq!(entry.link, "https://example.com/incidents/1");
            assert_eq!(entry.description, "Investigating elevated errors.");
            assert_eq!(entry.date, Some(at(12)));
        }

        #[test]
        fn prefers_alternate_link_in_atom() {
            let atom = r#"<?xml version="1.0"?>
            <feed xmlns="http://www.w3.org/2005/Atom"><title>Blog</title><id>blog</id>
              <entry>
                <id>post-1</id><title>Post</title>
                <link rel="replies" href="https://example.com/posts/1/comments"/>
                <link rel="edit" href="https://example.com/api/posts/1"/>
                <link rel="alternate" href="https://example.com/posts/1"/>
              </entry>
              <entry>
                <id>post-2</id><title>Post without rel</title>
                <link rel="replies" href="https://example.com/posts/2/comments"/>
                <link href="https://example.com/posts/2"/>
              </entry>
            </feed>"#;
            let fetched = parse("https://example.com/atom", atom.as_bytes()).unwrap();
            assert_eq!(fetched.entries[0].link, "https://example.com/posts/1");
            assert_eq!(fetched.entries[1].link, "https://example.com/posts/2");
        }

        #[test]
        fn generates_stable_ids_and_skips_entries_without_link_and_title() {
            let rss = r#"<?xml version="1.0"?>
            <rss version="2.0"><channel><title>Blog</title>
              <item><title>No guid</title><link>https://example.com/posts/1</link></item>
              <item><description>Neither link nor title</description></item>
            </channel></rss>"#;
            let first = parse("https://example.com/rss", rss.as_bytes()).unwrap();
            let second = parse("https://example.com/rss", rss.as_bytes()).unwrap();
            assert_eq!(first.entries.len(), 1);
            assert!(!first.entries[0].id.is_empty());
            assert_eq!(first.entries[0].id, second.entries[0].id);
        }
    }
}
