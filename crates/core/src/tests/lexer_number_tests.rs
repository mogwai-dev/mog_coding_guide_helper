use crate::lexer::Lexer;
use crate::token::*;

#[test]
fn test_lexer_decimal_number() {
    let input = "123";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::NumberLiteral(NumberLiteralToken { value, .. })) => {
            assert_eq!(value, "123");
        }
        other => panic!("Expected NumberLiteral, got {:?}", other),
    }
}

#[test]
fn test_lexer_hex_number() {
    let input = "0x1A2B";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::NumberLiteral(NumberLiteralToken { value, .. })) => {
            assert_eq!(value, "0x1A2B");
        }
        other => panic!("Expected NumberLiteral, got {:?}", other),
    }
}

#[test]
fn test_lexer_hex_number_lowercase() {
    let input = "0xabcd";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::NumberLiteral(NumberLiteralToken { value, .. })) => {
            assert_eq!(value, "0xabcd");
        }
        other => panic!("Expected NumberLiteral, got {:?}", other),
    }
}

#[test]
fn test_lexer_octal_number() {
    let input = "0755";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::NumberLiteral(NumberLiteralToken { value, .. })) => {
            assert_eq!(value, "0755");
        }
        other => panic!("Expected NumberLiteral, got {:?}", other),
    }
}

#[test]
fn test_lexer_zero() {
    let input = "0";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::NumberLiteral(NumberLiteralToken { value, .. })) => {
            assert_eq!(value, "0");
        }
        other => panic!("Expected NumberLiteral, got {:?}", other),
    }
}

#[test]
fn test_lexer_numbers_with_identifiers() {
    let input = "int x = 123;";
    let mut lexer = Lexer::new(input);
    
    // int
    assert!(matches!(lexer.next_token(), Some(Token::Int(_))));
    
    // x
    match lexer.next_token() {
        Some(Token::Ident(IdentToken { name, .. })) => {
            assert_eq!(name, "x");
        }
        other => panic!("Expected Ident, got {:?}", other),
    }
    
    // =
    assert!(matches!(lexer.next_token(), Some(Token::Equal(_))));
    
    // 123
    match lexer.next_token() {
        Some(Token::NumberLiteral(NumberLiteralToken { value, .. })) => {
            assert_eq!(value, "123");
        }
        other => panic!("Expected NumberLiteral, got {:?}", other),
    }
    
    // ;
    assert!(matches!(lexer.next_token(), Some(Token::Semicolon(_))));
}

#[test]
fn test_lexer_float_simple() {
    let input = "3.14";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::FloatLiteral(FloatLiteralToken { value, .. })) => {
            assert_eq!(value, "3.14");
        }
        other => panic!("Expected FloatLiteral, got {:?}", other),
    }
}

#[test]
fn test_lexer_float_with_f_suffix() {
    let input = "1.5f";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::FloatLiteral(FloatLiteralToken { value, .. })) => {
            assert_eq!(value, "1.5f");
        }
        other => panic!("Expected FloatLiteral with 'f' suffix, got {:?}", other),
    }
}

#[test]
fn test_lexer_float_with_large_f_suffix() {
    let input = "2.5F";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::FloatLiteral(FloatLiteralToken { value, .. })) => {
            assert_eq!(value, "2.5F");
        }
        other => panic!("Expected FloatLiteral with 'F' suffix, got {:?}", other),
    }
}

#[test]
fn test_lexer_float_with_large_l_suffix() {
    let input = "9.99L";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::FloatLiteral(FloatLiteralToken { value, .. })) => {
            assert_eq!(value, "9.99L");
        }
        other => panic!("Expected FloatLiteral with 'L' suffix, got {:?}", other),
    }
}

#[test]
fn test_lexer_float_exponent_positive() {
    let input = "1e10";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::FloatLiteral(FloatLiteralToken { value, .. })) => {
            assert_eq!(value, "1e10");
        }
        other => panic!("Expected FloatLiteral with exponent, got {:?}", other),
    }
}

#[test]
fn test_lexer_float_exponent_negative() {
    let input = "2.5e-3";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::FloatLiteral(FloatLiteralToken { value, .. })) => {
            assert_eq!(value, "2.5e-3");
        }
        other => panic!("Expected FloatLiteral with negative exponent, got {:?}", other),
    }
}

#[test]
fn test_lexer_float_exponent_with_plus() {
    let input = "3.0e+5";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::FloatLiteral(FloatLiteralToken { value, .. })) => {
            assert_eq!(value, "3.0e+5");
        }
        other => panic!("Expected FloatLiteral with positive exponent, got {:?}", other),
    }
}

#[test]
fn test_lexer_float_exponent_uppercase() {
    let input = "1.5E10";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::FloatLiteral(FloatLiteralToken { value, .. })) => {
            assert_eq!(value, "1.5E10");
        }
        other => panic!("Expected FloatLiteral with uppercase E, got {:?}", other),
    }
}

#[test]
fn test_lexer_float_exponent_with_suffix() {
    let input = "2.5e-3L";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::FloatLiteral(FloatLiteralToken { value, .. })) => {
            assert_eq!(value, "2.5e-3L");
        }
        other => panic!("Expected FloatLiteral with exponent and suffix, got {:?}", other),
    }
}

