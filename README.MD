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
  c_api/lib/          # libvoicevox_core.so など
  dict/               # open_jtalk_dic_utf_8-x.xx/
  models/vvms/        # *.vvm 音声モデルファイル
  onnxruntime/lib/    # libonnxruntime.so など
```

---

## 設定

プロジェクトルートに `.env` ファイルを作成し、以下の値を設定します。

```env
DISCORD_TOKEN=your_discord_bot_token
VOICEVOX_DICTIONARY=voicevox_core/dict/open_jtalk_dic_utf_8-1.11
VOICEVOX_ONNXRUNTIME=voicevox_core/onnxruntime/lib/libonnxruntime.so.1.17.3
```

| 変数名 | 説明 |
|---|---|
| `DISCORD_TOKEN` | Discord Developer Portal で取得した Bot トークン |
| `VOICEVOX_DICTIONARY` | Open JTalk 辞書ディレクトリのパス |
| `VOICEVOX_ONNXRUNTIME` | ONNX Runtime 共有ライブラリのパス |

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

### Jetson Nano（ARM64）

Jetson Nano 実機の 64-bit Linux 上で、プロジェクトルートから実行します。
コンパイル設定は `.cargo/config.toml` にまとめています。
ARM64 では CPU の Cortex-A57 向けに最適化し、同時ビルド数は 2 に制限します。
`jetson` プロファイルは最適化レベル 3 を維持しつつ、ThinLTO と
`codegen-units = 8` で通常の release よりビルド・リンク負荷を抑える設定です。
メモリ不足の場合は `-j 1` を追加してください。

```bash
# 実機のビルドツール（Rust は別途インストール）
sudo apt-get update
sudo apt-get install build-essential cmake pkg-config
rustup target add aarch64-unknown-linux-gnu

cargo build --target aarch64-unknown-linux-gnu --profile jetson
```

VOICEVOX Core と ONNX Runtime は **Linux ARM64 版**を配置してください。
このクラウド環境にある x64 版は実機では使えません。
標準の JetPack 4.x（Ubuntu 18.04 / glibc 2.27）では、現在の Rust・依存クレートや
配布済み VOICEVOX バイナリが要求する OS / glibc のバージョンに注意してください。
CPU 向けフラグだけでは解決しないため、互換性のあるライブラリのビルドか OS 環境の用意が必要です。
この設定は CPU 実行用であり、CUDA 対応を追加するものではありません。

```bash
# リポジトリ直下で起動（.env にトークンと辞書・Runtime のパスを設定）
export LD_LIBRARY_PATH="$PWD/voicevox_core/c_api/lib:$PWD/voicevox_core/onnxruntime/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
./target/aarch64-unknown-linux-gnu/jetson/JadeVoice
```

#### mold でリンクを高速化する場合

実機で `mold --version` が成功することを確認し、`.cargo/config.toml` の
`[target.aarch64-unknown-linux-gnu]` にある `link-arg=-fuse-ld=mold` の行を
コメント解除してください。
mold はリンク時間を短縮するためのもので、音声合成自体の高速化を保証しません。

```bash
# 利用中の OS のパッケージに mold がある場合
sudo apt-get install mold
mold --version

cargo build --target aarch64-unknown-linux-gnu --profile jetson
```

`cc` が `-fuse-ld=mold` に対応する必要があります（GCC 12.1 以降、または対応する Clang）。
GCC が古い環境では、対応する Clang と mold を導入したうえで同じセクションの
`linker = "cc"` を `linker = "clang"` に変更してください。
mold を利用できない場合は、そのフラグをコメントのままにして通常のリンカーを使ってください。
これらの設定は実機ビルド用です。x86_64 からのクロスコンパイルには別途、ARM64 用
C/C++ ツールチェーン、実機と互換性のある sysroot、ARM64 版 VOICEVOX ライブラリが必要です。
`RUSTFLAGS` / `CARGO_ENCODED_RUSTFLAGS`、`RUSTC_WRAPPER`、ターゲット固有の
linker / rustflags を上書きする環境変数がある場合は、設定の妨げになるため解除してください。
特にクラウド用 `/workspace/.jadevoice-setup/env.sh` は実機では読み込まないでください。

---

## コマンド一覧

| コマンド | 説明 |
|---|---|
| `/voice join` | ボイスチャンネルに参加し、読み上げを開始する |
| `/voice leave` | ボイスチャンネルから退出し、読み上げを停止する |
| `/voice model <style_id>` | 読み上げに使用する Voicevox スタイル ID を変更する（Bot オーナーのみ） |
