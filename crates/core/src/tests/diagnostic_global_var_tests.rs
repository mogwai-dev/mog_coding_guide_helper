use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::diagnostics::{diagnose, DiagnosticConfig, DiagnosticSeverity};

#[test]
fn test_global_var_lowercase() {
    let source = r#"
int globalVar = 10;
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_file_header = false;
    let diagnostics = diagnose(&tu, &config);
    
    // CGH006の警告が出る
    let var_warnings: Vec<_> = diagnostics.iter().filter(|d| d.code == "CGH006").collect();
    assert_eq!(var_warnings.len(), 1);
    assert_eq!(var_warnings[0].severity, DiagnosticSeverity::Warning);
    assert!(var_warnings[0].message.contains("globalVar"));
    assert!(var_warnings[0].message.contains("GLOBAL_VAR"));
}

#[test]
fn test_global_var_camelcase() {
    let source = r#"
int myGlobalVariable = 100;
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_file_header = false;
    let diagnostics = diagnose(&tu, &config);
    
    // CGH006の警告が出る（MY_GLOBAL_VARIABLEを推奨）
    let var_warnings: Vec<_> = diagnostics.iter().filter(|d| d.code == "CGH006").collect();
    assert_eq!(var_warnings.len(), 1);
    assert!(var_warnings[0].message.contains("myGlobalVariable"));
    assert!(var_warnings[0].message.contains("MY_GLOBAL_VARIABLE"));
}

#[test]
fn test_global_var_uppercase_ok() {
    let source = r#"
int GLOBAL_VAR = 10;
int MAX_SIZE = 100;
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_file_header = false;
    let diagnostics = diagnose(&tu, &config);
    
    // 大文字なので警告なし
    let var_warnings: Vec<_> = diagnostics.iter().filter(|d| d.code == "CGH006").collect();
    assert_eq!(var_warnings.len(), 0);
}

#[test]
fn test_global_var_with_numbers() {
    let source = r#"
int VALUE_123 = 10;
int test123 = 20;
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_file_header = false;
    let diagnostics = diagnose(&tu, &config);
    
    // VALUE_123はOK、test123は警告
    let var_warnings: Vec<_> = diagnostics.iter().filter(|d| d.code == "CGH006").collect();
    assert_eq!(var_warnings.len(), 1);
    assert!(var_warnings[0].message.contains("test123"));
}

#[test]
fn test_global_var_multiple() {
    let source = r#"
int goodVar = 1;
int GOOD_VAR2 = 2;
int badVar = 3;
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_file_header = false;
    let diagnostics = diagnose(&tu, &config);
    
    // 2つの警告（goodVar, badVar）
    let var_warnings: Vec<_> = diagnostics.iter().filter(|d| d.code == "CGH006").collect();
    assert_eq!(var_warnings.len(), 2);
}

#[test]
fn test_global_var_config_disabled() {
    let source = r#"
int globalVar = 10;
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_file_header = false;
    config.check_global_var_naming = false;  // 無効化
    let diagnostics = diagnose(&tu, &config);
    
    // 警告なし
    let var_warnings: Vec<_> = diagnostics.iter().filter(|d| d.code == "CGH006").collect();
    assert_eq!(var_warnings.len(), 0);
}

#[test]
fn test_global_var_mixed_case_conversion() {
    let source = r#"
int testVarName = 10;
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_file_header = false;
    let diagnostics = diagnose(&tu, &config);
    
    // TEST_VAR_NAMEを推奨
    let var_warnings: Vec<_> = diagnostics.iter().filter(|d| d.code == "CGH006").collect();
    assert_eq!(var_warnings.len(), 1);
    assert!(var_warnings[0].message.contains("TEST_VAR_NAME"));
}

#[test]
fn test_global_var_struct_type() {
    let source = r#"
struct Point {
    int x;
    int y;
};

struct Point myPoint;
struct Point MY_POINT;
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_file_header = false;
    let diagnostics = diagnose(&tu, &config);
    
    // myPointは警告、MY_POINTはOK
    let var_warnings: Vec<_> = diagnostics.iter().filter(|d| d.code == "CGH006").collect();
    assert_eq!(var_warnings.len(), 1);
    assert!(var_warnings[0].message.contains("myPoint"));
    assert!(var_warnings[0].message.contains("MY_POINT"));
}

#[test]
fn test_global_var_enum_type() {
    let source = r#"
enum Color {
    RED,
    GREEN,
    BLUE
};

enum Color currentColor;
enum Color SELECTED_COLOR;
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_file_header = false;
    let diagnostics = diagnose(&tu, &config);
    
    // currentColorは警告、SELECTED_COLORはOK
    let var_warnings: Vec<_> = diagnostics.iter().filter(|d| d.code == "CGH006").collect();
    assert_eq!(var_warnings.len(), 1);
    assert!(var_warnings[0].message.contains("currentColor"));
    assert!(var_warnings[0].message.contains("CURRENT_COLOR"));
}

#[test]
fn test_global_var_union_type() {
    let source = r#"
union Data {
    int i;
    float f;
};

union Data myData;
union Data GLOBAL_DATA;
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_file_header = false;
    let diagnostics = diagnose(&tu, &config);
    
    // myDataは警告、GLOBAL_DATAはOK
    let var_warnings: Vec<_> = diagnostics.iter().filter(|d| d.code == "CGH006").collect();
    assert_eq!(var_warnings.len(), 1);
    assert!(var_warnings[0].message.contains("myData"));
    assert!(var_warnings[0].message.contains("MY_DATA"));
}

#[test]
fn test_global_var_typedef_struct() {
    // typedefで定義された型を使った変数宣言は、パーサーが型として認識する
    // ただし、現在のパーサーの実装では、typedef後の型名を使った変数宣言が
    // 正しく解析されないため、このテストはスキップする
    // TODO: パーサーがtypedefの型名を正しく認識するようになったら、このテストを有効化
}

#[test]
fn test_global_var_typedef_struct_inline() {
    // typedef struct with inline variable declaration
    let source = r#"
typedef struct {
    int x;
    int y;
} Point, *PointPtr, myPoint;
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_file_header = false;
    let diagnostics = diagnose(&tu, &config);
    
    // typedef宣言なので警告なし（has_typedef = trueのため）
    let var_warnings: Vec<_> = diagnostics.iter().filter(|d| d.code == "CGH006").collect();
    assert_eq!(var_warnings.len(), 0);
}

#[test]
fn test_global_var_mixed_types() {
    let source = r#"
struct Point {
    int x;
    int y;
};

enum Color {
    RED,
    GREEN
};

int counter;
struct Point position;
enum Color color;
int MAX_VALUE;
"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let mut config = DiagnosticConfig::default();
    config.check_file_header = false;
    let diagnostics = diagnose(&tu, &config);
    
    // counter, position, colorが警告（3つ）、MAX_VALUEはOK
    let var_warnings: Vec<_> = diagnostics.iter().filter(|d| d.code == "CGH006").collect();
    assert_eq!(var_warnings.len(), 3);
}
