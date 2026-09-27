# 2. Relay feeds with webhooks instead of a bot

Date: 2026-09-27

## Status

Accepted

## Context

RSS / Atom フィードの新着を Discord の 2 つのチャンネルに流したい。

- #news: 技術ブログなど
- #alerts: パブリッククラウドなどの service status

フィードの登録・編集・削除は、設定ファイルを編集できれば十分で、Web UI や Discord のコマンドは要らない。

Discord に投稿する方法は 2 つある。

- bot: アプリケーションを登録し、bot トークンを管理する必要がある。サーバーへの招待や権限の管理も伴い、コマンドなどを受けるには gateway に接続し続ける
- Incoming Webhook: チャンネルごとに発行した URL に HTTP POST するだけで投稿できる。できることは投稿だけ

必要なのは投稿だけなので、bot の機能は使わない。

## Decision

フィードを取得して Discord の Incoming Webhook に HTTP POST するだけのツール feedhook を作る。

- bot にはせず、gateway にも接続しない
- Web UI や Discord のコマンドは持たない。設定はファイルで行う

## Consequences

- bot トークン、招待、権限の管理が要らない。Discord からイベントを受け取る経路もない
- 投稿者の名前とアイコンは webhook の `username` / `avatar_url` で指定できる
- フィードの追加や変更は設定ファイルの編集になる
- 機能の追加(フィルター、再投稿の条件など)は自分で実装する必要がある
