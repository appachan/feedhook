# feedhook

feedhook は、RSS / Atom フィードの新着記事を Discord の webhook に投稿するツールです。

- 1 回の起動ですべてのフィードを処理して終了します。常駐はしないので、定期実行には systemd timer や cron を使ってください
- 投稿には Discord の incoming webhook だけを使うので、bot を用意する必要はありません
- シングルバイナリで動きます

## Usage

Linux (x86_64) 向けのバイナリを [Releases](https://github.com/appachan/feedhook/releases) からダウンロードします。[`config.example.toml`](config.example.toml) を `config.toml` にコピーし、webhook の URL とフィードを書いてから実行します。

```sh
./feedhook
```

設定ファイルは、既定ではカレントディレクトリの `config.toml` を読みます。別の場所にあるファイルを使うときは、`--config` で指定します。

```sh
./feedhook --config path/to/config.toml
```

- フィードごとに投稿先のチャンネルを選び、チャンネルごとにメッセージのテンプレートを決めます。テンプレートには `{title}`、`{link}`、`{description}`、`{feed_title}` を書けます
- 初めて処理するフィードでは、今ある記事を既読にするだけで何も投稿しません。2 回目からは、`max_age_hours`(既定は 24 時間)以内に公開された新着記事を投稿します
- 既読の記事は `state.json` に記録されます。置き場所は `state_path` で変えられます
- 取得や投稿に失敗したフィードがあっても、残りのフィードは処理します。1 つでも失敗すると、終了ステータス 1 で終了します

`--dry-run` を付けると、Discord に投稿せず、投稿するはずのメッセージを表示します。`state.json` も更新しないので、何度でも試せます。

```sh
./feedhook --dry-run
```

systemd timer で定期実行する場合は、[Running with systemd](docs/running-with-systemd.md) を参照してください。

## Development

ツールチェーンは [mise](https://mise.jdx.dev/) で管理しています。

```sh
mise install
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## License

[MIT](LICENSE)
