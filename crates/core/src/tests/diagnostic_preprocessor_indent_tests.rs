use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::diagnostics::{diagnose, DiagnosticConfig, DiagnosticCode};

#[test]
fn test_preprocessor_indent_include() {
    let input = "  #include <stdio.h>\nint main() { return 0; }";
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    // TUのアイテムを確認
    eprintln!("TU items: {}", tu.items.len());
    for (i, item) in tu.items.iter().enumerate() {
        eprintln!("Item {}: {:?}", i, item);
        if let crate::ast::Item::Include { span, .. } = item {
            eprintln!("  Include span: start_column={}", span.start_column);
        }
    }
    
    let config = DiagnosticConfig::default();
    let diagnostics = diagnose(&tu, &config);
    
    // デバッグ出力
    eprintln!("Diagnostics: {}", diagnostics.len());
    for d in &diagnostics {
        eprintln!("  {:?}: {} at line {}, col {}", d.code, d.message, d.span.start_line, d.span.start_column);
    }
    
    // CGH008のみをフィルタリング
    let cgh008_diagnostics: Vec<_> = diagnostics.iter()
        .filter(|d| matches!(d.code, DiagnosticCode::Custom(ref code) if code == "CGH008"))
        .collect();
    
    assert_eq!(cgh008_diagnostics.len(), 1);
    assert!(cgh008_diagnostics[0].message.contains("2文字"));
}

#[test]
fn test_preprocessor_indent_define() {
    let input = "    #define MAX 100\nint x;";
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let config = DiagnosticConfig::default();
    let diagnostics = diagnose(&tu, &config);
    
    // CGH008のみをフィルタリング
    let cgh008_diagnostics: Vec<_> = diagnostics.iter()
        .filter(|d| matches!(d.code, DiagnosticCode::Custom(ref code) if code == "CGH008"))
        .collect();
    
    assert_eq!(cgh008_diagnostics.len(), 1);
    assert!(cgh008_diagnostics[0].message.contains("4文字"));
}

#[test]
fn test_preprocessor_indent_ifdef() {
    let input = " #ifdef DEBUG\nint x;\n#endif";
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let config = DiagnosticConfig::default();
    let diagnostics = diagnose(&tu, &config);
    
    // CGH008のみをフィルタリング
    let cgh008_diagnostics: Vec<_> = diagnostics.iter()
        .filter(|d| matches!(d.code, DiagnosticCode::Custom(ref code) if code == "CGH008"))
        .collect();
    
    assert_eq!(cgh008_diagnostics.len(), 1);
    assert!(cgh008_diagnostics[0].message.contains("1文字"));
}

#[test]
fn test_preprocessor_indent_nested_ifdef() {
    let input = "#ifdef WIN32\n  #ifdef DEBUG\n  int x;\n  #endif\n#endif";
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let config = DiagnosticConfig::default();
    let diagnostics = diagnose(&tu, &config);
    
    // CGH008のみをフィルタリング（内側のifdefとendifの2つが警告）
    let cgh008_diagnostics: Vec<_> = diagnostics.iter()
        .filter(|d| matches!(d.code, DiagnosticCode::Custom(ref code) if code == "CGH008"))
        .collect();
    
    assert_eq!(cgh008_diagnostics.len(), 2);
    assert!(cgh008_diagnostics.iter().all(|d| matches!(d.code, DiagnosticCode::Custom(ref code) if code == "CGH008")));
}

#[test]
fn test_preprocessor_indent_correct() {
    let input = "#include <stdio.h>\n#define MAX 100\n#ifdef DEBUG\nint x;\n#endif";
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let config = DiagnosticConfig::default();
    let diagnostics = diagnose(&tu, &config);
    
    // 正しい位置にあるので警告なし
    let preprocessor_warnings: Vec<_> = diagnostics.iter()
        .filter(|d| matches!(d.code, DiagnosticCode::Custom(ref code) if code == "CGH008"))
        .collect();
    assert_eq!(preprocessor_warnings.len(), 0);
}

#[test]
fn test_preprocessor_indent_mixed() {
    let input = "#include <stdio.h>\n  #define MAX 100\n#ifdef DEBUG\n    #define MIN 0\n#endif";
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let config = DiagnosticConfig::default();
    let diagnostics = diagnose(&tu, &config);
    
    // 2つのdefineが警告されるはず
    let preprocessor_warnings: Vec<_> = diagnostics.iter()
        .filter(|d| matches!(d.code, DiagnosticCode::Custom(ref code) if code == "CGH008"))
        .collect();
    assert_eq!(preprocessor_warnings.len(), 2);
}

#[test]
fn test_preprocessor_indent_config_disabled() {
    let input = "  #include <stdio.h>\n    #define MAX 100";
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_preprocessor_indent = false;
    let diagnostics = diagnose(&tu, &config);
    
    // チェックが無効なので警告なし
    let preprocessor_warnings: Vec<_> = diagnostics.iter()
        .filter(|d| matches!(d.code, DiagnosticCode::Custom(ref code) if code == "CGH008"))
        .collect();
    assert_eq!(preprocessor_warnings.len(), 0);
}
