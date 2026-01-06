use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::diagnostics::{diagnose, DiagnosticConfig, DiagnosticSeverity, DiagnosticCode};

#[test]
fn test_void_variable_error() {
    let input = "void x;";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_file_header = false;
    config.check_function_format = false;
    
    let diagnostics = diagnose(&tu, &config);
    
    // Should have error for void variable
    assert!(diagnostics.iter().any(|d| 
        d.severity == DiagnosticSeverity::Error 
        && matches!(d.code, DiagnosticCode::Custom(ref code) if code == "CGH101")
        && d.message.contains("void型にできません")
    ));
}

#[test]
fn test_void_pointer_allowed() {
    let input = "void *ptr;";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_file_header = false;
    config.check_function_format = false;
    
    let diagnostics = diagnose(&tu, &config);
    
    // Should NOT have error for void pointer
    assert!(!diagnostics.iter().any(|d| 
        d.severity == DiagnosticSeverity::Error 
        && matches!(d.code, DiagnosticCode::Custom(ref code) if code == "CGH101")
    ));
}

#[test]
fn test_triple_pointer_warning() {
    let input = "int ***ptr;";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_file_header = false;
    config.check_function_format = false;
    
    let diagnostics = diagnose(&tu, &config);
    
    // Should have warning for triple pointer
    assert!(diagnostics.iter().any(|d| 
        d.severity == DiagnosticSeverity::Warning 
        && matches!(d.code, DiagnosticCode::Custom(ref code) if code == "CGH102")
        && d.message.contains("3段階のポインタ")
    ));
}

#[test]
fn test_double_pointer_no_warning() {
    let input = "int **ptr;";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_file_header = false;
    config.check_function_format = false;
    
    let diagnostics = diagnose(&tu, &config);
    
    // Should NOT have warning for double pointer
    assert!(!diagnostics.iter().any(|d| 
        matches!(d.code, DiagnosticCode::Custom(ref code) if code == "CGH102")
    ));
}

#[test]
fn test_type_safety_disabled() {
    let input = "void x;";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_file_header = false;
    config.check_function_format = false;
    config.check_type_safety = false;
    
    let diagnostics = diagnose(&tu, &config);
    
    // Should NOT have type safety diagnostics when disabled
    assert!(!diagnostics.iter().any(|d| matches!(d.code, DiagnosticCode::Custom(ref code) if code.starts_with("CGH1"))));
}
