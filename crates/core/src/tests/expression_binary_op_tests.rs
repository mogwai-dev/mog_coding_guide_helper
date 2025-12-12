use crate::lexer::Lexer;
use crate::expression_parser::ExpressionParser;
use crate::expression::{Expression, BinaryOperator};

#[test]
fn test_parse_addition() {
    let input = "1 + 2";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    let expr = parser.parse_expression().expect("Failed to parse expression");
    
    match expr {
        Expression::BinaryOp { left, op, right, .. } => {
            assert_eq!(op, BinaryOperator::Add);
            assert!(matches!(*left, Expression::IntLiteral { value: 1, .. }));
            assert!(matches!(*right, Expression::IntLiteral { value: 2, .. }));
        },
        _ => panic!("Expected BinaryOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_multiplication_precedence() {
    let input = "1 + 2 * 3";  // Should parse as 1 + (2 * 3)
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    let expr = parser.parse_expression().expect("Failed to parse expression");
    
    match expr {
        Expression::BinaryOp { left, op: BinaryOperator::Add, right, .. } => {
            assert!(matches!(*left, Expression::IntLiteral { value: 1, .. }));
            match *right {
                Expression::BinaryOp { left: mul_left, op: BinaryOperator::Multiply, right: mul_right, .. } => {
                    assert!(matches!(*mul_left, Expression::IntLiteral { value: 2, .. }));
                    assert!(matches!(*mul_right, Expression::IntLiteral { value: 3, .. }));
                },
                _ => panic!("Expected multiplication on right side"),
            }
        },
        _ => panic!("Expected addition at top level"),
    }
}

#[test]
fn test_parse_comparison() {
    let input = "a == b";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    let expr = parser.parse_expression().expect("Failed to parse expression");
    
    match expr {
        Expression::BinaryOp { left, op, right, .. } => {
            assert_eq!(op, BinaryOperator::Equal);
            assert!(matches!(*left, Expression::Identifier { ref name, .. } if name == "a"));
            assert!(matches!(*right, Expression::Identifier { ref name, .. } if name == "b"));
        },
        _ => panic!("Expected BinaryOp"),
    }
}

#[test]
fn test_parse_logical_and() {
    let input = "x && y";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    let expr = parser.parse_expression().expect("Failed to parse expression");
    
    match expr {
        Expression::BinaryOp { op, .. } => {
            assert_eq!(op, BinaryOperator::LogicalAnd);
        },
        _ => panic!("Expected BinaryOp"),
    }
}

#[test]
fn test_parse_bitwise_operations() {
    let input = "a & b | c ^ d";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    let expr = parser.parse_expression().expect("Failed to parse expression");
    
    // Precedence: & (highest) > ^ > | (lowest)
    // So "a & b | c ^ d" parses as "(a & b) | (c ^ d)"
    match expr {
        Expression::BinaryOp { op: BinaryOperator::BitwiseOr, .. } => {
            // Top level should be OR
        },
        _ => panic!("Expected OR at top level, got {:?}", expr),
    }
}

#[test]
fn test_parse_shift_operations() {
    let input = "x << 2";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    let expr = parser.parse_expression().expect("Failed to parse expression");
    
    match expr {
        Expression::BinaryOp { op, .. } => {
            assert_eq!(op, BinaryOperator::LeftShift);
        },
        _ => panic!("Expected BinaryOp"),
    }
}

#[test]
fn test_parse_complex_expression() {
    let input = "a + b * c - d / e";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    let expr = parser.parse_expression();
    assert!(expr.is_some(), "Should successfully parse complex expression");
}

#[test]
fn test_parse_relational_operators() {
    let tests = vec![
        ("a < b", BinaryOperator::LessThan),
        ("a <= b", BinaryOperator::LessThanOrEq),
        ("a > b", BinaryOperator::GreaterThan),
        ("a >= b", BinaryOperator::GreaterThanOrEq),
    ];
    
    for (input, expected_op) in tests {
        let mut lexer = Lexer::new(input);
        let mut parser = ExpressionParser::new(&mut lexer);
        
        let expr = parser.parse_expression().expect(&format!("Failed to parse: {}", input));
        
        match expr {
            Expression::BinaryOp { op, .. } => {
                assert_eq!(op, expected_op, "Failed for input: {}", input);
            },
            _ => panic!("Expected BinaryOp for input: {}", input),
        }
    }
}

#[test]
fn test_parse_modulo() {
    let input = "10 % 3";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    
    let expr = parser.parse_expression().expect("Failed to parse expression");
    
    match expr {
        Expression::BinaryOp { left, op, right, .. } => {
            assert_eq!(op, BinaryOperator::Modulo);
            assert!(matches!(*left, Expression::IntLiteral { value: 10, .. }));
            assert!(matches!(*right, Expression::IntLiteral { value: 3, .. }));
        },
        _ => panic!("Expected BinaryOp"),
    }
}
