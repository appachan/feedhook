# 12. Keep release artifacts unsigned

Date: 2026-10-17

## Status

Accepted

## Context

ADR-0011 で、`v*` のタグを push すると GitHub Actions でビルドし、GitHub Release にバイナリを置くことにした。Release は immutable releases にしてあり、公開した後でバイナリを差し替えることはできない。

配布物が改ざんされていないことを確かめる仕組みとして、次のものがある。

- タグの ruleset: 誰がタグを作れるか、作ったタグを動かしたり消したりできるかを制限する
- artifact attestations: バイナリがどのワークフローのどの実行でビルドされたかを、署名付きで記録する
- checksum のファイル: Release にハッシュ値を置き、取得したバイナリと照合できるようにする

いずれも、リポジトリに書き込める人が複数いる場合や、メンテナー以外の人がバイナリを取得する場合に意味が大きい。今は、リポジトリに書き込めるのはメンテナー本人だけで、Release のバイナリを取得して使うのもメンテナー本人だけである。

## Decision

- タグの ruleset、artifact attestations、checksum のファイルは使わない
- 改ざんへの備えは、リポジトリへの書き込み権限を絞ることと、immutable releases に任せる

## Consequences

- リリースの手順とワークフローが単純なままになる
- 利用者は、バイナリがこのリポジトリのワークフローでビルドされたことを、自分で検証できない
- 書き込める人が増えたときや、メンテナー以外の利用者が出てきたときは、見直す
