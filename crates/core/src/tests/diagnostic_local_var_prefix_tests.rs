use crate::diagnostics::{diagnose, DiagnosticConfig};
use crate::lexer::Lexer;
use crate::parser::Parser;

fn local_prefix_config() -> DiagnosticConfig {
    DiagnosticConfig {
        check_file_header: false,
        check_function_format: false,
        check_type_safety: false,
        check_storage_class_order: false,
        check_macro_parentheses: false,
        check_global_var_naming: false,
        check_global_var_type_prefix: false,
        check_local_var_type_prefix: true,
        ..DiagnosticConfig::default()
    }
}

#[test]
fn warns_on_missing_prefix() {
    let source = r#"
        typedef unsigned char VU8;
        void foo() {
            VU8 counter;
        }
    "#;

    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();

    let diagnostics = diagnose(&tu, &local_prefix_config());

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "CGH010");
    assert!(diagnostics[0].message.contains("u8_"));
    assert!(diagnostics[0].message.contains("counter"));
}

#[test]
fn passes_when_prefix_is_correct() {
    let source = r#"
        typedef unsigned int VU32;
        void foo() {
            VU32 u32_value;
        }
    "#;

    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();

    let diagnostics = diagnose(&tu, &local_prefix_config());

    assert!(diagnostics.is_empty());
}

#[test]
fn handles_nested_blocks_and_for_init() {
    let source = r#"
        typedef signed short VS16;
        void foo() {
            {
                VS16 wrong;
            }
            for (VS16 i = 0; i < 3; i++) {
                VS16 s16_ok;
            }
        }
    "#;

    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();

    let diagnostics = diagnose(&tu, &local_prefix_config());

    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics.iter().all(|d| d.code == "CGH010"));
}
