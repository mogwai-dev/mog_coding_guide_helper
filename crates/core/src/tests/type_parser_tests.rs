use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::type_system::{BaseType, TypeQualifier};
use crate::ast::Item;

#[test]
fn test_parse_basic_int() {
    let input = "int";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let ty = parser.parse_type().expect("Should parse int type");
    
    assert_eq!(ty.base_type, BaseType::Int);
    assert_eq!(ty.base_qualifiers.len(), 0);
    assert_eq!(ty.pointer_level(), 0);
    assert!(!ty.is_pointer());
}

#[test]
fn test_parse_const_int() {
    let input = "const int";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let ty = parser.parse_type().expect("Should parse const int type");
    
    assert_eq!(ty.base_type, BaseType::Int);
    assert_eq!(ty.base_qualifiers.len(), 1);
    assert!(ty.has_base_qualifier(TypeQualifier::Const));
    assert_eq!(ty.pointer_level(), 0);
    assert!(!ty.is_pointer());
}

#[test]
fn test_parse_single_pointer() {
    let input = "int *";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let ty = parser.parse_type().expect("Should parse int * type");
    
    assert_eq!(ty.base_type, BaseType::Int);
    assert_eq!(ty.base_qualifiers.len(), 0);
    assert_eq!(ty.pointer_level(), 1);
    assert!(ty.is_pointer());
    assert_eq!(ty.pointer_layers[0].qualifiers.len(), 0);
}

#[test]
fn test_parse_double_pointer() {
    let input = "int **";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let ty = parser.parse_type().expect("Should parse int ** type");
    
    assert_eq!(ty.base_type, BaseType::Int);
    assert_eq!(ty.pointer_level(), 2);
    assert!(ty.is_pointer());
}

#[test]
fn test_parse_const_pointer() {
    let input = "int *const";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let ty = parser.parse_type().expect("Should parse int *const type");
    
    assert_eq!(ty.base_type, BaseType::Int);
    assert_eq!(ty.pointer_level(), 1);
    assert!(ty.is_pointer());
    assert_eq!(ty.pointer_layers[0].qualifiers.len(), 1);
    assert!(ty.pointer_layers[0].has_qualifier(TypeQualifier::Const));
}

#[test]
fn test_var_decl_with_pointer_type() {
    let input = "int *ptr;";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let tu = parser.parse();
    
    assert_eq!(tu.items.len(), 1);
    match &tu.items[0] {
        Item::VarDecl { var_name, var_type, .. } => {
            assert_eq!(var_name, "ptr");
            assert!(var_type.is_some());
            let ty = var_type.as_ref().unwrap();
            assert_eq!(ty.base_type, BaseType::Int);
            assert_eq!(ty.pointer_level(), 1);
        }
        _ => panic!("Expected VarDecl"),
    }
}
