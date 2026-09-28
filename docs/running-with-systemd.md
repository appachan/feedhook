# Running feedhook with systemd

feedhook を Linux のサーバーに置き、systemd timer で 10 分おきに動かす手順。1 回の起動ですべてのフィードを処理して終了するので、定期実行は systemd に任せる。

前提:

- x86_64 の Linux で、systemd と curl がある
- 以下の例では、feedhook を動かすユーザーを `feedhook`、置き場所を `/opt/feedhook` とする。どちらも好きに変えてよい。ディレクトリには、そのユーザーが書き込めるようにしておく(`state.json` を書くため)
- 特に断りのないコマンドは、`/opt/feedhook` で `feedhook` ユーザーとして実行する

## 1. バイナリを取得する

[Releases](https://github.com/appachan/feedhook/releases) から、`x86_64-unknown-linux-musl` 向けのバイナリを取得する。`VERSION` は取得するタグ。

```sh
VERSION=v0.1.0
curl -fsSLo feedhook "https://github.com/appachan/feedhook/releases/download/$VERSION/feedhook-x86_64-unknown-linux-musl"
chmod +x feedhook
./feedhook --version
```

## 2. 設定を書く

[`config.example.toml`](../config.example.toml) を `config.toml` として置き、webhook の URL とフィードを書く。webhook の URL は秘密情報なので、ほかのユーザーから読めないようにする。

```sh
chmod 600 config.toml
```

1 回目を実行する。1 回目は今ある記事を既読にするだけで、何も投稿しない。取得や解析に失敗したフィードがあれば、エラーが表示される。

```sh
./feedhook
```

- TLS のエラーが出たら、CA 証明書を入れる(Debian 系なら `ca-certificates` パッケージ)

## 3. systemd timer で定期実行する

root で、service と timer の unit ファイルを作る。

`/etc/systemd/system/feedhook.service`:

```ini
[Unit]
Description=Relay feeds to Discord webhooks
Wants=network-online.target
After=network-online.target

[Service]
Type=oneshot
User=feedhook
WorkingDirectory=/opt/feedhook
ExecStart=/opt/feedhook/feedhook
```

`/etc/systemd/system/feedhook.timer`:

```ini
[Unit]
Description=Run feedhook every 10 minutes

[Timer]
OnCalendar=*:0/10
Persistent=true

[Install]
WantedBy=timers.target
```

有効にして、状態を確かめる。

```sh
sudo systemctl daemon-reload
sudo systemctl enable --now feedhook.timer
systemctl list-timers feedhook.timer   # 次の実行時刻
systemctl status feedhook.service      # 直前の実行結果
journalctl -u feedhook                 # 実行ログ
```

- `Type=oneshot` なので、実行を終えるとサービスは止まる。`inactive (dead)` と `status=0/SUCCESS` が出ていれば成功
- 失敗があると、feedhook は終了コード 1 で終わり、unit は failed になる。内容は journal に出る
- 設定の変更は、次の起動から反映される。再起動は要らない

## 4. 失敗を検知する

feedhook 自体には通知の仕組みがなく、失敗は終了コードと journal でしかわからない。気づきたいのは、フィードの URL の変更や webhook の削除のように、直すまで続く失敗である。一時的なネットワークエラーは次の起動で回復するので、そのたびに知らせる必要はない。

ここでは、外部の死活監視サービス(dead man's switch)を使う例を示す。feedhook が成功するたびに ping を送り、一定の時間 ping が来なければサービスが通知する。サーバーや timer が止まった場合にも気づける。例として [healthchecks.io](https://healthchecks.io/) を使う。

1. healthchecks.io で check を作る
   - Period は 10 分(起動の間隔)、Grace は 1 時間程度にする
   - 通知先(Integrations)に Discord などを設定する
2. check の ping の URL を控える
3. `feedhook.service` の `[Service]` に次の行を足す

   ```ini
   ExecStartPost=curl -fsS -m 10 --retry 5 -o /dev/null https://hc-ping.com/<check の UUID>
   ```

   `Type=oneshot` では、`ExecStartPost=` は feedhook が成功したときにだけ実行される。失敗を知らせる ping(`/fail`)は送らない

4. 反映する

   ```sh
   sudo systemctl daemon-reload
   ```

次の実行のあと、healthchecks.io の画面で ping が届いていることを確かめる。

## 5. 更新する

1 と同じ手順で新しいバージョンを `feedhook.new` として取得し、確かめてから置き換える。

```sh
VERSION=<新しいタグ>
curl -fsSLo feedhook.new "https://github.com/appachan/feedhook/releases/download/$VERSION/feedhook-x86_64-unknown-linux-musl"
chmod +x feedhook.new
./feedhook.new --version
./feedhook.new --dry-run
mv feedhook.new feedhook
```

- `mv` はファイルを一度に入れ替えるので、timer で実行中でも壊れない
- 戻すときは、前のタグで同じ手順を行う
