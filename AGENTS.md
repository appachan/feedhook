# AGENTS.md

## テスト

- テストの対象が複数の関数にまたがる場合は、`mod tests` の中に対象の関数ごとのモジュールを作る(例: `foo::tests::bar::returns_empty_when_input_is_empty`)。対象が 1 つだけなら分けない

## GitHub Actions の依存

- action は `.github/workflows/actions.lock` でコミットに固定している。ワークフローの `uses:` にはタグ(例: `actions/checkout@v7.0.1`)を書き、SHA pinning はしない
- ツールは `gh extension install github/gh-actions-lock --pin v0.4.0` で入れる
- ワークフローを追加したときや `uses:` を追加・削除・変更したときは、リポジトリのルートで `gh actions-lock` を実行し、生成された lockfile をワークフローの変更と一緒にコミットする。lockfile は手で編集しない
- 既存の action の更新は Dependabot が lockfile ごと行う
