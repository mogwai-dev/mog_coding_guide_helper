use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::diagnostics::{diagnose, DiagnosticConfig, DiagnosticSeverity};

fn type_prefix_config(enable: bool) -> DiagnosticConfig {
    DiagnosticConfig {
        check_file_header: false,
        check_function_format: false,
        check_type_safety: false,
        check_storage_class_order: false,
        check_macro_parentheses: false,
        check_global_var_naming: false,
        check_global_var_type_prefix: enable,
        check_local_var_type_prefix: false,
        ..DiagnosticConfig::default()
    }
}

/// VU8型のプレフィックステスト
#[test]
fn test_type_prefix_vu8() {
    let source = "typedef unsigned char VU8;\nVU8 counter;";
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    println!("Items: {}", tu.items.len());
    for item in &tu.items {
        match item {
            crate::ast::Item::VarDecl { var_name, text, .. } => {
                println!("VarDecl: {} | text: {}", var_name, text);
            }
            crate::ast::Item::TypedefDecl { .. } => {
                println!("TypedefDecl");
            }
            _ => {
                println!("Other item: {:?}", item);
            }
        }
    }
    
    let config = type_prefix_config(true);
    
    let diagnostics = diagnose(&tu, &config);
    println!("Diagnostics: {}", diagnostics.len());
    for d in &diagnostics {
        println!("  {}: {}", d.code, d.message);
    }
    
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "CGH007");
    assert!(diagnostics[0].message.contains("VU8_"));
    assert!(diagnostics[0].message.contains("counter"));
}

/// VU16型のプレフィックステスト
#[test]
fn test_type_prefix_vu16() {
    let source = r#"
typedef unsigned short VU16;
VU16 value;
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let config = type_prefix_config(true);
    
    let diagnostics = diagnose(&tu, &config);
    
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "CGH007");
    assert!(diagnostics[0].message.contains("VU16_"));
}

/// VS32型のプレフィックステスト
#[test]
fn test_type_prefix_vs32() {
    let source = r#"
typedef signed int VS32;
VS32 result;
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let config = type_prefix_config(true);
    
    let diagnostics = diagnose(&tu, &config);
    
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "CGH007");
    assert!(diagnostics[0].message.contains("VS32_"));
}

/// CU8型のプレフィックステスト
#[test]
fn test_type_prefix_cu8() {
    let source = r#"
typedef const unsigned char CU8;
CU8 byte;
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let config = type_prefix_config(true);
    
    let diagnostics = diagnose(&tu, &config);
    
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "CGH007");
    assert!(diagnostics[0].message.contains("CU8_"));
}

/// CS64型のプレフィックステスト
#[test]
fn test_type_prefix_cs64() {
    let source = r#"
typedef const signed long long CS64;
CS64 long_val;
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let config = type_prefix_config(true);
    
    let diagnostics = diagnose(&tu, &config);
    
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "CGH007");
    assert!(diagnostics[0].message.contains("CS64_"));
}

/// 正しいプレフィックスの場合は警告なし
#[test]
fn test_type_prefix_correct() {
    let source = r#"
typedef unsigned char VU8;
typedef unsigned short VU16;
typedef signed int VS32;
typedef const unsigned char CU8;
VU8 VU8_counter;
VU16 VU16_value;
VS32 VS32_result;
CU8 CU8_byte;
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let config = type_prefix_config(true);
    
    let diagnostics = diagnose(&tu, &config);
    
    assert_eq!(diagnostics.len(), 0);
}

/// 複数の変数の混在テスト
#[test]
fn test_type_prefix_mixed() {
    let source = r#"
typedef unsigned char VU8;
typedef unsigned short VU16;
typedef signed int VS32;
typedef const unsigned int CU32;
VU8 VU8_valid;
VU16 invalid;
VS32 VS32_ok;
CU32 wrong_name;
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let config = type_prefix_config(true);
    
    let diagnostics = diagnose(&tu, &config);
    
    assert_eq!(diagnostics.len(), 2);
    assert!(diagnostics.iter().all(|d| d.code == "CGH007"));
}

/// 該当しない型の場合は警告なし
#[test]
fn test_type_prefix_non_target_type() {
    let source = r#"
int counter;
unsigned long value;
char* str;
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let config = type_prefix_config(true);
    
    let diagnostics = diagnose(&tu, &config);
    
    assert_eq!(diagnostics.len(), 0);
}

/// 設定で無効化されている場合
#[test]
fn test_type_prefix_config_disabled() {
    let source = r#"
VU8 counter;
VU16 value;
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let config = type_prefix_config(false);
    
    let diagnostics = diagnose(&tu, &config);
    
    assert_eq!(diagnostics.len(), 0);
}

/// 全ての型のプレフィックステスト
#[test]
fn test_type_prefix_all_types() {
    let source = r#"
typedef unsigned char VU8;
typedef unsigned short VU16;
typedef unsigned int VU32;
typedef unsigned long long VU64;
typedef signed char VS8;
typedef signed short VS16;
typedef signed int VS32;
typedef signed long long VS64;
typedef const unsigned char CU8;
typedef const unsigned short CU16;
typedef const unsigned int CU32;
typedef const unsigned long long CU64;
typedef const signed char CS8;
typedef const signed short CS16;
typedef const signed int CS32;
typedef const signed long long CS64;
VU8 VU8_a;
VU16 VU16_b;
VU32 VU32_c;
VU64 VU64_d;
VS8 VS8_e;
VS16 VS16_f;
VS32 VS32_g;
VS64 VS64_h;
CU8 CU8_i;
CU16 CU16_j;
CU32 CU32_k;
CU64 CU64_l;
CS8 CS8_m;
CS16 CS16_n;
CS32 CS32_o;
CS64 CS64_p;
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let config = type_prefix_config(true);
    
    let diagnostics = diagnose(&tu, &config);
    
    // 全て正しいプレフィックスなので警告なし
    assert_eq!(diagnostics.len(), 0);
}

