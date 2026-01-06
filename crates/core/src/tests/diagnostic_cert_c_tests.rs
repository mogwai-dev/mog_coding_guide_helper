use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::diagnostics::{diagnose_with_source, DiagnosticConfig, DiagnosticCode};

#[test]
fn test_cert_c_check_enabled() {
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
    config.check_cert_c = true;
    
    let diagnostics = diagnose_with_source(&tu, &config, input);
    
    // Currently no CERT-C rules are implemented, so should return empty
    let cert_c_diagnostics: Vec<_> = diagnostics.iter()
        .filter(|d| matches!(d.code, DiagnosticCode::CertC(_)))
        .collect();
    
    assert_eq!(cert_c_diagnostics.len(), 0, "CERT-C rules not yet implemented");
}

#[test]
fn test_cert_c_check_disabled() {
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
    config.check_cert_c = false;
    
    let diagnostics = diagnose_with_source(&tu, &config, input);
    
    // No CERT-C diagnostics should be produced when disabled
    let cert_c_diagnostics: Vec<_> = diagnostics.iter()
        .filter(|d| matches!(d.code, DiagnosticCode::CertC(_)))
        .collect();
    
    assert_eq!(cert_c_diagnostics.len(), 0);
}
