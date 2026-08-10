use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;

use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use walkdir::WalkDir;

use crate::config::ConvertConfig;
use crate::converter::{convert_single_image, ConversionResult};

/// 対応する画像拡張子一覧 (小文字)
const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "bmp", "tiff", "webp"];

/// 全体の処理結果サマリー
#[derive(Debug, Default)]
pub struct ProcessSummary {
    pub total_files: usize,
    pub success_count: usize,
    pub skipped_count: usize,
    pub failed_count: usize,
    pub original_total_bytes: u64,
    pub converted_total_bytes: u64,
    pub elapsed_secs: f64,
}

impl ProcessSummary {
    /// 削減総バイト数
    pub fn total_saved_bytes(&self) -> i64 {
        self.original_total_bytes as i64 - self.converted_total_bytes as i64
    }

    /// 削減総割合 (%)
    pub fn total_saved_percentage(&self) -> f64 {
        if self.original_total_bytes == 0 {
            return 0.0;
        }
        (1.0 - (self.converted_total_bytes as f64 / self.original_total_bytes as f64)) * 100.0
    }

    /// サマリーのコンソール表示
    pub fn print(&self) {
        println!("\n{}", "================ 変換処理完了サマリー ================".bold().cyan());
        println!("  処理対象ファイル数 : {}", self.total_files.to_string().bold());
        println!("  成功              : {}", self.success_count.to_string().green().bold());
        println!("  スキップ (既存)   : {}", self.skipped_count.to_string().yellow());
        if self.failed_count > 0 {
            println!("  失敗              : {}", self.failed_count.to_string().red().bold());
        } else {
            println!("  失敗              : {}", "0".bold());
        }

        let saved_mb = self.total_saved_bytes() as f64 / (1024.0 * 1024.0);
        let orig_mb = self.original_total_bytes as f64 / (1024.0 * 1024.0);
        let conv_mb = self.converted_total_bytes as f64 / (1024.0 * 1024.0);

        println!("  変換前総サイズ     : {:.2} MB", orig_mb);
        println!("  変換後総サイズ     : {:.2} MB", conv_mb);
        if self.total_saved_bytes() >= 0 {
            println!(
                "  削減量            : {} ({:.1}% 削減)",
                format!("{:.2} MB", saved_mb).bold().green(),
                self.total_saved_percentage().bold().green()
            );
        } else {
            println!(
                "  サイズ変化        : {} ({:.1}% 増加)",
                format!("{:.2} MB", -saved_mb).bold().yellow(),
                -self.total_saved_percentage()
            );
        }
        println!("  総所要時間        : {:.2} 秒", self.elapsed_secs);
        println!("{}", "======================================================".bold().cyan());
    }
}

/// 入力パス（ファイル/フォルダの混在）から対象画像パスを収集・重複排除する
pub fn collect_image_files(input_paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut unique_paths = HashSet::new();

    for path in input_paths {
        if path.is_file() {
            if is_supported_image(path) {
                if let Ok(canonical) = path.canonicalize() {
                    unique_paths.insert(canonical);
                } else {
                    unique_paths.insert(path.clone());
                }
            }
        } else if path.is_dir() {
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_file() && is_supported_image(p) {
                    if let Ok(canonical) = p.canonicalize() {
                        unique_paths.insert(canonical);
                    } else {
                        unique_paths.insert(p.to_path_buf());
                    }
                }
            }
        }
    }

    let mut result: Vec<PathBuf> = unique_paths.into_iter().collect();
    result.sort();
    result
}

/// 対応する画像拡張子かどうかをチェック
fn is_supported_image(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext_str| SUPPORTED_EXTENSIONS.contains(&ext_str.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// 画像変換パイプラインの実行
pub fn process_images(config: &ConvertConfig) -> Result<ProcessSummary> {
    let start_time = Instant::now();

    println!("{}", "画像ファイルを探索中...".bold().yellow());
    let image_files = collect_image_files(&config.input_paths);

    if image_files.is_empty() {
        println!("{}", "対象の画像ファイルが見つかりませんでした。".red());
        return Ok(ProcessSummary::default());
    }

    let total_files = image_files.len();
    println!("対象画像ファイル数: {} 件", total_files.to_string().cyan().bold());

    // プログレスバーの初期化
    let pb = ProgressBar::new(total_files as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .expect("Invalid progress bar template")
            .progress_chars("#>-"),
    );

    // アトミック集計カウンター
    let success_count = AtomicUsize::new(0);
    let skipped_count = AtomicUsize::new(0);
    let failed_count = AtomicUsize::new(0);
    let original_total_bytes = AtomicU64::new(0);
    let converted_total_bytes = AtomicU64::new(0);

    // Rayon による並列変換
    image_files.par_iter().for_each(|file_path| {
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("image");

        pb.set_message(format!("変換中: {}", file_name));

        match convert_single_image(file_path, config.output_dir.as_deref(), config) {
            Ok(res) => {
                original_total_bytes.fetch_add(res.original_size_bytes, Ordering::Relaxed);
                converted_total_bytes.fetch_add(res.converted_size_bytes, Ordering::Relaxed);

                if res.was_skipped {
                    skipped_count.fetch_add(1, Ordering::Relaxed);
                } else {
                    success_count.fetch_add(1, Ordering::Relaxed);
                }
            }
            Err(err) => {
                failed_count.fetch_add(1, Ordering::Relaxed);
                pb.println(format!("{} {:?}: {}", "エラー:".red().bold(), file_path, err));
            }        }

        pb.inc(1);
    });

    pb.finish_with_message("すべての処理が完了しました。".green().to_string());

    let elapsed = start_time.elapsed().as_secs_f64();

    Ok(ProcessSummary {
        total_files,
        success_count: success_count.load(Ordering::Relaxed),
        skipped_count: skipped_count.load(Ordering::Relaxed),
        failed_count: failed_count.load(Ordering::Relaxed),
        original_total_bytes: original_total_bytes.load(Ordering::Relaxed),
        converted_total_bytes: converted_total_bytes.load(Ordering::Relaxed),
        elapsed_secs: elapsed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_collect_image_files() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("img1.png");
        let file2 = dir.path().join("img2.JPG");
        let file_txt = dir.path().join("notes.txt");

        File::create(&file1).unwrap();
        File::create(&file2).unwrap();
        File::create(&file_txt).unwrap();

        let sub_dir = dir.path().join("subdir");
        std::fs::create_dir(&sub_dir).unwrap();
        let file3 = sub_dir.path().join("img3.jpeg");
        File::create(&file3).unwrap();

        // フォルダとファイルを混在指定
        let collected = collect_image_files(&[dir.path().to_path_buf(), file1.clone()]);

        // img1, img2, img3 の合計 3 個が収集されるはず（ファイル重複は排除）
        assert_eq!(collected.len(), 3);
    }
}
