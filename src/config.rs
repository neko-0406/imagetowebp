use std::path::PathBuf;

/// 画像変換の設定オプション
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertConfig {
    /// 変換対象のファイルまたはフォルダのパス一覧
    pub input_paths: Vec<PathBuf>,
    /// 保存先ディレクトリ（Noneの場合は元画像と同じディレクトリに保存）
    pub output_dir: Option<PathBuf>,
    /// 画質 (1..=100, デフォルト: 80)。lossless = true の場合は使用されない。
    pub quality: u8,
    /// Lossless (可逆圧縮) モードを使用するかどうか。true の場合、quality は無視される。
    pub lossless: bool,
    /// 画像の長辺の最大サイズ（ピクセル）。Noneの場合はリサイズなし
    pub max_dimension: Option<u32>,
    /// 同名WebPファイルが存在する場合に上書きするかどうか
    pub overwrite: bool,
    /// 出力先ディレクトリに入力の相対ディレクトリ構造を保持するかどうか（同名ファイル衝突防止）
    pub preserve_structure: bool,
}

impl Default for ConvertConfig {
    fn default() -> Self {
        Self {
            input_paths: Vec::new(),
            output_dir: None,
            quality: 80,
            lossless: false,
            max_dimension: None,
            overwrite: false,
            preserve_structure: false,
        }
    }
}

impl ConvertConfig {
    /// 設定の妥当性検証
    pub fn validate(&self) -> Result<(), String> {
        if self.input_paths.is_empty() {
            return Err("At least one input file or directory must be specified.".to_string());
        }
        if self.quality == 0 || self.quality > 100 {
            return Err("Quality must be between 1 and 100.".to_string());
        }
        if let Some(dim) = self.max_dimension {
            if dim == 0 {
                return Err("Max dimension must be greater than 0.".to_string());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation_valid() {
        let config = ConvertConfig {
            input_paths: vec![PathBuf::from("test.jpg")],
            quality: 80,
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_empty_inputs() {
        let config = ConvertConfig::default();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_invalid_quality() {
        let config = ConvertConfig {
            input_paths: vec![PathBuf::from("test.jpg")],
            quality: 105,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_zero_quality() {
        let config = ConvertConfig {
            input_paths: vec![PathBuf::from("test.jpg")],
            quality: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_preserve_structure_default_false() {
        let config = ConvertConfig::default();
        assert!(!config.preserve_structure);
    }
}
