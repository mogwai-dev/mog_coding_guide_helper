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
        Item::EnumDecl { enum_name, has_typedef, variable_names, variants, .. } => {
            assert_eq!(enum_name.as_deref(), Some("Status"));
            assert_eq!(*has_typedef, false);
            assert_eq!(variable_names.len(), 0);
            
            // variantsの検証
            assert_eq!(variants.len(), 2);
            assert_eq!(variants[0].name, "OK");
            assert_eq!(variants[0].value, Some(0));
            assert_eq!(variants[1].name, "NG");
            assert_eq!(variants[1].value, Some(1));
        },
        _ => panic!("Expected EnumDecl"),
    }
}

#[test]
fn test_parser_enum_with_hex_values() {
    let code = "enum Flags { FLAG_A = 0x01, FLAG_B = 0x02, FLAG_C = 0x04 };";
    let lx = Lexer::new(code);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();
    assert_eq!(tu.items.len(), 1);
    
    match &tu.items[0] {
        Item::EnumDecl { variants, .. } => {
            assert_eq!(variants.len(), 3);
            assert_eq!(variants[0].name, "FLAG_A");
            assert_eq!(variants[0].value, Some(0x01));
            assert_eq!(variants[1].name, "FLAG_B");
            assert_eq!(variants[1].value, Some(0x02));
            assert_eq!(variants[2].name, "FLAG_C");
            assert_eq!(variants[2].value, Some(0x04));
        },
        _ => panic!("Expected EnumDecl"),
    }
}

#[test]
fn test_parser_enum_mixed_values() {
    let code = "enum Mixed { A, B = 10, C, D = 0x20 };";
    let lx = Lexer::new(code);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();
    assert_eq!(tu.items.len(), 1);
    
    match &tu.items[0] {
        Item::EnumDecl { variants, .. } => {
            assert_eq!(variants.len(), 4);
            assert_eq!(variants[0].name, "A");
            assert_eq!(variants[0].value, None);
            assert_eq!(variants[1].name, "B");
            assert_eq!(variants[1].value, Some(10));
            assert_eq!(variants[2].name, "C");
            assert_eq!(variants[2].value, None);
            assert_eq!(variants[3].name, "D");
            assert_eq!(variants[3].value, Some(0x20));
        },
        _ => panic!("Expected EnumDecl"),
    }
}
