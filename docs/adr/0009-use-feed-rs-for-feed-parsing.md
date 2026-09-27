# 9. Use feed-rs for feed parsing

Date: 2026-09-27

## Status

Accepted

## Context

RSS 2.0 / RSS 1.0 / Atom のフィードを解析する必要がある。Rust では `rss` と `atom_syndication` がそれぞれの形式を扱い、feed-rs は複数の形式を 1 つのモデルに統一して扱う。

## Decision

フィードの解析に feed-rs を使う。取得は reqwest(ADR-0008)で行い、得たバイト列を feed-rs に渡す。

## Consequences

- 形式を気にせず、ID、タイトル、リンク、公開日、本文を同じ型で扱える(JSON Feed にも対応する)
- ID のない記事には feed-rs が ID を生成する。その扱いは ADR-0010 で決める
