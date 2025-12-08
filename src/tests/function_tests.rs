use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::ast::Item;

#[test]
fn test_parser_simple_function() {
    let s = "int main(void) { return 0; }\n";
    let lx = Lexer::new(s);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();

    assert_eq!(tu.items.len(), 1);

    match &tu.items[0] {
        Item::FunctionDecl { return_type, function_name, parameters, text, .. } => {
            assert_eq!(return_type, "int");
            assert_eq!(function_name, "main");
            assert!(parameters.starts_with("(void)"));
            assert!(text.contains("int main(void)"));
        }
        _ => panic!("Expected FunctionDecl item"),
    }
}

#[test]
fn test_parser_function_with_parameters() {
    let s = "void foo(int x, char *y) { }\n";
    let lx = Lexer::new(s);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();

    assert_eq!(tu.items.len(), 1);

    match &tu.items[0] {
        Item::FunctionDecl { return_type, function_name, parameters, .. } => {
            assert_eq!(return_type, "void");
            assert_eq!(function_name, "foo");
            assert!(parameters.contains("int x"));
            assert!(parameters.contains("char *y"));
        }
        _ => panic!("Expected FunctionDecl item"),
    }
}

#[test]
fn test_parser_static_function() {
    let s = "static int helper(void) { return 1; }\n";
    let lx = Lexer::new(s);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();

    assert_eq!(tu.items.len(), 1);

    match &tu.items[0] {
        Item::FunctionDecl { return_type, function_name, .. } => {
            // return_type には "static int" が含まれるはず
            assert!(return_type.contains("int"));
            assert_eq!(function_name, "helper");
        }
        _ => panic!("Expected FunctionDecl item"),
    }
}

#[test]
fn test_parser_multiple_functions() {
    let s = r#"
int add(int a, int b) { return a + b; }
void print(void) { }
"#;
    let lx = Lexer::new(s);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();

    assert_eq!(tu.items.len(), 2);

    match &tu.items[0] {
        Item::FunctionDecl { function_name, .. } => {
            assert_eq!(function_name, "add");
        }
        _ => panic!("Expected FunctionDecl item"),
    }

    match &tu.items[1] {
        Item::FunctionDecl { function_name, .. } => {
            assert_eq!(function_name, "print");
        }
        _ => panic!("Expected FunctionDecl item"),
    }
}

#[test]
fn test_parser_function_with_nested_braces() {
    let s = "int test(void) { if (1) { return 2; } return 0; }\n";
    let lx = Lexer::new(s);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();

    assert_eq!(tu.items.len(), 1);

    match &tu.items[0] {
        Item::FunctionDecl { function_name, text, .. } => {
            assert_eq!(function_name, "test");
            // ブロック全体が含まれているか確認
            assert!(text.contains("if (1)"));
            assert!(text.contains("return 2"));
        }
        _ => panic!("Expected FunctionDecl item"),
    }
}
