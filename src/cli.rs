use std::path::PathBuf;
use clap::Parser;
use dialoguer::{Input, Confirm, Select};
use crate::config::ConvertConfig;

/// ブログ用画像を WebP 形式に超高速・堅牢に一括変換する CLI ツール
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// 変換対象の画像ファイルまたはディレクトリのパス (複数指定可能)
    #[arg(short, long, value_name = "PATH", num_args = 1..)]
    pub input: Vec<PathBuf>,

    /// 出力先ディレクトリ (未指定の場合は元画像と同じディレクトリに保存)
    #[arg(short, long, value_name = "DIR")]
    pub output: Option<PathBuf>,

    /// WebP 画質 (1〜100、デフォルト: 80)
    #[arg(short, long, default_value_t = 80)]
    pub quality: u8,

    /// Lossless (可逆圧縮) モードを使用
    #[arg(l, long, default_value_t = false)]
    pub lossless: bool,

    /// 画像の長辺の最大サイズ (ピクセル)。指定した場合、アスペクト比を維持して縮小
    #[arg(m, long, value_name = "PIXELS")]
    pub max_dimension: Option<u32>,

    /// 既存の WebP ファイルが存在する場合に上書きする
    #[arg(w, long, default_value_t = false)]
    pub overwrite: bool,
}

impl CliArgs {
    /// CLI引数を `ConvertConfig` に変換する
    pub fn to_config(self) -> ConvertConfig {
        ConvertConfig {
            input_paths: self.input,
            output_dir: self.output,
            quality: self.quality,
            lossless: self.lossless,
            max_dimension: self.max_dimension,
            overwrite: self.overwrite,
        }
    }
}

/// コマンドライン引数が未指定の場合に、対話形式でユーザーから設定を取得する
pub fn prompt_interactive_config() -> anyhow::Result<ConvertConfig> {
    println!("=== WebP 画像変換ツール (対話型ウィザード) ===");

    let target_type = Select::new()
        .with_prompt("変換対象の指定方法を選択してください")
        .items(&["フォルダ指定 (フォルダ内画像を再帰探索)", "ファイル単位指定 (直接パス入力)"])
        .default(0)
        .interact()?;

    let input_path_str: String = match target_type {
        0 => Input::new()
            .with_prompt("対象フォルダのパスを入力してください")
            .interact_text()?,
        _ => Input::new()
            .with_prompt("対象ファイルパスを入力してください (カンマ区切りで複数可)")
            .interact_text()?,
    };

    let input_paths: Vec<PathBuf> = input_path_str
        .split(',')
        .map(|s| PathBuf::from(s.trim()))
        .filter(|p| !p.as_os_str().is_empty())
        .collect();

    let output_str: String = Input::new()
        .with_prompt("出力先ディレクトリ (空欄のままEnterで元画像と同じフォルダ)")
        .allow_empty(true)
        .interact_text()?;

    let output_dir = if output_str.trim().is_empty() {
        None
    } else {
        Some(PathBuf::from(output_str.trim()))
    };

    let lossless = Confirm::new()
        .with_prompt("Lossless (可逆圧縮) にしますか？ (通常はいいえ=Lossy推奨)")
        .default(false)
        .interact()?;

    let quality: u8 = if lossless {
        100
    } else {
        Input::new()
            .with_prompt("画質を設定してください (1 ~ 100)")
            .default(80)
            .interact_text()?
    };

    let enable_resize = Confirm::new()
        .with_prompt("画像リサイズ (長辺制限) を行いますか？")
        .default(false)
        .interact()?;

    let max_dimension: Option<u32> = if enable_resize {
        Some(
            Input::new()
                .with_prompt("最大長辺ピクセル数 (例: 1920)")
                .default(1920)
                .interact_text()?,
        )
    } else {
        None
    };

    let overwrite = Confirm::new()
        .with_prompt("既存の WebP ファイルを上書きしますか？")
        .default(false)
        .interact()?;

    Ok(ConvertConfig {
        input_paths,
        output_dir,
        quality,
        lossless,
        max_dimension,
        overwrite,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_args_to_config() {
        let args = CliArgs {
            input: vec![PathBuf::from("dir1"), PathBuf::from("file2.png")],
            output: Some(PathBuf::from("out_dir")),
            quality: 85,
            lossless: false,
            max_dimension: Some(1920),
            overwrite: true,
        };

        let config = args.to_config();
        assert_eq!(config.input_paths.len(), 2);
        assert_eq!(config.output_dir, Some(PathBuf::from("out_dir")));
        assert_eq!(config.quality, 85);
        assert_eq!(config.max_dimension, Some(1920));
        assert!(config.overwrite);
    }
}
