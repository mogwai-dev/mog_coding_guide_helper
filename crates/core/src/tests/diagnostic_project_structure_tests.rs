use crate::ast::TranslationUnit;
use crate::diagnostics::{diagnose, DiagnosticConfig, DiagnosticCode};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::trivia::Trivia;
use std::fs;
use tempfile::tempdir;

#[test]
fn warns_when_include_and_src_missing() {
    let temp = tempdir().unwrap();
    let tu = TranslationUnit {
        items: Vec::new(),
        leading_trivia: Trivia::empty(),
    };

    let mut config = DiagnosticConfig::default();
    config.project_root = Some(temp.path().to_path_buf());

    let diagnostics = diagnose(&tu, &config);
    assert!(diagnostics.iter().any(|d| matches!(d.code, DiagnosticCode::Custom(ref code) if code == "CGH011")));
    assert!(diagnostics.iter().any(|d| matches!(d.code, DiagnosticCode::Custom(ref code) if code == "CGH012")));
}

#[test]
fn no_warning_when_include_and_src_exist() {
    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join("include")).unwrap();
    fs::create_dir_all(temp.path().join("src")).unwrap();

    let tu = TranslationUnit {
        items: Vec::new(),
        leading_trivia: Trivia::empty(),
    };

    let mut config = DiagnosticConfig::default();
    config.project_root = Some(temp.path().to_path_buf());

    let diagnostics = diagnose(&tu, &config);
    assert!(diagnostics.iter().all(|d| !matches!(d.code, DiagnosticCode::Custom(ref code) if code == "CGH011" || code == "CGH012")));
}

#[test]
fn skips_diagnostics_for_excluded_paths() {
    let temp = tempdir().unwrap();
    let excluded_dir = temp.path().join("excluded");
    fs::create_dir_all(&excluded_dir).unwrap();

    let file_path = excluded_dir.join("test.c");
    fs::write(&file_path, "int globalVar;\n").unwrap();

    let contents = fs::read_to_string(&file_path).unwrap();
    let mut parser = Parser::new(Lexer::new(&contents));
    parser.set_current_file_dir(&excluded_dir);
    let tu = parser.parse();

    let mut base_config = DiagnosticConfig::default();
    base_config.project_root = Some(temp.path().to_path_buf());
    base_config.source_path = Some(file_path.clone());

    let diagnostics = diagnose(&tu, &base_config);
    assert!(diagnostics.iter().any(|d| matches!(d.code, DiagnosticCode::Custom(ref code) if code == "CGH006")));

    let mut excluded_config = base_config.clone();
    excluded_config.exclude_paths = vec![excluded_dir];
    let skipped = diagnose(&tu, &excluded_config);
    assert!(skipped.is_empty());
}
