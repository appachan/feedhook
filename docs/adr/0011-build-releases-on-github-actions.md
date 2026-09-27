# 11. Build releases on GitHub Actions

Date: 2026-09-28

## Status

Accepted

## Context

ADR-0003 で、Linux 向けのビルド方法はデプロイのときに決めることにした。

glibc に動的リンクしたバイナリは、ビルドした環境より古い glibc では動かないことがある。musl で静的リンクすれば、サーバーの glibc のバージョンに左右されない。

ビルドする場所の候補は次のとおり。

- 手元でクロスコンパイルする: cargo-zigbuild や cross などの追加のツールが要り、手元の環境に左右される。どの手元の状態からビルドしたのかも残らない
- サーバー上でビルドする: サーバーにツールチェーンを入れることになり、ADR-0003 の方針に反する
- GitHub Actions でビルドする: x86_64 の Ubuntu のランナーなら、musl 向けのターゲットと `musl-tools` を入れるだけでネイティブにビルドできる。タグとバイナリが対応し、ビルドの記録も残る

## Decision

- ターゲットは `x86_64-unknown-linux-musl` にし、静的リンクする
- `v*` のタグを push したら、GitHub Actions でビルドし、GitHub Release にバイナリを置く
  - タグは `v` + `Cargo.toml` の `version` とし、一致しなければ失敗させる。`--version` の表示とタグを揃えるため
  - Rust のツールチェーンとターゲットは、CI と同じく `mise.toml` から入れる
  - バイナリはアーカイブにせず、`feedhook-x86_64-unknown-linux-musl` という名前でそのまま置く。サーバーに置くのは 1 ファイルだけなので
  - Release の作成には、ランナーに入っている `gh` を使い、サードパーティの Action を増やさない
  - ビルドのキャッシュは使わない。リリースは頻繁ではなく、キャッシュの中身が成果物に混ざる余地をなくすため
- サーバーは、バージョンを指定して Release から curl でバイナリを取得し、置き換える

## Consequences

- リリースの手順は、`Cargo.toml` の `version` を上げる PR を merge し、そのコミットにタグを付けて push することになる
- 手元でのクロスコンパイルは不要になる
- 対象のアーキテクチャを増やすときは、ランナーやターゲットを追加する
