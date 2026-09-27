use std::time::Duration;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use crate::config::Channel;
use crate::feed::Entry;

const MAX_CONTENT_CHARS: usize = 2000;
const MAX_ATTEMPTS: usize = 3;

/// Expands `{title}`, `{link}`, `{description}` and `{feed_title}` in the template.
/// `{description}` is the entry's HTML converted to plain text.
///
/// The template is scanned once, so placeholders contained in the values are left as they are.
pub fn render(template: &str, feed_title: &str, entry: &Entry) -> String {
    let description = html_to_text(&entry.description_html);
    let lookup = |key: &str| match key {
        "title" => Some(entry.title.as_str()),
        "link" => Some(entry.link.as_str()),
        "description" => Some(description.as_str()),
        "feed_title" => Some(feed_title),
        _ => None,
    };

    let mut out = String::new();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        rest = &rest[start..];
        let placeholder = rest
            .find('}')
            .and_then(|end| lookup(&rest[1..end]).map(|value| (end, value)));
        match placeholder {
            Some((end, value)) => {
                out.push_str(value);
                rest = &rest[end + 1..];
            }
            None => {
                out.push('{');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    truncate(out, MAX_CONTENT_CHARS)
}

fn html_to_text(html: &str) -> String {
    // Discord wraps lines by itself, so render without wrapping. If html2text cannot render the
    // HTML, say so in the message instead of posting raw HTML.
    html2text::config::plain_no_decorate()
        .string_from_read(html.as_bytes(), usize::MAX)
        .map(|text| text.trim().to_string())
        .unwrap_or_else(|err| {
            format!("(feedhook: failed to convert the description to text: {err})")
        })
}

fn truncate(text: String, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text;
    }
    let mut truncated: String = text.chars().take(max_chars - 1).collect();
    truncated.push('…');
    truncated
}

#[derive(Serialize)]
struct Payload<'a> {
    content: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<&'a str>,
    allowed_mentions: AllowedMentions,
}

/// An empty `parse` disables all mentions, e.g. `@everyone` in an entry.
#[derive(Serialize)]
struct AllowedMentions {
    parse: [&'static str; 0],
}

#[derive(Deserialize)]
struct RateLimited {
    retry_after: f64,
}

pub async fn post(client: &reqwest::Client, channel: &Channel, content: &str) -> Result<()> {
    let payload = Payload {
        content,
        username: channel.username.as_deref(),
        avatar_url: channel.avatar_url.as_deref(),
        allowed_mentions: AllowedMentions { parse: [] },
    };
    for _ in 0..MAX_ATTEMPTS {
        // The webhook URL is a secret, so keep it out of error messages.
        let response = client
            .post(&channel.discord_webhook)
            .json(&payload)
            .send()
            .await
            .map_err(|err| err.without_url())?;
        let status = response.status();
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let RateLimited { retry_after } =
                response.json().await.map_err(|err| err.without_url())?;
            tokio::time::sleep(Duration::from_secs_f64(retry_after)).await;
            continue;
        }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            bail!("webhook returned {status}: {body}");
        }
        return Ok(());
    }
    bail!("webhook is still rate limited after {MAX_ATTEMPTS} attempts")
}

#[cfg(test)]
mod tests {
    use super::*;

    mod render {
        use super::*;

        fn entry(title: &str, description_html: &str) -> Entry {
            Entry {
                id: "id".to_string(),
                title: title.to_string(),
                link: "https://example.com/1".to_string(),
                description_html: description_html.to_string(),
                date: None,
            }
        }

        #[test]
        fn expands_placeholders() {
            let text = render(
                "🚨 **{feed_title} | {title}**\n\n{link}\n\n{description}",
                "Status",
                &entry("Incident", "<p>Investigating <b>elevated</b> errors.</p>"),
            );
            assert_eq!(
                text,
                "🚨 **Status | Incident**\n\nhttps://example.com/1\n\nInvestigating elevated errors."
            );
        }

        #[test]
        fn keeps_unknown_placeholders_and_braces() {
            let text = render("{unknown} {title} {", "", &entry("A", ""));
            assert_eq!(text, "{unknown} A {");
        }

        #[test]
        fn does_not_expand_placeholders_in_values() {
            let text = render("{title} {link}", "", &entry("{link}", ""));
            assert_eq!(text, "{link} https://example.com/1");
        }

        #[test]
        fn truncates_to_2000_chars_with_ellipsis() {
            let text = render("{description}", "", &entry("", &"あ".repeat(2500)));
            assert_eq!(text.chars().count(), 2000);
            assert!(text.ends_with("あ…"));
        }

        #[test]
        fn keeps_text_of_exactly_2000_chars() {
            let text = render("{description}", "", &entry("", &"あ".repeat(2000)));
            assert_eq!(text, "あ".repeat(2000));
        }
    }

    mod post {
        use super::*;

        #[tokio::test]
        async fn keeps_webhook_url_out_of_errors() {
            // The .invalid TLD never resolves (RFC 2606), so the request always fails.
            let channel = Channel {
                discord_webhook: "https://discord.invalid/api/webhooks/1/secret-token".to_string(),
                template: String::new(),
                username: None,
                avatar_url: None,
            };
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(1))
                .build()
                .unwrap();
            let err = post(&client, &channel, "content").await.unwrap_err();
            assert!(!format!("{err:#}").contains("secret-token"), "{err:#}");
        }
    }
}
