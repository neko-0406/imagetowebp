#!/usr/bin/env sh
# install.sh — imagetowebp インストールスクリプト (Linux / macOS)
#
# 使い方:
#   curl -fsSL https://raw.githubusercontent.com/<owner>/imagetowebp/main/install.sh | sh
#
# オプション:
#   INSTALL_DIR=/custom/path sh install.sh   # インストール先を変更 (デフォルト: /usr/local/bin)

set -e

REPO="<owner>/imagetowebp"
BINARY="imagetowebp"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
RELEASE_URL="https://github.com/${REPO}/releases/download/latest"

# ── OS 判定 ──────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
  Linux)
    case "${ARCH}" in
      x86_64) ASSET="imagetowebp-linux-x86_64" ;;
      *)
        echo "エラー: サポートされていないアーキテクチャです: ${ARCH}" >&2
        echo "手動ビルド: cargo install --git https://github.com/${REPO}" >&2
        exit 1
        ;;
    esac
    ;;
  *)
    echo "エラー: このスクリプトは Linux 専用です。" >&2
    echo "Windows は release.sh を参照してください。" >&2
    exit 1
    ;;
esac

# ── ダウンロード ───────────────────────────────────────────
DOWNLOAD_URL="${RELEASE_URL}/${ASSET}"
TMP_FILE="$(mktemp)"

echo ">>> imagetowebp をダウンロード中: ${DOWNLOAD_URL}"
if command -v curl >/dev/null 2>&1; then
  curl -fsSL "${DOWNLOAD_URL}" -o "${TMP_FILE}"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "${TMP_FILE}" "${DOWNLOAD_URL}"
else
  echo "エラー: curl または wget が必要です。" >&2
  exit 1
fi

chmod +x "${TMP_FILE}"

# ── インストール ───────────────────────────────────────────
if [ -w "${INSTALL_DIR}" ]; then
  mv "${TMP_FILE}" "${INSTALL_DIR}/${BINARY}"
else
  echo ">>> sudo でインストールします: ${INSTALL_DIR}/${BINARY}"
  sudo mv "${TMP_FILE}" "${INSTALL_DIR}/${BINARY}"
fi

echo ">>> インストール完了: $(command -v ${BINARY})"
echo ">>> バージョン確認: ${BINARY} --version"
