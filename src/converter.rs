use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result, anyhow};
use image::{GenericImageView, DynamicImage};
use webp::{Encoder, WebPMemory};

use crate::config::ConvertConfig;

/// 変換結果の統計データ
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversionResult {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub original_size_bytes: u64,
    pub converted_size_bytes: u64,
    pub was_skipped: bool,
}

impl ConversionResult {
    /// 削減バイト数
    pub fn saved_bytes(&self) -> i64 {
        self.original_size_bytes as i64 - self.converted_size_bytes as i64
    }

    /// 削減率 (%)
    pub fn saved_percentage(&self) -> f64 {
        if self.original_size_bytes == 0 {
            return 0.0;
        }
        (1.0 - (self.converted_size_bytes as f64 / self.original_size_bytes as f64)) * 100.0
    }
}

/// 単一の画像ファイルを WebP に変換するコア関数
///
/// # Arguments
/// - `input_path` - 変換元ファイルパス
/// - `input_root` - フォルダ構造保持のための基底ディレクトリ（`preserve_structure` 有効時に使用）
/// - `output_dir` - 出力先ディレクトリ（None の場合は元画像と同じ場所に出力）
/// - `config` - 変換設定
pub fn convert_single_image(
    input_path: &Path,
    input_root: Option<&Path>,
    output_dir: Option<&Path>,
    config: &ConvertConfig,
) -> Result<ConversionResult> {
    if !input_path.is_file() {
        return Err(anyhow!("Input path is not a file: {:?}", input_path));
    }

    // 元画像のサイズ取得
    let metadata = fs::metadata(input_path)
        .with_context(|| format!("Failed to read metadata for {:?}", input_path))?;
    let original_size_bytes = metadata.len();

    // 出力先パスの決定
    let output_path = determine_output_path(input_path, input_root, output_dir, config.preserve_structure)?;

    // 上書きチェック
    if output_path.exists() && !config.overwrite {
        let converted_size_bytes = fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);
        return Ok(ConversionResult {
            input_path: input_path.to_path_buf(),
            output_path,
            original_size_bytes,
            converted_size_bytes,
            was_skipped: true,
        });
    }

    // 出力先ディレクトリの作成
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create output directory {:?}", parent))?;
        }
    }

    // 画像読み込み
    let img = image::open(input_path)
        .with_context(|| format!("Failed to open/decode image {:?}", input_path))?;

    // リサイズ処理 (長辺サイズ制限)
    let processed_img = resize_if_needed(img, config.max_dimension);

    // WebP エンコード
    // 注意: lossless = true の場合、quality パラメータは使用されません
    let webp_data: WebPMemory = encode_to_webp(&processed_img, config)?;

    // ファイル書き込み
    let mut file = File::create(&output_path)
        .with_context(|| format!("Failed to create target file {:?}", output_path))?;
    file.write_all(&webp_data)
        .with_context(|| format!("Failed to write WebP data to {:?}", output_path))?;

    let converted_size_bytes = webp_data.len() as u64;

    Ok(ConversionResult {
        input_path: input_path.to_path_buf(),
        output_path,
        original_size_bytes,
        converted_size_bytes,
        was_skipped: false,
    })
}

/// 出力ファイルのパスを決定する
///
/// `preserve_structure = true` かつ `output_dir` と `input_root` が両方指定されている場合は、
/// input_path の input_root からの相対パスを output_dir 配下に再現する。
/// 例: input_root=./images, input_path=./images/blog/header.png, output_dir=./out
///     → ./out/blog/header.webp
fn determine_output_path(
    input_path: &Path,
    input_root: Option<&Path>,
    output_dir: Option<&Path>,
    preserve_structure: bool,
) -> Result<PathBuf> {
    let file_stem = input_path
        .file_stem()
        .ok_or_else(|| anyhow!("Invalid file name: {:?}", input_path))?;
    let mut file_name = file_stem.to_os_string();
    file_name.push(".webp");

    if let Some(out_dir) = output_dir {
        if preserve_structure {
            if let Some(root) = input_root {
                // 入力ファイルの root からの相対パスを求め、output_dir 配下に構造を再現
                if let Ok(relative) = input_path.strip_prefix(root) {
                    let relative_dir = relative.parent().unwrap_or_else(|| Path::new(""));
                    let out_path = out_dir.join(relative_dir).join(file_name);
                    return Ok(out_path);
                }
            }
        }
        // preserve_structure 無効、または root からの相対パスが取れない場合は flat に出力
        Ok(out_dir.join(file_name))
    } else {
        // 出力先未指定: 元ファイルと同じディレクトリ
        let parent = input_path.parent().unwrap_or_else(|| Path::new(""));
        Ok(parent.join(file_name))
    }
}

