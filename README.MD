# JadeMusic

Discord 向け音楽再生 Bot。YouTube などの URL をボイスチャンネルで再生できます。

## 必要な環境

- **ffmpeg** — 音声デコード・変換に使用
- **yt-dlp** — 動画サイトからの音声取得に使用
- **deno** — yt-dlp の JavaScript ランタイムとして使用

---

## 環境構築手順

### 1. ffmpeg

```bash
# Ubuntu / Debian
sudo apt update && sudo apt install -y ffmpeg

# macOS (Homebrew)
brew install ffmpeg
```

### 2. yt-dlp

apt などのパッケージマネージャ版は古い場合があるため、pip でインストールしてください。

```bash
pip install -U yt-dlp
```

> **注意**: apt でインストール済みの場合、古いバージョンが PATH に残ることがあります。  
> `which yt-dlp` でパスを確認し、必要に応じてシンボリックリンクを更新してください。
>
> ```bash
> # pip インストール先を確認して上書きする例
> sudo ln -sf "$(python3 -c 'import yt_dlp, os; print(os.path.dirname(yt_dlp.__file__))')"/../../../bin/yt-dlp /usr/local/bin/yt-dlp
> ```

バージョンの確認：

```bash
yt-dlp --version   # 2026.03.17 以降推奨
```

### 3. deno

npm 経由でインストールします（Node.js が必要）。

```bash
npm install -g deno
deno --version
```

---

## 設定

プロジェクトルートに `.env` ファイルを作成し、以下の値を設定します。

```env
DISCORD_TOKEN=your_discord_bot_token
YTDLP_COOKIES=/path/to/cookies.txt
```

| 変数名 | 説明 |
|---|---|
| `DISCORD_TOKEN` | Discord Developer Portal で取得した Bot トークン |
| `YTDLP_COOKIES` | YouTube 認証用 cookies.txt のパス（年齢制限コンテンツなどに必要） |

### Discord Bot の設定

[Discord Developer Portal](https://discord.com/developers/applications) の Bot 設定で以下を確認してください。

- **Server Members Intent** — 有効化
- **Message Content Intent** — 不要（slash command のみ使用）

---

## ビルドと実行

```bash
# 開発用ビルド
cargo run

# リリースビルド
cargo run --release

# musl ターゲットでのビルド（Linux 静的リンクバイナリ）
cargo build --release --target x86_64-unknown-linux-musl
```

---

## コマンド一覧

| コマンド | 説明 |
|---|---|
| `/music play <url>` | URL を再生キューに追加（ボイスチャンネル未参加の場合は自動参加） |
| `/music skip` | 再生中の曲をスキップ |
| `/music list` | 再生キューの一覧を表示 |
| `/music shuffle` | 再生キューをシャッフル（再生中の曲は除く） |
| `/music delete <n>` | キューの n 番目の曲を削除 |
| `/music join` | ボイスチャンネルに参加 |
| `/music leave` | ボイスチャンネルから退出 |
