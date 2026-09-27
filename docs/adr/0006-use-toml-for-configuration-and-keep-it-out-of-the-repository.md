# 6. Use TOML for configuration and keep it out of the repository

Date: 2026-09-27

## Status

Accepted

## Context

設定ファイルの形式と置き場所を決める。設定にはチャンネルごとの webhook URL(知っていれば誰でも投稿できる秘密情報)と、フィードの一覧が含まれる。

YAML は Rust での定番だった `serde_yaml` が 2024 年に非推奨となり、後継のクレートが定まっていない。TOML は Cargo でも使われる形式で、`toml` クレートがそのまま使える。

## Decision

- 設定は TOML で書き、`toml` クレートと serde で読み込む
- 設定ファイル(`config.toml`)はリポジトリに含めず、`.gitignore` に入れる。リポジトリには書式の例として `config.example.toml` を置く
- このリポジトリはツール本体のためのものとし、実運用のフィード一覧を git で管理したくなったら、別のリポジトリで管理する
- 起動時に、各フィードが参照するチャンネルが設定に存在するかを検証する

## Consequences

- webhook URL を誤ってコミットする心配がない
- サーバー上の `config.toml` はパーミッションを 600 にして管理する
- 設定の変更履歴は、別リポジトリを作るまでは残らない
