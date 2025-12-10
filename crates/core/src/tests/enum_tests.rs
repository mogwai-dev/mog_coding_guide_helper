use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::ast::Item;

#[test]
fn test_parser_simple_enum() {
    let code = "enum Color { RED, GREEN, BLUE };";
    let lx = Lexer::new(code);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();
    assert_eq!(tu.items.len(), 1);
    
    match &tu.items[0] {
        Item::EnumDecl { enum_name, has_typedef, variable_names, .. } => {
            assert_eq!(enum_name.as_deref(), Some("Color"));
            assert_eq!(*has_typedef, false);
            assert_eq!(variable_names.len(), 0);
        },
        _ => panic!("Expected EnumDecl"),
    }
}

#[test]
fn test_parser_anonymous_enum() {
    let code = "enum { UNKNOWN, ERROR };";
    let lx = Lexer::new(code);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();
    assert_eq!(tu.items.len(), 1);
    
    match &tu.items[0] {
        Item::EnumDecl { enum_name, has_typedef, variable_names, .. } => {
            assert_eq!(*enum_name, None);
            assert_eq!(*has_typedef, false);
            assert_eq!(variable_names.len(), 0);
        },
        _ => panic!("Expected EnumDecl"),
    }
}

#[test]
fn test_parser_typedef_enum() {
    let code = "typedef enum { SUCCESS, FAILURE } Status;";
    let lx = Lexer::new(code);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();
    assert_eq!(tu.items.len(), 1);
    
    match &tu.items[0] {
        Item::EnumDecl { enum_name, has_typedef, variable_names, .. } => {
            assert_eq!(*enum_name, None);
            assert_eq!(*has_typedef, true);
            assert_eq!(variable_names, &vec!["Status"]);
        },
        _ => panic!("Expected EnumDecl"),
    }
}

#[test]
fn test_parser_enum_with_variables() {
    let code = "enum Color { RED, BLUE } color1, color2;";
    let lx = Lexer::new(code);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();
    assert_eq!(tu.items.len(), 1);
    
    match &tu.items[0] {
        Item::EnumDecl { enum_name, has_typedef, variable_names, .. } => {
            assert_eq!(enum_name.as_deref(), Some("Color"));
            assert_eq!(*has_typedef, false);
            assert_eq!(variable_names, &vec!["color1", "color2"]);
        },
        _ => panic!("Expected EnumDecl"),
    }
}

#[test]
fn test_parser_enum_with_values() {
    let code = "enum Status { OK = 0, NG = 1 };";
    let lx = Lexer::new(code);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();
    assert_eq!(tu.items.len(), 1);
    
    match &tu.items[0] {
        Item::EnumDecl { enum_name, has_typedef, variable_names, .. } => {
            assert_eq!(enum_name.as_deref(), Some("Status"));
            assert_eq!(*has_typedef, false);
            assert_eq!(variable_names.len(), 0);
        },
        _ => panic!("Expected EnumDecl"),
    }
}
