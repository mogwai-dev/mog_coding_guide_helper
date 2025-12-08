use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::ast::Item;

#[test]
fn test_parser_simple_union() {
    let code = "union Data { int i; float f; };";
    let lx = Lexer::new(code);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();
    assert_eq!(tu.items.len(), 1);
    
    match &tu.items[0] {
        Item::UnionDecl { union_name, has_typedef, variable_names, .. } => {
            assert_eq!(union_name.as_deref(), Some("Data"));
            assert_eq!(*has_typedef, false);
            assert_eq!(variable_names.len(), 0);
        },
        _ => panic!("Expected UnionDecl"),
    }
}

#[test]
fn test_parser_anonymous_union() {
    let code = "union { int x; float y; };";
    let lx = Lexer::new(code);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();
    assert_eq!(tu.items.len(), 1);
    
    match &tu.items[0] {
        Item::UnionDecl { union_name, has_typedef, variable_names, .. } => {
            assert_eq!(*union_name, None);
            assert_eq!(*has_typedef, false);
            assert_eq!(variable_names.len(), 0);
        },
        _ => panic!("Expected UnionDecl"),
    }
}

#[test]
fn test_parser_typedef_union() {
    let code = "typedef union { int i; float f; } Number;";
    let lx = Lexer::new(code);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();
    assert_eq!(tu.items.len(), 1);
    
    match &tu.items[0] {
        Item::UnionDecl { union_name, has_typedef, variable_names, .. } => {
            assert_eq!(*union_name, None);
            assert_eq!(*has_typedef, true);
            assert_eq!(variable_names, &vec!["Number"]);
        },
        _ => panic!("Expected UnionDecl"),
    }
}

#[test]
fn test_parser_union_with_variables() {
    let code = "union Data { int i; float f; } data1, data2;";
    let lx = Lexer::new(code);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();
    assert_eq!(tu.items.len(), 1);
    
    match &tu.items[0] {
        Item::UnionDecl { union_name, has_typedef, variable_names, .. } => {
            assert_eq!(union_name.as_deref(), Some("Data"));
            assert_eq!(*has_typedef, false);
            assert_eq!(variable_names, &vec!["data1", "data2"]);
        },
        _ => panic!("Expected UnionDecl"),
    }
}

#[test]
fn test_parser_named_typedef_union() {
    let code = "typedef union Value { int i; float f; } Value;";
    let lx = Lexer::new(code);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();
    assert_eq!(tu.items.len(), 1);
    
    match &tu.items[0] {
        Item::UnionDecl { union_name, has_typedef, variable_names, .. } => {
            assert_eq!(union_name.as_deref(), Some("Value"));
            assert_eq!(*has_typedef, true);
            assert_eq!(variable_names, &vec!["Value"]);
        },
        _ => panic!("Expected UnionDecl"),
    }
}
