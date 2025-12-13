use crate::formatter::Formatter;
use crate::lexer::Lexer;
use crate::parser::Parser;

#[test]
fn test_formatter_convert_spaces_to_tabs() {
    let source = r#"int main() {
    int x = 10;
    if (x > 5) {
        return x;
    }
    return 0;
}"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let formatter = Formatter::new_with_all_options(false, false, true);
    let formatted = formatter.format_tu(&tu);
    
    println!("=== Formatted output ===");
    println!("{}", formatted);
    println!("=== Debug ===");
    println!("{:?}", formatted);
    
    // 4スペースがタブに変換されていることを確認
    assert!(formatted.contains("\tint x = 10;"));
    assert!(formatted.contains("\tif (x > 5) {"));
    assert!(formatted.contains("\t\treturn x;"));  // ネストは2タブ
    assert!(formatted.contains("\treturn 0;"));
}

#[test]
fn test_formatter_mixed_indentation() {
    let source = r#"void test() {
    int a = 1;
        int b = 2;
      int c = 3;
}"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let formatter = Formatter::new_with_all_options(false, false, true);
    let formatted = formatter.format_tu(&tu);
    
    // 4スペース = 1タブ, 8スペース = 2タブ
    assert!(formatted.contains("\tint a = 1;"));
    // 6スペース = 1タブ + 2スペース
    assert!(formatted.contains("\t  int c = 3;"));
}

#[test]
fn test_formatter_no_tab_conversion() {
    let source = r#"int main() {
    return 0;
}"#;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    let formatter = Formatter::new_with_all_options(false, false, false);
    let formatted = formatter.format_tu(&tu);
    
    // use_tabs = false の場合、スペースのまま
    assert!(formatted.contains("    return 0;"));
    assert!(!formatted.contains("\treturn 0;"));
}