/// 長辺制限に基づくアスペクト比維持リサイズ
fn resize_if_needed(img: DynamicImage, max_dimension: Option<u32>) -> DynamicImage {
    if let Some(max_dim) = max_dimension {
        let (width, height) = img.dimensions();
        if width > max_dim || height > max_dim {
            return img.thumbnail(max_dim, max_dim);
        }
    }
    img
}

/// WebP へエンコードする
///
/// `config.lossless = true` の場合は品質パラメータを無視して可逆エンコードを行う。
/// `config.lossless = false` の場合は `config.quality` (1..=100) を使用する。
fn encode_to_webp(img: &DynamicImage, config: &ConvertConfig) -> Result<WebPMemory> {
    let encoder = Encoder::from_image(img)
        .map_err(|e| anyhow!("Failed to create WebP encoder from image: {}", e))?;

    let webp_mem = if config.lossless {
        encoder.encode_lossless()
    } else {
        encoder.encode(config.quality as f32)
    };

    Ok(webp_mem)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{RgbImage, Rgb};
    use tempfile::tempdir;

    fn create_dummy_png(path: &Path, width: u32, height: u32) {
        let mut img = RgbImage::new(width, height);
        for x in 0..width {
            for y in 0..height {
                img.put_pixel(x, y, Rgb([x as u8 % 255, y as u8 % 255, 128]));
            }
        }
        img.save(path).unwrap();
    }

    #[test]
    fn test_convert_single_image_basic() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("test_input.png");
        create_dummy_png(&input_path, 100, 100);

        let config = ConvertConfig {
            input_paths: vec![input_path.clone()],
            quality: 80,
            ..Default::default()
        };

        let result = convert_single_image(&input_path, None, None, &config).unwrap();

        assert!(!result.was_skipped);
        assert!(result.output_path.exists());
        assert_eq!(result.output_path.extension().unwrap(), "webp");
        assert!(result.converted_size_bytes > 0);
    }

    #[test]
    fn test_convert_single_image_resize() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("big_input.png");
        create_dummy_png(&input_path, 400, 200);

        let config = ConvertConfig {
            input_paths: vec![input_path.clone()],
            max_dimension: Some(200),
            quality: 80,
            ..Default::default()
        };

        let result = convert_single_image(&input_path, None, None, &config).unwrap();
        assert!(result.output_path.exists());

        // 変換後の画像を読み込んでサイズ確認 (400x200 -> 200x100 になるはず)
        let converted_img = image::open(&result.output_path).unwrap();
        let (w, h) = converted_img.dimensions();
        assert_eq!(w, 200);
        assert_eq!(h, 100);
    }

    #[test]
    fn test_convert_single_image_skip_if_exists() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("test_input.png");
        create_dummy_png(&input_path, 50, 50);

        let config = ConvertConfig {
            input_paths: vec![input_path.clone()],
            overwrite: false,
            ..Default::default()
        };

        // 1回目の変換
        let res1 = convert_single_image(&input_path, None, None, &config).unwrap();
        assert!(!res1.was_skipped);

        // 2回目の変換 (上書きなしなのでスキップされる)
        let res2 = convert_single_image(&input_path, None, None, &config).unwrap();
        assert!(res2.was_skipped);
    }

    #[test]
    fn test_preserve_structure_output_path() {
        let root = Path::new("/images");
        let input = Path::new("/images/blog/header.png");
        let out_dir = Path::new("/out");

        let result = determine_output_path(input, Some(root), Some(out_dir), true).unwrap();
        assert_eq!(result, PathBuf::from("/out/blog/header.webp"));
    }

    #[test]
    fn test_flat_output_path_without_preserve_structure() {
        let input = Path::new("/images/blog/header.png");
        let out_dir = Path::new("/out");

        let result = determine_output_path(input, None, Some(out_dir), false).unwrap();
        assert_eq!(result, PathBuf::from("/out/header.webp"));
    }
}
