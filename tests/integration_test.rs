use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use image::{RgbImage, Rgb};
use imagetowebp::config::ConvertConfig;
use imagetowebp::processor::process_images;

fn create_sample_png(path: &std::path::Path, width: u32, height: u32) {
    let mut img = RgbImage::new(width, height);
    for x in 0..width {
        for y in 0..height {
            img.put_pixel(x, y, Rgb([255, (x % 255) as u8, (y % 255) as u8]));
        }
    }
    img.save(path).unwrap();
}

#[test]
fn test_end_to_end_conversion_directory_and_file() {
    let dir = tempdir().unwrap();
    let sub_dir = dir.path().join("sub");
    std::fs::create_dir_all(&sub_dir).unwrap();

    let img1 = dir.path().join("image1.png");
    let img2 = sub_dir.join("image2.png");
    let corrupt_img = dir.path().join("corrupt.jpg");

    create_sample_png(&img1, 300, 300);
    create_sample_png(&img2, 500, 250);

    // 破損ファイルの作成 (無効なバイナリデータ)
    let mut f = File::create(&corrupt_img).unwrap();
    f.write_all(b"this is not a valid jpg image").unwrap();

    // 出力先フォルダ
    let out_dir = dir.path().join("output_webp");

    let config = ConvertConfig {
        input_paths: vec![dir.path().to_path_buf()],
        output_dir: Some(out_dir.clone()),
        quality: 75,
        lossless: false,
        max_dimension: Some(200),
        overwrite: true,
    };

    let summary = process_images(&config).unwrap();

    // 正しく処理されたか検証
    assert_eq!(summary.total_files, 3);
    assert_eq!(summary.success_count, 2); // 2個成功
    assert_eq!(summary.failed_count, 1);  // 1個エラー（破損ファイル）

    // 変換されたWebP画像が存在すること
    assert!(out_dir.join("image1.webp").exists());
    assert!(out_dir.join("image2.webp").exists());

    // リサイズが適用されていること (300x300 -> 200x200)
    let converted1 = image::open(out_dir.join("image1.webp")).unwrap();
    assert_eq!(converted1.width(), 200);
    assert_eq!(converted1.height(), 200);
}
