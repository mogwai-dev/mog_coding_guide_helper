use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::ast::Item;

#[test]
fn test_parser_extern_variable() {
    let code = "extern int global_var;";
    let lx = Lexer::new(code);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();
    assert_eq!(tu.items.len(), 1);
    
    match &tu.items[0] {
        Item::VarDecl { text, var_name, .. } => {
            assert!(text.contains("extern"));
            assert!(text.contains("int"));
            assert_eq!(var_name, "global_var");
        },
        _ => panic!("Expected VarDecl, got {:?}", tu.items[0]),
    }
}

#[test]
fn test_parser_extern_function() {
    let code = "extern void external_func(int x);";
    let lx = Lexer::new(code);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();
    assert_eq!(tu.items.len(), 1);
    
    match &tu.items[0] {
        Item::FunctionDecl { return_type, function_name, storage_class, .. } => {
            assert_eq!(return_type, "void");
            assert_eq!(function_name, "external_func");
            assert_eq!(storage_class.as_deref(), Some("extern"));
        },
        _ => panic!("Expected FunctionDecl, got {:?}", tu.items[0]),
    }
}

#[test]
fn test_parser_extern_array() {
    let code = "extern char buffer[256];";
    let lx = Lexer::new(code);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();
    assert_eq!(tu.items.len(), 1);
    
    match &tu.items[0] {
        Item::VarDecl { text, var_name, .. } => {
            assert!(text.contains("extern"));
            assert!(text.contains("char"));
            assert_eq!(var_name, "buffer");
        },
        _ => panic!("Expected VarDecl, got {:?}", tu.items[0]),
    }
}

#[test]
fn test_parser_extern_pointer() {
    let code = "extern int* ptr;";
    let lx = Lexer::new(code);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();
    assert_eq!(tu.items.len(), 1);
    
    match &tu.items[0] {
        Item::VarDecl { text, var_name, .. } => {
            assert!(text.contains("extern"));
            assert!(text.contains("int"));
            assert_eq!(var_name, "ptr");
        },
        _ => panic!("Expected VarDecl, got {:?}", tu.items[0]),
    }
}

#[test]
fn test_parser_multiple_externs() {
    let code = "extern int x;\nextern float y;";
    let lx = Lexer::new(code);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();
    assert_eq!(tu.items.len(), 2);
    
    match &tu.items[0] {
        Item::VarDecl { var_name, .. } => {
            assert_eq!(var_name, "x");
        },
        _ => panic!("Expected VarDecl, got {:?}", tu.items[0]),
    }
    
    match &tu.items[1] {
        Item::VarDecl { var_name, .. } => {
            assert_eq!(var_name, "y");
        },
        _ => panic!("Expected VarDecl, got {:?}", tu.items[1]),
    }
}
