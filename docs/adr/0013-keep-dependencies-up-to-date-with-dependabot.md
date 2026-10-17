# 13. Keep dependencies up to date with Dependabot

Date: 2026-10-17

## Status

Accepted

## Context

依存しているクレートと GitHub Actions を、手で追いかけずに更新したい。脆弱性が見つかったときにも気づけるようにしたい。

依存の更新の候補は、Dependabot と Renovate で、どちらも Cargo と GitHub Actions に対応している。

- Renovate: 設定の自由度が高い。ただし GitHub App にリポジトリへの書き込み権限を渡すことになる。App が乗っ取られた場合に備えるには、タグの保護、merge の前の承認の必須、Secrets の置き場所の見直しなどが要る
- Dependabot: GitHub 自身の機能なので、リポジトリへの書き込みを任せる相手が増えない

脆弱性の検知の候補は、`cargo audit` / `cargo deny` を CI で動かすことと、Dependabot alerts である。Dependabot alerts は GitHub Advisory Database を使い、RustSec の勧告も取り込んでいる。

## Decision

- 依存の更新には Dependabot を使う。対象は `cargo` と `github-actions` で、週 1 回とする
  - 公開から 7 日たったバージョンだけを提案させる(`cooldown`)。公開直後に乗っ取られたバージョンを取り込まないため
  - patch の更新は、CI が通ったら自動で merge する
- 脆弱性の検知は Dependabot alerts に任せ、`cargo audit` / `cargo deny` は入れない

## Consequences

- 信頼する相手は GitHub だけのままになる
- Rust のツールチェーン(`mise.toml`)は Dependabot が更新できないので、手で上げる
- ライセンスの確認や、重複した依存の検出など、`cargo deny` にしかない検査は行わない。必要になったら見直す
