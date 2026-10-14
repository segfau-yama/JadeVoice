# JadeVoice

Discord 向けテキスト読み上げ（TTS）Bot。[Voicevox](https://voicevox.hiroshiba.jp/) を使ってテキストチャンネルのメッセージをボイスチャンネルで読み上げます。

## 動作概要

1. `/voice join` でボイスチャンネルに参加
2. Bot が参加しているボイスチャンネルと **同名のテキストチャンネル** に投稿されたメッセージを Voicevox で合成・読み上げ
3. `/voice leave` で退出（読み上げ停止）

---

## 必要な環境

- **Rust** — ビルドに必要（`rustup` 推奨）
- **Voicevox Core** — `voicevox_core/` ディレクトリに同梱（別途配置が必要な場合は [公式リリース](https://github.com/VOICEVOX/voicevox_core/releases) を参照）

外部コマンド（ffmpeg・yt-dlp・deno など）は**不要**です。

---

## 環境構築手順

### 1. Rust のインストール

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustc --version
```

### 2. Voicevox Core の配置

リポジトリには `voicevox_core/` ディレクトリに以下が含まれている必要があります。

```
voicevox_core/
  additional_libraries/ # CUDA・cuDNN など（GPU 版のみ）
  c_api/lib/          # libvoicevox_core.so など
  dict/               # open_jtalk_dic_utf_8-x.xx/
  models/vvms/        # *.vvm 音声モデルファイル
  onnxruntime/lib/    # libvoicevox_onnxruntime.so など
```

#### CUDA 版 ONNX Runtime の導入

NVIDIA GPU を使う場合は、先にドライバーが認識されていることを確認します。CUDA Toolkit の別途インストールは不要です。

```bash
nvidia-smi
```

[VOICEVOX Core の公式リリース](https://github.com/VOICEVOX/voicevox_core/releases)から Linux x64 用ダウンローダーを取得し、プロジェクトルートへ `download-linux-x64` として配置します。新規導入では、Core・音声モデル・CUDA 版 ONNX Runtime をまとめてダウンロードできます。

```bash
chmod +x ./download-linux-x64
./download-linux-x64 --devices cuda --output ./voicevox_core
```

CPU 版を導入済みの場合は、ONNX Runtime と追加ライブラリだけを CUDA 版へ置き換えます。

```bash
./download-linux-x64 \
  --only onnxruntime additional-libraries \
  --devices cuda \
  --output ./voicevox_core
```

ダウンロードした CUDA ライブラリを実行時の検索対象へ追加してから起動します。

```bash
export LD_LIBRARY_PATH="$PWD/voicevox_core/c_api/lib:$PWD/voicevox_core/onnxruntime/lib:$PWD/voicevox_core/additional_libraries${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
./JadeVoice
```

次のログが表示されれば、CUDAが有効です。起動中のプロセスがある場合は、一度終了してから再起動してください。

```text
CUDA (device_id=0): OK
CUDA (device_id=0)を利用します
```

Linux では `DirectML ... 利用できません` と表示されても問題ありません。`CPUを利用します` と表示される場合は、CPU 版 ONNX Runtime が残っていないか、`LD_LIBRARY_PATH` に `additional_libraries` が含まれているか確認してください。

---

## 設定

プロジェクトルートに `.env` ファイルを作成し、以下の値を設定します。

```env
DISCORD_TOKEN=your_discord_bot_token
VOICEVOX_DICTIONARY=voicevox_core/dict/open_jtalk_dic_utf_8-1.11
VOICEVOX_ONNXRUNTIME=voicevox_core/onnxruntime/lib/libvoicevox_onnxruntime.so.1.17.3
VOICEVOX_MODEL_DIR=voicevox_core/models/vvms
```

| 変数名 | 説明 |
|---|---|
| `DISCORD_TOKEN` | Discord Developer Portal で取得した Bot トークン |
| `VOICEVOX_DICTIONARY` | Open JTalk 辞書ディレクトリのパス |
| `VOICEVOX_ONNXRUNTIME` | ONNX Runtime 共有ライブラリのパス |
| `VOICEVOX_MODEL_DIR` | VOICEVOX 音声モデル（`.vvm`）ディレクトリのパス |

### Discord Bot の設定

[Discord Developer Portal](https://discord.com/developers/applications) の Bot 設定で以下を有効化してください。

- **Server Members Intent** — 有効化
- **Message Content Intent** — **有効化**（メッセージ読み上げに必要）

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
| `/voice join` | ボイスチャンネルに参加し、読み上げを開始する |
| `/voice leave` | ボイスチャンネルから退出し、読み上げを停止する |
| `/voice model <style_id>` | 読み上げに使用する Voicevox スタイル ID を変更する（Bot オーナーのみ） |
