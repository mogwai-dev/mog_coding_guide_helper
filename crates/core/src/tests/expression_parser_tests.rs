use crate::lexer::Lexer;
use crate::expression_parser::ExpressionParser;
use crate::expression::Expression;

#[test]
fn test_parse_integer_literal() {
    let code = "123";
    let mut lexer = Lexer::new(code);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    match parser.parse_expression() {
        Some(Expression::IntLiteral { value, .. }) => {
            assert_eq!(value, 123);
        }
        other => panic!("Expected IntLiteral, got {:?}", other),
    }
}

#[test]
fn test_parse_hex_literal() {
    let code = "0xFF";
    let mut lexer = Lexer::new(code);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    match parser.parse_expression() {
        Some(Expression::IntLiteral { value, .. }) => {
            assert_eq!(value, 255);
        }
        other => panic!("Expected IntLiteral with value 255, got {:?}", other),
    }
}

#[test]
fn test_parse_float_literal() {
    let code = "3.14";
    let mut lexer = Lexer::new(code);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    match parser.parse_expression() {
        Some(Expression::FloatLiteral { value, .. }) => {
            assert!((value - 3.14).abs() < 0.0001);
        }
        other => panic!("Expected FloatLiteral, got {:?}", other),
    }
}

#[test]
fn test_parse_identifier() {
    let code = "foo";
    let mut lexer = Lexer::new(code);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    match parser.parse_expression() {
        Some(Expression::Identifier { name, .. }) => {
            assert_eq!(name, "foo");
        }
        other => panic!("Expected Identifier, got {:?}", other),
    }
}
