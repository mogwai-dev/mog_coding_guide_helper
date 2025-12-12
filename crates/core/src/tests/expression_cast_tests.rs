use crate::expression::*;
use crate::expression_parser::ExpressionParser;
use crate::lexer::Lexer;
use crate::type_system::{BaseType, Type};

#[test]
fn test_cast_int() {
    let input = "(int) 3.14";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Cast { target_type, operand, .. } => {
            // 型がintであることを確認
            assert_eq!(target_type.base_type, BaseType::Int);
            assert!(target_type.pointer_layers.is_empty());
            
            // オペランドが浮動小数点リテラルであることを確認
            match *operand {
                Expression::FloatLiteral { value, .. } => {
                    assert!((value - 3.14).abs() < 0.0001);
                },
                _ => panic!("Expected FloatLiteral, got {:?}", operand),
            }
        },
        _ => panic!("Expected Cast expression, got {:?}", expr),
    }
}

#[test]
fn test_cast_float() {
    let input = "(float) 42";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Cast { target_type, operand, .. } => {
            assert_eq!(target_type.base_type, BaseType::Float);
            
            match *operand {
                Expression::IntLiteral { value, .. } => {
                    assert_eq!(value, 42);
                },
                _ => panic!("Expected IntLiteral, got {:?}", operand),
            }
        },
        _ => panic!("Expected Cast expression, got {:?}", expr),
    }
}

#[test]
fn test_cast_pointer() {
    let input = "(int *) ptr";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Cast { target_type, operand, .. } => {
            assert_eq!(target_type.base_type, BaseType::Int);
            assert_eq!(target_type.pointer_layers.len(), 1);
            
            match *operand {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "ptr");
                },
                _ => panic!("Expected Identifier, got {:?}", operand),
            }
        },
        _ => panic!("Expected Cast expression, got {:?}", expr),
    }
}

#[test]
fn test_cast_double_pointer() {
    let input = "(char * *) p";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Cast { target_type, operand, .. } => {
            assert_eq!(target_type.base_type, BaseType::Char);
            assert_eq!(target_type.pointer_layers.len(), 2);
            
            match *operand {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "p");
                },
                _ => panic!("Expected Identifier, got {:?}", operand),
            }
        },
        _ => panic!("Expected Cast expression, got {:?}", expr),
    }
}

#[test]
fn test_cast_unsigned() {
    let input = "(unsigned int) -1";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Cast { target_type, operand, .. } => {
            // unsigned int型を確認（signedとintの組み合わせ）
            assert_eq!(target_type.base_type, BaseType::Unsigned);
            
            match *operand {
                Expression::UnaryOp { op, operand: inner, .. } => {
                    assert_eq!(op, UnaryOperator::Negate);
                    match *inner {
                        Expression::IntLiteral { value, .. } => {
                            assert_eq!(value, 1);
                        },
                        _ => panic!("Expected IntLiteral in unary op"),
                    }
                },
                _ => panic!("Expected UnaryOp, got {:?}", operand),
            }
        },
        _ => panic!("Expected Cast expression, got {:?}", expr),
    }
}

#[test]
fn test_cast_long() {
    let input = "(long) value";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Cast { target_type, operand, .. } => {
            assert_eq!(target_type.base_type, BaseType::Long);
            
            match *operand {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "value");
                },
                _ => panic!("Expected Identifier, got {:?}", operand),
            }
        },
        _ => panic!("Expected Cast expression, got {:?}", expr),
    }
}

#[test]
fn test_cast_in_expression() {
    let input = "(int) x + y";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    let expr = parser.parse_expression().unwrap();

    // (int)x + y は ((int)x) + y としてパースされる
    match expr {
        Expression::BinaryOp { op, left, right, .. } => {
            assert_eq!(op, BinaryOperator::Add);
            
            // left が Cast
            match *left {
                Expression::Cast { target_type, operand, .. } => {
                    assert_eq!(target_type.base_type, BaseType::Int);
                    match *operand {
                        Expression::Identifier { name, .. } => {
                            assert_eq!(name, "x");
                        },
                        _ => panic!("Expected Identifier in cast"),
                    }
                },
                _ => panic!("Expected Cast on left side"),
            }
            
            // right が Identifier
            match *right {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "y");
                },
                _ => panic!("Expected Identifier on right side"),
            }
        },
        _ => panic!("Expected BinaryOp expression, got {:?}", expr),
    }
}

#[test]
fn test_double_cast() {
    let input = "(int) (float) x";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    let expr = parser.parse_expression().unwrap();

    // (int) (float) x は (int) ((float) x) としてパースされる
    match expr {
        Expression::Cast { target_type, operand, .. } => {
            assert_eq!(target_type.base_type, BaseType::Int);
            
            // オペランドも Cast
            match *operand {
                Expression::Cast { target_type: inner_type, operand: inner_operand, .. } => {
                    assert_eq!(inner_type.base_type, BaseType::Float);
                    
                    match *inner_operand {
                        Expression::Identifier { name, .. } => {
                            assert_eq!(name, "x");
                        },
                        _ => panic!("Expected Identifier in inner cast"),
                    }
                },
                _ => panic!("Expected Cast as operand"),
            }
        },
        _ => panic!("Expected Cast expression, got {:?}", expr),
    }
}

#[test]
fn test_cast_with_unary() {
    let input = "(int) *ptr";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Cast { target_type, operand, .. } => {
            assert_eq!(target_type.base_type, BaseType::Int);
            
            // オペランドが Dereference
            match *operand {
                Expression::UnaryOp { op, operand: inner, .. } => {
                    assert_eq!(op, UnaryOperator::Dereference);
                    match *inner {
                        Expression::Identifier { name, .. } => {
                            assert_eq!(name, "ptr");
                        },
                        _ => panic!("Expected Identifier in dereference"),
                    }
                },
                _ => panic!("Expected UnaryOp (dereference)"),
            }
        },
        _ => panic!("Expected Cast expression, got {:?}", expr),
    }
}

#[test]
fn test_cast_void_pointer() {
    let input = "(void *) data";
    let mut lexer = Lexer::new(input);
    let mut parser = ExpressionParser::new(&mut lexer);
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Cast { target_type, operand, .. } => {
            assert_eq!(target_type.base_type, BaseType::Void);
            assert_eq!(target_type.pointer_layers.len(), 1);
            
            match *operand {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "data");
                },
                _ => panic!("Expected Identifier, got {:?}", operand),
            }
        },
        _ => panic!("Expected Cast expression, got {:?}", expr),
    }
}
