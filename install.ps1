# install.ps1 — imagetowebp インストールスクリプト (Windows PowerShell)
#
# 使い方 (PowerShell):
#   irm https://raw.githubusercontent.com/<owner>/imagetowebp/main/install.ps1 | iex
#
# オプション:
#   $env:INSTALL_DIR = "C:\Tools"   # インストール先を変更
#   irm ... | iex

$ErrorActionPreference = "Stop"

$Repo = "neko-0406/imagetowebp"
$BinaryName = "imagetowebp.exe"
$AssetName = "imagetowebp-windows-x86_64.exe"
$ReleaseUrl = "https://github.com/$Repo/releases/download/latest/$AssetName"

# インストール先ディレクトリ (デフォルト: ユーザーのローカルAppData\Programs\imagetowebp)
$InstallDir = if ($env:INSTALL_DIR) {
    $env:INSTALL_DIR
} else {
    "$env:LOCALAPPDATA\Programs\imagetowebp"
}

Write-Host ">>> imagetowebp をインストールします"
Write-Host ">>> ダウンロード元: $ReleaseUrl"
Write-Host ">>> インストール先: $InstallDir"

# インストール先ディレクトリを作成
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# バイナリをダウンロード
$TargetPath = Join-Path $InstallDir $BinaryName
Invoke-WebRequest -Uri $ReleaseUrl -OutFile $TargetPath -UseBasicParsing

Write-Host ">>> バイナリを配置しました: $TargetPath"

# PATH にインストール先を追加（ユーザースコープ）
$CurrentPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
if ($CurrentPath -notlike "*$InstallDir*") {
    [System.Environment]::SetEnvironmentVariable(
        "PATH",
        "$InstallDir;$CurrentPath",
        "User"
    )
    Write-Host ">>> PATH に $InstallDir を追加しました"
    Write-Host ">>> ターミナルを再起動すると 'imagetowebp' コマンドが使用できます"
} else {
    Write-Host ">>> PATH はすでに設定済みです"
}

Write-Host ""
Write-Host ">>> インストール完了!"
Write-Host ">>> 確認: imagetowebp --version"
