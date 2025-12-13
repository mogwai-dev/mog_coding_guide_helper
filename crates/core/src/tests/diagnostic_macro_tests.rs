use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::diagnostics::{diagnose, DiagnosticConfig, DiagnosticSeverity};

#[test]
fn test_macro_without_parentheses() {
    let source = r#"
#define MAX 10 + 20
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_file_header = false;  // ファイルヘッダーチェックを無効化
    let diagnostics = diagnose(&tu, &config);
    
    // CGH005の警告が出ることを確認
    let macro_warnings: Vec<_> = diagnostics.iter().filter(|d| d.code == "CGH005").collect();
    assert_eq!(macro_warnings.len(), 1);
    assert_eq!(macro_warnings[0].severity, DiagnosticSeverity::Warning);
    assert!(macro_warnings[0].message.contains("MAX"));
    assert!(macro_warnings[0].message.contains("10 + 20"));
}

#[test]
fn test_macro_with_parentheses() {
    let source = r#"
#define MAX (10 + 20)
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let config = DiagnosticConfig::default();
    let diagnostics = diagnose(&tu, &config);
    
    // 警告が出ないことを確認
    let macro_warnings: Vec<_> = diagnostics.iter().filter(|d| d.code == "CGH005").collect();
    assert_eq!(macro_warnings.len(), 0);
}

#[test]
fn test_macro_simple_literal() {
    let source = r#"
#define MAX 100
#define PI 3.14
#define HEX 0xFF
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let config = DiagnosticConfig::default();
    let diagnostics = diagnose(&tu, &config);
    
    // 単純なリテラルは警告が出ない
    let macro_warnings: Vec<_> = diagnostics.iter().filter(|d| d.code == "CGH005").collect();
    assert_eq!(macro_warnings.len(), 0);
}

#[test]
fn test_macro_function_like() {
    let source = r#"
#define ADD(a, b) a + b
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let config = DiagnosticConfig::default();
    let diagnostics = diagnose(&tu, &config);
    
    // 関数マクロは警告対象外（引数に括弧が必要）
    let macro_warnings: Vec<_> = diagnostics.iter().filter(|d| d.code == "CGH005").collect();
    assert_eq!(macro_warnings.len(), 0);
}

#[test]
fn test_macro_multiple_operators() {
    let source = r#"
#define CALC 10 + 20 * 30
#define SHIFT 1 << 8
#define BIT 0xFF & 0x0F
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_file_header = false;  // ファイルヘッダーチェックを無効化
    let diagnostics = diagnose(&tu, &config);
    
    // 3つの警告が出る
    let macro_warnings: Vec<_> = diagnostics.iter().filter(|d| d.code == "CGH005").collect();
    assert_eq!(macro_warnings.len(), 3);
}

#[test]
fn test_macro_config_disabled() {
    let source = r#"
#define MAX 10 + 20
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_macro_parentheses = false;
    let diagnostics = diagnose(&tu, &config);
    
    // チェックが無効なので警告は出ない
    let macro_warnings: Vec<_> = diagnostics.iter().filter(|d| d.code == "CGH005").collect();
    assert_eq!(macro_warnings.len(), 0);
}

#[test]
fn test_macro_in_conditional_block() {
    let source = r#"
#ifdef DEBUG
#define LOG_LEVEL 1 + 2
#endif
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let config = DiagnosticConfig::default();
    let diagnostics = diagnose(&tu, &config);
    
    // 条件コンパイルブロック内のマクロもチェックされる
    let macro_warnings: Vec<_> = diagnostics.iter().filter(|d| d.code == "CGH005").collect();
    assert_eq!(macro_warnings.len(), 1);
}
