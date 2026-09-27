# 8. Use reqwest and tokio for HTTP

Date: 2026-09-27

## Status

Accepted

## Context

フィードの取得と webhook への投稿に HTTP クライアントが要る。候補は reqwest(async、tokio 上で動く)と ureq(同期のみ)。

- reqwest と tokio は、Rust の HTTP クライアントと async ランタイムの事実上の標準で、情報も多い
- ureq は同期 API だけを持ち、依存とバイナリが小さい。reqwest の blocking API も同期で書けるが、内部で tokio を動かしている
- 数十のフィードを 10 分おきに順番に取得するだけなら並列化は要らないが、将来は並列に取得したくなるかもしれない

## Decision

reqwest(既定の rustls による TLS)と tokio を使う。

- 最初は 1 フィードずつ順番に処理する
- 並列化が必要になったら、同時実行数を制限して(`futures` の `buffer_unordered` など)並列に取得する
- 429 の待機は `tokio::time::sleep` で行う

## Consequences

- 標準的なライブラリなので、情報や周辺のクレートを得やすい
- ureq に比べて依存が増え、ビルド時間とバイナリが大きくなる
- 順番に処理するうちは、async で増えるのは `#[tokio::main]`、`async fn`、`.await` 程度で、タスクの spawn に伴う `Send` の制約などはまだ出てこない
