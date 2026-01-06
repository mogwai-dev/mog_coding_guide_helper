use crate::config::{IndentStyle, ProjectConfig};
use crate::diagnostics::{diagnose_with_source, DiagnosticConfig, DiagnosticCode};
use crate::lexer::Lexer;
use crate::parser::Parser;

#[test]
fn test_indent_style_tabs_expected() {
    let source = r#"
void foo() {
	int x = 1;
	if (x > 0) {
		x++;
	}
}
"#;

    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_indent_style = true;
    config.indent_style = IndentStyle::Tabs;
    
    let diagnostics = diagnose_with_source(&tu, &config, source);
    let cgh009_diagnostics: Vec<_> = diagnostics.iter().filter(|d| matches!(d.code, DiagnosticCode::Custom(ref code) if code == "CGH009")).collect();
    
    // タブを使用しているので警告なし
    assert_eq!(cgh009_diagnostics.len(), 0);
}

#[test]
fn test_indent_style_tabs_expected_but_spaces_used() {
    let source = r#"
void foo() {
    int x = 1;
    if (x > 0) {
        x++;
    }
}
"#;

    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_indent_style = true;
    config.indent_style = IndentStyle::Tabs;
    
    let diagnostics = diagnose_with_source(&tu, &config, source);
    let cgh009_diagnostics: Vec<_> = diagnostics.iter().filter(|d| matches!(d.code, DiagnosticCode::Custom(ref code) if code == "CGH009")).collect();
    
    // スペースを使用しているので警告が出る（4行分: lines 3,4,5,6）
    assert_eq!(cgh009_diagnostics.len(), 4);
    assert!(cgh009_diagnostics[0].message.contains("タブを使用すべきところでスペースが使われています"));
}

#[test]
fn test_indent_style_spaces_expected() {
    let source = r#"
void foo() {
    int x = 1;
    if (x > 0) {
        x++;
    }
}
"#;

    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_indent_style = true;
    config.indent_style = IndentStyle::Spaces;
    config.indent_width = 4;
    
    let diagnostics = diagnose_with_source(&tu, &config, source);
    let cgh009_diagnostics: Vec<_> = diagnostics.iter().filter(|d| matches!(d.code, DiagnosticCode::Custom(ref code) if code == "CGH009")).collect();
    
    // スペースを使用しているので警告なし
    assert_eq!(cgh009_diagnostics.len(), 0);
}

#[test]
fn test_indent_style_spaces_expected_but_tabs_used() {
    let source = r#"
void foo() {
	int x = 1;
	if (x > 0) {
		x++;
	}
}
"#;

    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_indent_style = true;
    config.indent_style = IndentStyle::Spaces;
    config.indent_width = 4;
    
    let diagnostics = diagnose_with_source(&tu, &config, source);
    let cgh009_diagnostics: Vec<_> = diagnostics.iter().filter(|d| matches!(d.code, DiagnosticCode::Custom(ref code) if code == "CGH009")).collect();
    
    // タブを使用しているので警告が出る（4行分: lines 3,4,5,6）
    assert_eq!(cgh009_diagnostics.len(), 4);
    assert!(cgh009_diagnostics[0].message.contains("スペース（4文字単位）を使用すべきところでタブが使われています"));
}

#[test]
fn test_indent_style_mixed_tabs_and_spaces() {
    let source = "void foo() {\n\t    int x = 1;\n}\n";

    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_indent_style = true;
    config.indent_style = IndentStyle::Spaces;
    config.indent_width = 4;
    
    let diagnostics = diagnose_with_source(&tu, &config, source);
    let cgh009_diagnostics: Vec<_> = diagnostics.iter().filter(|d| matches!(d.code, DiagnosticCode::Custom(ref code) if code == "CGH009")).collect();
    
    // 混在しているので警告が出る
    assert_eq!(cgh009_diagnostics.len(), 1);
    assert!(cgh009_diagnostics[0].message.contains("タブとスペースが混在しています"));
}

#[test]
fn test_indent_style_config_disabled() {
    let source = r#"
void foo() {
	int x = 1;
}
"#;

    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_indent_style = false;  // チェック無効
    config.indent_style = IndentStyle::Spaces;
    
    let diagnostics = diagnose_with_source(&tu, &config, source);
    let cgh009_diagnostics: Vec<_> = diagnostics.iter().filter(|d| matches!(d.code, DiagnosticCode::Custom(ref code) if code == "CGH009")).collect();
    
    // チェックが無効なので警告なし
    assert_eq!(cgh009_diagnostics.len(), 0);
}
