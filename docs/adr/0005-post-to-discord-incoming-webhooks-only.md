# 5. Post to Discord incoming webhooks only

Date: 2026-09-27

## Status

Accepted

## Context

投稿先は当面 Discord だけである。webhook のリクエスト形式(`content`、`username`、`avatar_url`)、markdown、メッセージの文字数の上限、レート制限はいずれも Discord 固有で、Slack などと共通化しても得るものが少ない。

## Decision

投稿先を Discord の Incoming Webhook に限定し、送り先を抽象化しない。設定のキー名(`discord_webhook`)などで Discord 用であることを明示する。

- メッセージはチャンネルごとのテンプレート(`{title}` `{link}` `{description}` `{feed_title}`)を展開して作る。`description` は記事の HTML をテキストに変換したもの。`feed_title` は、フィードの XML に書かれたフィード全体の名前(RSS の `<channel>` 直下の `<title>`、Atom の `<feed>` 直下の `<title>`)。ただし、設定ファイルの `[[feeds]]` で `title` を指定すれば、そちらを使う(フィードの XML に書かれた名前は「fastly rss feed」「Akamai Status - Incident History」のように、投稿に向かないことが多いため)
- メッセージが 2000 文字を超える場合は、末尾を切り詰めて `…` を付ける
- `allowed_mentions` を `{"parse": []}` にして、記事中の `@everyone` などでメンションが飛ばないようにする
- 429 が返ったら、レスポンスの `retry_after`(秒)だけ待って再送する。再送の回数には上限を設ける
- 投稿に失敗した記事は既読にせず、次回の起動で再試行する
- リンクのプレビューは Discord が URL から自動で生成するので、embed は使わずテキストだけを送る

## Consequences

- 実装が小さく、Discord の仕様に合わせた細かい制御がしやすい
- 他のサービスに対応したくなったら、投稿の部分を作り直す必要がある
