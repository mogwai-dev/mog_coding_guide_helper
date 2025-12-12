use crate::lexer::Lexer;
use crate::token::Token;

#[test]
fn test_lexer_arithmetic_operators() {
    let input = "a + b - c * d / e % f";
    let mut lexer = Lexer::new(input);
    
    // a
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_))));
    // +
    assert!(matches!(lexer.next_token(), Some(Token::Plus(_))));
    // b
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_))));
    // -
    assert!(matches!(lexer.next_token(), Some(Token::Minus(_))));
    // c
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_))));
    // *
    assert!(matches!(lexer.next_token(), Some(Token::Asterisk(_))));
    // d
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_))));
    // /
    assert!(matches!(lexer.next_token(), Some(Token::Slash(_))));
    // e
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_))));
    // %
    assert!(matches!(lexer.next_token(), Some(Token::Percent(_))));
    // f
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_))));
}

#[test]
fn test_lexer_comparison_operators() {
    let input = "a == b != c < d <= e > f >= g";
    let mut lexer = Lexer::new(input);
    
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // a
    assert!(matches!(lexer.next_token(), Some(Token::EqualEqual(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // b
    assert!(matches!(lexer.next_token(), Some(Token::NotEqual(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // c
    assert!(matches!(lexer.next_token(), Some(Token::LessThan(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // d
    assert!(matches!(lexer.next_token(), Some(Token::LessThanOrEqual(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // e
    assert!(matches!(lexer.next_token(), Some(Token::GreaterThan(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // f
    assert!(matches!(lexer.next_token(), Some(Token::GreaterThanOrEqual(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // g
}

#[test]
fn test_lexer_logical_operators() {
    let input = "a && b || c ! d";
    let mut lexer = Lexer::new(input);
    
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // a
    assert!(matches!(lexer.next_token(), Some(Token::AmpersandAmpersand(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // b
    assert!(matches!(lexer.next_token(), Some(Token::PipePipe(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // c
    assert!(matches!(lexer.next_token(), Some(Token::Exclamation(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // d
}

#[test]
fn test_lexer_bitwise_operators() {
    let input = "a & b | c ^ d ~ e << f >> g";
    let mut lexer = Lexer::new(input);
    
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // a
    assert!(matches!(lexer.next_token(), Some(Token::Ampersand(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // b
    assert!(matches!(lexer.next_token(), Some(Token::Pipe(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // c
    assert!(matches!(lexer.next_token(), Some(Token::Caret(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // d
    assert!(matches!(lexer.next_token(), Some(Token::Tilde(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // e
    assert!(matches!(lexer.next_token(), Some(Token::LeftShift(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // f
    assert!(matches!(lexer.next_token(), Some(Token::RightShift(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // g
}

#[test]
fn test_lexer_increment_decrement() {
    let input = "a++ b-- ++c --d";
    let mut lexer = Lexer::new(input);
    
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // a
    assert!(matches!(lexer.next_token(), Some(Token::PlusPlus(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // b
    assert!(matches!(lexer.next_token(), Some(Token::MinusMinus(_))));
    assert!(matches!(lexer.next_token(), Some(Token::PlusPlus(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // c
    assert!(matches!(lexer.next_token(), Some(Token::MinusMinus(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // d
}

#[test]
fn test_lexer_member_access() {
    let input = "a.b c->d";
    let mut lexer = Lexer::new(input);
    
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // a
    assert!(matches!(lexer.next_token(), Some(Token::Dot(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // b
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // c
    assert!(matches!(lexer.next_token(), Some(Token::Arrow(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // d
}

#[test]
fn test_lexer_array_and_ternary() {
    let input = "a[b] c ? d : e";
    let mut lexer = Lexer::new(input);
    
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // a
    assert!(matches!(lexer.next_token(), Some(Token::LeftBracket(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // b
    assert!(matches!(lexer.next_token(), Some(Token::RightBracket(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // c
    assert!(matches!(lexer.next_token(), Some(Token::Question(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // d
    assert!(matches!(lexer.next_token(), Some(Token::Colon(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // e
}

#[test]
fn test_lexer_comma() {
    let input = "int a, b, c;";
    let mut lexer = Lexer::new(input);
    
    assert!(matches!(lexer.next_token(), Some(Token::Int(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // a
    assert!(matches!(lexer.next_token(), Some(Token::Comma(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // b
    assert!(matches!(lexer.next_token(), Some(Token::Comma(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // c
    assert!(matches!(lexer.next_token(), Some(Token::Semicolon(_))));
}

#[test]
fn test_lexer_assignment_vs_equality() {
    let input = "a = b == c";
    let mut lexer = Lexer::new(input);
    
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // a
    assert!(matches!(lexer.next_token(), Some(Token::Equal(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // b
    assert!(matches!(lexer.next_token(), Some(Token::EqualEqual(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // c
}

#[test]
fn test_lexer_shift_vs_comparison() {
    let input = "a << b < c >> d > e";
    let mut lexer = Lexer::new(input);
    
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // a
    assert!(matches!(lexer.next_token(), Some(Token::LeftShift(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // b
    assert!(matches!(lexer.next_token(), Some(Token::LessThan(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // c
    assert!(matches!(lexer.next_token(), Some(Token::RightShift(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // d
    assert!(matches!(lexer.next_token(), Some(Token::GreaterThan(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // e
}

#[test]
fn test_lexer_bitwise_and_vs_logical_and() {
    let input = "a & b && c";
    let mut lexer = Lexer::new(input);
    
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // a
    assert!(matches!(lexer.next_token(), Some(Token::Ampersand(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // b
    assert!(matches!(lexer.next_token(), Some(Token::AmpersandAmpersand(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // c
}

#[test]
fn test_lexer_bitwise_or_vs_logical_or() {
    let input = "a | b || c";
    let mut lexer = Lexer::new(input);
    
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // a
    assert!(matches!(lexer.next_token(), Some(Token::Pipe(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // b
    assert!(matches!(lexer.next_token(), Some(Token::PipePipe(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Ident(_)))); // c
}
