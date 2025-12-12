use crate::lexer::Lexer;
use crate::expression_parser::ExpressionParser;
use crate::expression::{Expression, UnaryOperator};

#[test]
fn test_parse_unary_minus() {
    let input = "-5";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    let expr = parser.parse_expression().expect("Failed to parse");
    
    match expr {
        Expression::UnaryOp { op, operand, .. } => {
            assert_eq!(op, UnaryOperator::Negate);
            assert!(matches!(*operand, Expression::IntLiteral { value: 5, .. }));
        },
        _ => panic!("Expected UnaryOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_logical_not() {
    let input = "!x";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    let expr = parser.parse_expression().expect("Failed to parse");
    
    match expr {
        Expression::UnaryOp { op, .. } => {
            assert_eq!(op, UnaryOperator::LogicalNot);
        },
        _ => panic!("Expected UnaryOp"),
    }
}

#[test]
fn test_parse_bitwise_not() {
    let input = "~a";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    let expr = parser.parse_expression().expect("Failed to parse");
    
    match expr {
        Expression::UnaryOp { op, .. } => {
            assert_eq!(op, UnaryOperator::BitwiseNot);
        },
        _ => panic!("Expected UnaryOp"),
    }
}

#[test]
fn test_parse_address_of() {
    let input = "&var";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    let expr = parser.parse_expression().expect("Failed to parse");
    
    match expr {
        Expression::UnaryOp { op, .. } => {
            assert_eq!(op, UnaryOperator::AddressOf);
        },
        _ => panic!("Expected UnaryOp"),
    }
}

#[test]
fn test_parse_dereference() {
    let input = "*ptr";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    let expr = parser.parse_expression().expect("Failed to parse");
    
    match expr {
        Expression::UnaryOp { op, .. } => {
            assert_eq!(op, UnaryOperator::Dereference);
        },
        _ => panic!("Expected UnaryOp"),
    }
}

#[test]
fn test_parse_pre_increment() {
    let input = "++i";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    let expr = parser.parse_expression().expect("Failed to parse");
    
    match expr {
        Expression::UnaryOp { op, .. } => {
            assert_eq!(op, UnaryOperator::PreIncrement);
        },
        _ => panic!("Expected UnaryOp"),
    }
}

#[test]
fn test_parse_post_increment() {
    let input = "i++";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    let expr = parser.parse_expression().expect("Failed to parse");
    
    match expr {
        Expression::UnaryOp { op, .. } => {
            assert_eq!(op, UnaryOperator::PostIncrement);
        },
        _ => panic!("Expected UnaryOp"),
    }
}

#[test]
fn test_parse_double_unary() {
    let input = "-(-5)";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    let expr = parser.parse_expression().expect("Failed to parse");
    
    match expr {
        Expression::UnaryOp { op: UnaryOperator::Negate, operand, .. } => {
            match *operand {
                Expression::UnaryOp { op: UnaryOperator::Negate, .. } => {
                    // OK: double negation
                },
                _ => panic!("Expected nested UnaryOp"),
            }
        },
        _ => panic!("Expected UnaryOp at top level"),
    }
}

#[test]
fn test_parse_parentheses() {
    let input = "(5)";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    let expr = parser.parse_expression().expect("Failed to parse");
    
    match expr {
        Expression::IntLiteral { value: 5, .. } => {
            // OK
        },
        _ => panic!("Expected IntLiteral"),
    }
}

#[test]
fn test_parse_parentheses_changes_precedence() {
    let input = "(1 + 2) * 3";  // Should parse as (1 + 2) * 3 = 9, not 1 + (2 * 3) = 7
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    let expr = parser.parse_expression().expect("Failed to parse");
    
    // Top level should be multiplication
    match expr {
        Expression::BinaryOp { left, op, right, .. } => {
            assert_eq!(op, crate::expression::BinaryOperator::Multiply);
            // Left should be addition
            match *left {
                Expression::BinaryOp { op: crate::expression::BinaryOperator::Add, .. } => {
                    // OK
                },
                _ => panic!("Expected addition on left side"),
            }
            // Right should be 3
            assert!(matches!(*right, Expression::IntLiteral { value: 3, .. }));
        },
        _ => panic!("Expected BinaryOp at top level"),
    }
}

#[test]
fn test_parse_complex_with_unary() {
    let input = "-a + b * -c";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    let expr = parser.parse_expression();
    assert!(expr.is_some(), "Should successfully parse complex expression with unary operators");
}

#[test]
fn test_parse_nested_parentheses() {
    let input = "((5))";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    let expr = parser.parse_expression().expect("Failed to parse");
    
    match expr {
        Expression::IntLiteral { value: 5, .. } => {
            // OK
        },
        _ => panic!("Expected IntLiteral"),
    }
}

#[test]
fn test_parse_unary_with_binary() {
    let input = "!a && b";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    let expr = parser.parse_expression().expect("Failed to parse");
    
    match expr {
        Expression::BinaryOp { left, op, .. } => {
            assert_eq!(op, crate::expression::BinaryOperator::LogicalAnd);
            // Left should be logical not
            match *left {
                Expression::UnaryOp { op: UnaryOperator::LogicalNot, .. } => {
                    // OK
                },
                _ => panic!("Expected LogicalNot on left side"),
            }
        },
        _ => panic!("Expected BinaryOp at top level"),
    }
}

#[test]
fn test_parse_address_dereference() {
    let input = "*&x";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    let expr = parser.parse_expression().expect("Failed to parse");
    
    match expr {
        Expression::UnaryOp { op: UnaryOperator::Dereference, operand, .. } => {
            match *operand {
                Expression::UnaryOp { op: UnaryOperator::AddressOf, .. } => {
                    // OK: *&x
                },
                _ => panic!("Expected AddressOf inside Dereference"),
            }
        },
        _ => panic!("Expected Dereference at top level"),
    }
}