#[test]
fn test_lexer_float_no_decimal_with_exponent() {
    let input = "5e2";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::FloatLiteral(FloatLiteralToken { value, .. })) => {
            assert_eq!(value, "5e2");
        }
        other => panic!("Expected FloatLiteral (integer with exponent), got {:?}", other),
    }
}

#[test]
fn test_lexer_float_in_code() {
    let input = "float pi = 3.14159f;";
    let mut lexer = Lexer::new(input);
    
    // float
    assert!(matches!(lexer.next_token(), Some(Token::Float(_))));
    
    // pi
    match lexer.next_token() {
        Some(Token::Ident(IdentToken { name, .. })) => {
            assert_eq!(name, "pi");
        }
        other => panic!("Expected Ident, got {:?}", other),
    }
    
    // =
    assert!(matches!(lexer.next_token(), Some(Token::Equal(_))));
    
    // 3.14159f
    match lexer.next_token() {
        Some(Token::FloatLiteral(FloatLiteralToken { value, .. })) => {
            assert_eq!(value, "3.14159f");
        }
        other => panic!("Expected FloatLiteral, got {:?}", other),
    }
    
    // ;
    assert!(matches!(lexer.next_token(), Some(Token::Semicolon(_))));
}

#[test]
fn test_lexer_number_with_unsigned_suffix() {
    let input = "123u";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::NumberLiteral(NumberLiteralToken { value, .. })) => {
            assert_eq!(value, "123u");
        }
        other => panic!("Expected NumberLiteral with 'u' suffix, got {:?}", other),
    }
}

#[test]
fn test_lexer_number_with_unsigned_suffix_uppercase() {
    let input = "456U";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::NumberLiteral(NumberLiteralToken { value, .. })) => {
            assert_eq!(value, "456U");
        }
        other => panic!("Expected NumberLiteral with 'U' suffix, got {:?}", other),
    }
}

#[test]
fn test_lexer_number_with_long_suffix() {
    let input = "789L";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::NumberLiteral(NumberLiteralToken { value, .. })) => {
            assert_eq!(value, "789L");
        }
        other => panic!("Expected NumberLiteral with 'L' suffix, got {:?}", other),
    }
}

#[test]
fn test_lexer_number_with_long_long_suffix() {
    let input = "999LL";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::NumberLiteral(NumberLiteralToken { value, .. })) => {
            assert_eq!(value, "999LL");
        }
        other => panic!("Expected NumberLiteral with 'LL' suffix, got {:?}", other),
    }
}

#[test]
fn test_lexer_number_with_unsigned_long_suffix() {
    let input = "0xFFul";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::NumberLiteral(NumberLiteralToken { value, .. })) => {
            assert_eq!(value, "0xFFul");
        }
        other => panic!("Expected NumberLiteral with 'ul' suffix, got {:?}", other),
    }
}

#[test]
fn test_lexer_number_with_unsigned_long_long_suffix() {
    let input = "1234ULL";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::NumberLiteral(NumberLiteralToken { value, .. })) => {
            assert_eq!(value, "1234ULL");
        }
        other => panic!("Expected NumberLiteral with 'ULL' suffix, got {:?}", other),
    }
}

#[test]
fn test_lexer_hex_with_suffix() {
    let input = "0x100u";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::NumberLiteral(NumberLiteralToken { value, .. })) => {
            assert_eq!(value, "0x100u");
        }
        other => panic!("Expected NumberLiteral, got {:?}", other),
    }
}

#[test]
fn test_lexer_octal_with_suffix() {
    let input = "0755L";
    let mut lexer = Lexer::new(input);
    
    match lexer.next_token() {
        Some(Token::NumberLiteral(NumberLiteralToken { value, .. })) => {
            assert_eq!(value, "0755L");
        }
        other => panic!("Expected NumberLiteral, got {:?}", other),
    }
}

#[test]
fn test_lexer_integer_vs_float_with_large_l_suffix() {
    // 整数 + L = NumberLiteral (long int)
    let input1 = "123L";
    let mut lexer1 = Lexer::new(input1);
    match lexer1.next_token() {
        Some(Token::NumberLiteral(NumberLiteralToken { value, .. })) => {
            assert_eq!(value, "123L");
        }
        other => panic!("Expected NumberLiteral for '123L', got {:?}", other),
    }
    
    // 浮動小数点 + L = FloatLiteral (long double)
    let input2 = "123.0L";
    let mut lexer2 = Lexer::new(input2);
    match lexer2.next_token() {
        Some(Token::FloatLiteral(FloatLiteralToken { value, .. })) => {
            assert_eq!(value, "123.0L");
        }
        other => panic!("Expected FloatLiteral for '123.0L', got {:?}", other),
    }
    
    // 指数表記 + L = FloatLiteral (long double)
    let input3 = "1e5L";
    let mut lexer3 = Lexer::new(input3);
    match lexer3.next_token() {
        Some(Token::FloatLiteral(FloatLiteralToken { value, .. })) => {
            assert_eq!(value, "1e5L");
        }
        other => panic!("Expected FloatLiteral for '1e5L', got {:?}", other),
    }
}
