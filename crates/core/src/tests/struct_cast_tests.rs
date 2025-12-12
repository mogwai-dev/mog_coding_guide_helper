use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::expression_parser::ExpressionParser;
use crate::expression::Expression;

/// Phase 3: struct/union/enum型のキャストテスト

#[test]
fn test_struct_cast() {
    // struct Point型を使ったキャスト
    let input = "(struct Point) value";
    let mut lexer = Lexer::new(input);
    let mut expr_parser = ExpressionParser::new(&mut lexer);
    let expr = expr_parser.parse_expression().unwrap();
    
    assert!(matches!(expr, Expression::Cast { .. }), "Should be recognized as cast");
}

#[test]
fn test_union_cast() {
    // union Data型を使ったキャスト
    let input = "(union Data) value";
    let mut lexer = Lexer::new(input);
    let mut expr_parser = ExpressionParser::new(&mut lexer);
    let expr = expr_parser.parse_expression().unwrap();
    
    assert!(matches!(expr, Expression::Cast { .. }), "Should be recognized as cast");
}

#[test]
fn test_enum_cast() {
    // enum Color型を使ったキャスト
    let input = "(enum Color) value";
    let mut lexer = Lexer::new(input);
    let mut expr_parser = ExpressionParser::new(&mut lexer);
    let expr = expr_parser.parse_expression().unwrap();
    
    assert!(matches!(expr, Expression::Cast { .. }), "Should be recognized as cast");
}

#[test]
fn test_typedef_struct_cast() {
    // typedef struct を定義してキャスト
    let typedef_code = "typedef struct { int x; int y; } Point;";
    let typedef_lexer = Lexer::new(typedef_code);
    let mut parser = Parser::new(typedef_lexer);
    let _ast = parser.parse();
    
    // Pointを使ったキャスト（typedefされた名前）
    let input = "(Point) value";
    let mut lexer = Lexer::new(input);
    let mut expr_parser = ExpressionParser::new(&mut lexer).with_type_table(parser.get_type_table());
    let expr = expr_parser.parse_expression().unwrap();
    
    assert!(matches!(expr, Expression::Cast { .. }), "typedef struct should work as cast");
}

#[test]
fn test_named_struct_typedef_cast() {
    // 名前付きstructのtypedef
    let typedef_code = "typedef struct Point { int x; int y; } Point;";
    let typedef_lexer = Lexer::new(typedef_code);
    let mut parser = Parser::new(typedef_lexer);
    let _ast = parser.parse();
    
    // struct Point または Point でキャスト可能
    let input1 = "(struct Point) value";
    let mut lexer1 = Lexer::new(input1);
    let mut expr_parser1 = ExpressionParser::new(&mut lexer1).with_type_table(parser.get_type_table());
    let expr1 = expr_parser1.parse_expression().unwrap();
    assert!(matches!(expr1, Expression::Cast { .. }), "struct Point should work");
    
    let input2 = "(Point) value";
    let mut lexer2 = Lexer::new(input2);
    let mut expr_parser2 = ExpressionParser::new(&mut lexer2).with_type_table(parser.get_type_table());
    let expr2 = expr_parser2.parse_expression().unwrap();
    assert!(matches!(expr2, Expression::Cast { .. }), "Point typedef should work");
}

#[test]
fn test_struct_pointer_cast() {
    // struct ポインタ型のキャスト
    let input = "(struct Point *) ptr";
    let mut lexer = Lexer::new(input);
    let mut expr_parser = ExpressionParser::new(&mut lexer);
    let expr = expr_parser.parse_expression().unwrap();
    
    assert!(matches!(expr, Expression::Cast { .. }), "struct pointer cast should work");
}

#[test]
fn test_const_struct_cast() {
    // const struct型のキャスト
    let input = "(const struct Point) value";
    let mut lexer = Lexer::new(input);
    let mut expr_parser = ExpressionParser::new(&mut lexer);
    let expr = expr_parser.parse_expression().unwrap();
    
    assert!(matches!(expr, Expression::Cast { .. }), "const struct cast should work");
}

#[test]
fn test_multiple_struct_types() {
    // 複数のstruct型を定義
    let typedef_code = r#"
        typedef struct { int x; } Point2D;
        typedef struct { int x; int y; int z; } Point3D;
        typedef union { int i; float f; } Data;
        typedef enum { RED, GREEN, BLUE } Color;
    "#;
    let typedef_lexer = Lexer::new(typedef_code);
    let mut parser = Parser::new(typedef_lexer);
    let _ast = parser.parse();
    
    // すべて登録されているか確認
    assert!(parser.get_type_table().is_type_name("Point2D"), "Point2D should be registered");
    assert!(parser.get_type_table().is_type_name("Point3D"), "Point3D should be registered");
    assert!(parser.get_type_table().is_type_name("Data"), "Data should be registered");
    assert!(parser.get_type_table().is_type_name("Color"), "Color should be registered");
}
