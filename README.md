# imagetowebp 🖼️⚡

ブログ掲載用の画像 (JPG, PNG, BMP, TIFF等) を堅牢かつ高速に WebP 形式へ変換する Rust 製 CLI ツールです。

## 概要と特徴

- 📁📄 **柔軟な入力形式**: フォルダ指定（再帰検索）だけでなく、単一・複数の**ファイル単位での直接指定**や混在指定に対応。
- ⚡ **超高速並列変換**: `rayon` によるマルチスレッド処理で、大量の画像を一括で高速変換。
- 🛡️ **堅牢なエラーハンドリング**: 破損画像や読み込み不可ファイルがあっても全体を停止させず、ログを出力して処理を継続。
- 🎨 **ブログ最適化機能**:
  - 画質調整 (0 ~ 100%、デフォルト: 80%)
  - 可逆 (Lossless) / 非可逆 (Lossy) モード選択
  - 長辺最大サイズ指定による自動リサイズ (アスペクト比維持)
- 📊 **インタラクティブ & リッチなUI**:
  - コマンドライン引数と引数なし起動時の対話型（ウィザード）モードの両対応
  - リアルタイム進捗バー (`indicatif`)
  - 処理完了後の削減容量・削減率サマリー出力
- 🔒 **安全設計**: 既存 WebP ファイルの誤上書きを防止（`--overwrite` で指定可能）

---

## ドキュメント一覧 (`docs/`)

プロジェクトに関するすべてのドキュメントは `docs/` ディレクトリにまとめられています。

- [docs/implementation_plan.md](docs/implementation_plan.md) : 開発計画書・全体要件
- [docs/PLAN.md](docs/PLAN.md) : 設計仕様およびアーキテクチャ詳細

---

## 主な使い方（予定仕様）

### 1. 対話型モード (コマンドライン引数なしで実行)
```bash
cargo run --release
```
起動時に「フォルダ単位で変換」「ファイル単位で指定」を選択可能です。

### 2. コマンドライン引数モード

```bash
# フォルダ単位指定 (フォルダ内画像を再帰的に一括変換)
cargo run --release -- -i ./images

# ファイル単位指定 (単一・複数ファイル)
cargo run --release -- -i photo1.png photo2.jpg

# フォルダとファイルを混在して指定
cargo run --release -- -i ./images_dir photo3.png

# 画質85%、最大長辺1920pxにリサイズして一括変換
cargo run --release -- -i ./images -q 85 --max-dimension 1920

# 既存WebPを上書き許可
cargo run --release -- -i ./images --overwrite
```

---

## ビルド方法

```bash
cargo build --release
```
`target/release/imagetowebp` (Windowsの場合は `.exe`) が生成されます。
