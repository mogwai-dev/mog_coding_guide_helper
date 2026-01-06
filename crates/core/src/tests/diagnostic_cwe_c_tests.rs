use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::diagnostics::{diagnose_with_source, DiagnosticConfig, DiagnosticCode};

#[test]
fn test_cwe_c_check_enabled() {
    let input = r#"
int main() {
    return 0;
}
"#;
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_file_header = false;
    config.check_function_format = false;
    config.check_cwe_c = true;
    
    let diagnostics = diagnose_with_source(&tu, &config, input);
    
    // Currently no CWE-C rules are implemented, so should return empty
    let cwe_c_diagnostics: Vec<_> = diagnostics.iter()
        .filter(|d| matches!(d.code, DiagnosticCode::CweC(_)))
        .collect();
    
    assert_eq!(cwe_c_diagnostics.len(), 0, "CWE-C rules not yet implemented");
}

#[test]
fn test_cwe_c_check_disabled() {
    let input = r#"
int main() {
    return 0;
}
"#;
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_file_header = false;
    config.check_function_format = false;
    config.check_cwe_c = false;
    
    let diagnostics = diagnose_with_source(&tu, &config, input);
    
    // No CWE-C diagnostics should be produced when disabled
    let cwe_c_diagnostics: Vec<_> = diagnostics.iter()
        .filter(|d| matches!(d.code, DiagnosticCode::CweC(_)))
        .collect();
    
    assert_eq!(cwe_c_diagnostics.len(), 0);
}
