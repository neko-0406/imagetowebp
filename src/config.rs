use std::path::PathBuf;

/// 画像変換の設定オプション
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertConfig {
    /// 変換対象のファイルまたはフォルダのパス一覧
    pub input_paths: Vec<PathBuf>,
    /// 保存先ディレクトリ（Noneの場合は元画像と同じディレクトリに保存）
    pub output_dir: Option<PathBuf>,
    /// 画質 (1..=100, デフォルト: 80)
    pub quality: u8,
    /// Lossless (可逆圧縮) モードを使用するかどうか
    pub lossless: bool,
    /// 画像の長辺の最大サイズ（ピクセル）。Noneの場合はリサイズなし
    pub max_dimension: Option<u32>,
    /// 同名WebPファイルが存在する場合に上書きするかどうか
    pub overwrite: bool,
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
        }
    }
}

impl ConvertConfig {
    /// 設定の妥当性検証と正規化
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
}
