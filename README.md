# imagetowebp 🖼️⚡

[![CI](https://github.com/<owner>/imagetowebp/actions/workflows/ci.yml/badge.svg)](https://github.com/<owner>/imagetowebp/actions/workflows/ci.yml)
[![Release](https://github.com/<owner>/imagetowebp/actions/workflows/release.yml/badge.svg)](https://github.com/<owner>/imagetowebp/actions/workflows/release.yml)

ブログ掲載用の画像 (JPG, PNG, BMP, TIFF等) を堅牢かつ高速に WebP 形式へ変換する Rust 製 CLI ツールです。

---

## ⚡ インストール / 更新

### Linux (x86_64)
```bash
# 新規インストール・更新（同じコマンドで更新も可）
curl -fsSL https://raw.githubusercontent.com/<owner>/imagetowebp/main/install.sh | sh
```

または手動でダウンロード:
```bash
curl -L https://github.com/<owner>/imagetowebp/releases/download/latest/imagetowebp-linux-x86_64 \
  -o imagetowebp && chmod +x imagetowebp && sudo mv imagetowebp /usr/local/bin/
```

### Windows (PowerShell)
```powershell
# 新規インストール・更新（同じコマンドで更新も可）
irm https://raw.githubusercontent.com/<owner>/imagetowebp/main/install.ps1 | iex
```

または手動でダウンロード:
```powershell
Invoke-WebRequest -Uri "https://github.com/<owner>/imagetowebp/releases/download/latest/imagetowebp-windows-x86_64.exe" `
  -OutFile "$env:USERPROFILE\imagetowebp.exe"
```

### ソースからビルド（Cargo）
```bash
cargo install --git https://github.com/<owner>/imagetowebp

# 更新時
cargo install --git https://github.com/<owner>/imagetowebp --force
```

---

## 概要と特徴

- 📁📄 **柔軟な入力形式**: フォルダ指定（再帰検索）・ファイル単位指定・混在指定に対応。
- ⚡ **超高速並列変換**: `rayon` によるマルチスレッド処理で大量の画像を一括で高速変換。
- 🛡️ **堅牢なエラーハンドリング**: 破損画像があっても全体を停止させず継続処理。
- 🔒 **安全設計**:
  - 変換済みの `.webp` ファイルを変換対象から自動除外（再処理防止）
  - 既存 WebP ファイルの誤上書きを防止（`--overwrite` で指定可能）
  - 同名ファイル衝突を防ぐ `--preserve-structure` オプション
- 🎨 **ブログ最適化機能**:
  - 画質調整 (1 ~ 100%、デフォルト: 80%)
  - 可逆 (Lossless) / 非可逆 (Lossy) モード選択
  - 長辺最大サイズ指定による自動リサイズ (アスペクト比維持)
- 📊 **インタラクティブ & リッチなUI**:
  - コマンドライン引数と対話型（ウィザード）モードの両対応
  - リアルタイム進捗バー・処理完了後サマリー表示

---

## ドキュメント一覧 (`docs/`)

- [docs/implementation_plan.md](docs/implementation_plan.md) : 開発計画書・全体要件
- [docs/PLAN.md](docs/PLAN.md) : 設計仕様およびアーキテクチャ詳細

---

## 主な使い方

### 1. 対話型モード (コマンドライン引数なしで実行)
```bash
imagetowebp
```

### 2. コマンドライン引数モード

```bash
# フォルダ単位指定 (再帰的に一括変換)
imagetowebp -i ./images

# ファイル単位指定 (複数ファイル)
imagetowebp -i photo1.png photo2.jpg

# フォルダとファイルを混在
imagetowebp -i ./images_dir photo3.png

# 画質85%、最大長辺1920pxにリサイズ
imagetowebp -i ./images -q 85 -m 1920

# 既存WebPを上書き許可
imagetowebp -i ./images -w

# サブフォルダ構造を保持して出力 (同名衝突防止)
imagetowebp -i ./images -o ./output -s

# 可逆圧縮 (Lossless) モード
imagetowebp -i ./images -l
```

### コマンドラインオプション一覧

| オプション | 短縮 | 説明 |
|:---|:---:|:---|
| `--input <PATH>...` | `-i` | 変換対象のファイルまたはフォルダ（複数指定可） |
| `--output <DIR>` | `-o` | 出力先ディレクトリ（省略時は元ファイルと同階層） |
| `--quality <1-100>` | `-q` | 画質設定 (デフォルト: 80) |
| `--lossless` | `-l` | 可逆圧縮モード（quality は無視される） |
| `--max-dimension <px>` | `-m` | 画像の長辺の最大ピクセル数（アスペクト比維持） |
| `--overwrite` | `-w` | 既存の WebP ファイルを上書きする |
| `--preserve-structure` | `-s` | 出力先にサブフォルダ構造を保持する（衝突防止） |

---

## ビルド方法（開発者向け）

```bash
cargo build --release
```
`target/release/imagetowebp` (Windows: `.exe`) が生成されます。

---

## 対応入力フォーマット

`.jpg`, `.jpeg`, `.png`, `.bmp`, `.tiff` (大文字小文字不問)

> ⚠️ `.webp` ファイルは変換対象から自動的に除外されます（変換済みファイルの再処理防止）。

---

## CI / CD

| トリガー | 実行内容 |
|:---|:---|
| `main` への push / PR マージ | テスト (Linux・Windows) |
| `main` への push / PR マージ | リリースビルド → GitHub Release 自動更新 |
