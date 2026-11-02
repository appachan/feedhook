# AGENTS.md

## テスト

- テストの対象が複数の関数にまたがる場合は、`mod tests` の中に対象の関数ごとのモジュールを作る(例: `foo::tests::bar::returns_empty_when_input_is_empty`)。対象が 1 つだけなら分けない
