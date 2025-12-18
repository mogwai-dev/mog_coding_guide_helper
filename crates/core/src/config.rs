use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;

/// プロジェクト設定ファイル (coding-guide.toml)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ProjectConfig {
    pub diagnostics: DiagnosticsConfig,
    pub file_header: FileHeaderConfig,
    pub formatting: FormattingConfig,
    pub preprocessor: PreprocessorConfig,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        ProjectConfig {
            diagnostics: DiagnosticsConfig::default(),
            file_header: FileHeaderConfig::default(),
            formatting: FormattingConfig::default(),
            preprocessor: PreprocessorConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct DiagnosticsConfig {
    pub check_file_header: bool,
    pub check_function_format: bool,
    pub check_type_safety: bool,
    pub check_storage_class_order: bool,
    pub check_macro_parentheses: bool,
    pub check_global_var_naming: bool,
    pub check_global_var_type_prefix: bool,
    pub check_local_var_type_prefix: bool,
    pub check_preprocessor_indent: bool,
    pub check_indent_style: bool,
}

impl Default for DiagnosticsConfig {
    fn default() -> Self {
        DiagnosticsConfig {
            check_file_header: true,
            check_function_format: true,
            check_type_safety: true,
            check_storage_class_order: true,
            check_macro_parentheses: true,
            check_global_var_naming: true,
            check_global_var_type_prefix: true,
            check_local_var_type_prefix: true,
            check_preprocessor_indent: true,
            check_indent_style: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct FileHeaderConfig {
    pub required_fields: Vec<String>,
    pub template: Option<String>,
}

impl Default for FileHeaderConfig {
    fn default() -> Self {
        FileHeaderConfig {
            required_fields: vec![
                "Author".to_string(),
                "Date".to_string(),
                "Purpose".to_string(),
            ],
            template: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IndentStyle {
    Tabs,
    Spaces,
}

impl Default for IndentStyle {
    fn default() -> Self {
        IndentStyle::Spaces
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct FormattingConfig {
    pub add_file_header: bool,
    pub use_tabs: bool,  // 4スペースをタブに変換
    pub indent_style: IndentStyle,  // インデントにタブまたはスペースを使用
    pub indent_width: usize,  // スペース使用時のインデント幅
}

impl Default for FormattingConfig {
    fn default() -> Self {
        FormattingConfig {
            add_file_header: true,
            use_tabs: false,  // デフォルトはスペースのまま
            indent_style: IndentStyle::Spaces,
            indent_width: 4,
        }
    }
}

/// プリプロセッサ設定
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct PreprocessorConfig {
    /// 定義済みマクロのリスト（例: ["_WIN32", "DEBUG"]）
    pub defines: Vec<String>,
    /// ヘッダーファイル検索パス（例: ["./include", "C:/SDK/include"]）
    pub include_paths: Vec<PathBuf>,
}

impl Default for PreprocessorConfig {
    fn default() -> Self {
        PreprocessorConfig {
            defines: Vec::new(),
            include_paths: vec![PathBuf::from(".")],
        }
    }
}

impl PreprocessorConfig {
    /// 指定されたマクロが定義されているかチェック
    pub fn is_macro_defined(&self, macro_name: &str) -> bool {
        self.defines.iter().any(|def| {
            // "MACRO" または "MACRO=value" の形式に対応
            def == macro_name || def.starts_with(&format!("{}=", macro_name))
        })
    }
}

impl ProjectConfig {
    /// 設定ファイルを読み込む
    /// 見つからない場合はデフォルト設定を返す
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: ProjectConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// プロジェクトルートから設定ファイルを検索
    /// 現在のディレクトリから親ディレクトリへ遡って "coding-guide.toml" を探す
    pub fn find_and_load<P: AsRef<Path>>(start_dir: P) -> Self {
        let mut current = start_dir.as_ref().to_path_buf();
        
        loop {
            let config_path = current.join("coding-guide.toml");
            if config_path.exists() {
                match Self::load_from_file(&config_path) {
                    Ok(config) => {
                        eprintln!("Loaded config from: {}", config_path.display());
                        return config;
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse {}: {}", config_path.display(), e);
                        eprintln!("Using default configuration");
                        return Self::default();
                    }
                }
            }
            
            // 親ディレクトリへ
            if !current.pop() {
                // ルートディレクトリに到達
                break;
            }
        }
        
        // 見つからない場合はデフォルト
        Self::default()
    }

    /// DiagnosticConfigに変換
    pub fn to_diagnostic_config(&self) -> crate::diagnostics::DiagnosticConfig {
        crate::diagnostics::DiagnosticConfig {
            check_file_header: self.diagnostics.check_file_header,
            check_function_format: self.diagnostics.check_function_format,
            check_type_safety: self.diagnostics.check_type_safety,
            check_storage_class_order: self.diagnostics.check_storage_class_order,
            check_macro_parentheses: self.diagnostics.check_macro_parentheses,
            check_global_var_naming: self.diagnostics.check_global_var_naming,
            check_global_var_type_prefix: self.diagnostics.check_global_var_type_prefix,
            check_local_var_type_prefix: self.diagnostics.check_local_var_type_prefix,
            check_preprocessor_indent: self.diagnostics.check_preprocessor_indent,
            check_indent_style: self.diagnostics.check_indent_style,
            indent_style: self.formatting.indent_style.clone(),
            indent_width: self.formatting.indent_width,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ProjectConfig::default();
        assert!(config.diagnostics.check_file_header);
        assert!(config.diagnostics.check_function_format);
        assert_eq!(config.file_header.required_fields.len(), 3);
    }

    #[test]
    fn test_parse_toml() {
        let toml_str = r#"
[diagnostics]
check_file_header = false
check_function_format = true

[file_header]
required_fields = ["Author", "Date"]

[formatting]
add_file_header = false
"#;
        
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert!(!config.diagnostics.check_file_header);
        assert!(config.diagnostics.check_function_format);
        assert_eq!(config.file_header.required_fields.len(), 2);
        assert!(!config.formatting.add_file_header);
    }

    #[test]
    fn test_partial_config() {
        // 一部のみ指定した場合、残りはデフォルト値
        let toml_str = r#"
[diagnostics]
check_file_header = false
"#;
        
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert!(!config.diagnostics.check_file_header);
        assert!(config.diagnostics.check_function_format); // デフォルト値
    }

    #[test]
    fn test_new_diagnostic_options() {
        // CGH007とCGH008の設定読み込みテスト
        let toml_str = r#"
[diagnostics]
check_global_var_type_prefix = false
check_local_var_type_prefix = false
check_preprocessor_indent = false
"#;
        
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert!(!config.diagnostics.check_global_var_type_prefix);
        assert!(!config.diagnostics.check_local_var_type_prefix);
        assert!(!config.diagnostics.check_preprocessor_indent);
        
        // デフォルト値の確認
        let default_config = ProjectConfig::default();
        assert!(default_config.diagnostics.check_global_var_type_prefix);
        assert!(default_config.diagnostics.check_local_var_type_prefix);
        assert!(default_config.diagnostics.check_preprocessor_indent);
    }
}
