use crate::lexer::Lexer;
use crate::token::*;

#[test]
fn test_lexer_backslash_error_simple() {
    let s = "int x\\";
    let mut lx = Lexer::new(s);

    // int トークン
    let token1 = lx.next_token();
    assert!(matches!(token1, Some(Token::Int(_))));

    // x トークン
    let token2 = lx.next_token();
    assert!(matches!(token2, Some(Token::Ident(_))));

    // バックスラッシュでエラー
    let token3 = lx.next_token();
    match token3 {
        Some(Token::Error(ErrorToken { message, .. })) => {
            assert!(message.contains("Line continuation"));
            assert!(message.contains("not supported"));
            assert!(message.contains("preprocessor directives"));
        }
        _ => panic!("Expected Error token for backslash"),
    }
}

#[test]
fn test_lexer_backslash_with_newline_error() {
    let s = "int x \\\n= 10;";
    let mut lx = Lexer::new(s);

    // int, x をスキップ
    lx.next_token();
    lx.next_token();

    // バックスラッシュ + 改行でエラー
    let token = lx.next_token();
    match token {
        Some(Token::Error(ErrorToken { message, span })) => {
            assert!(message.contains("Line continuation"));
            assert_eq!(span.start_line, 0);
        }
        _ => panic!("Expected Error token, got {:?}", token),
    }
}

#[test]
fn test_lexer_backslash_crlf_error() {
    let s = "int x \\\r\n= 10;";
    let mut lx = Lexer::new(s);

    // int, x をスキップ
    lx.next_token();
    lx.next_token();

    // バックスラッシュ + CRLF でエラー
    let token = lx.next_token();
    match token {
        Some(Token::Error(ErrorToken { message, .. })) => {
            assert!(message.contains("Line continuation"));
        }
        _ => panic!("Expected Error token"),
    }
}

#[test]
fn test_lexer_preprocessor_backslash_ok() {
    // プリプロセッサ内のバックスラッシュはOK
    let s = "#define MACRO \\\nVALUE\n";
    let mut lx = Lexer::new(s);

    let token = lx.next_token();
    match token {
        Some(Token::Define(DefineToken { macro_name, macro_value, .. })) => {
            assert_eq!(macro_name, "MACRO");
            assert_eq!(macro_value, "VALUE");
        }
        _ => panic!("Expected Define token, got {:?}", token),
    }
}

#[test]
fn test_lexer_multiple_backslash_errors() {
    let s = "int x\\, y\\;";
    let mut lx = Lexer::new(s);

    // int, x をスキップ
    lx.next_token();
    lx.next_token();

    // 最初のバックスラッシュ
    let token1 = lx.next_token();
    assert!(matches!(token1, Some(Token::Error(_))));

    // カンマ
    let token2 = lx.next_token();
    assert!(matches!(token2, Some(Token::Comma(_))));

    // y
    let token3 = lx.next_token();
    assert!(matches!(token3, Some(Token::Ident(_))));

    // 2つ目のバックスラッシュ
    let token4 = lx.next_token();
    assert!(matches!(token4, Some(Token::Error(_))));
}

#[test]
fn test_lexer_backslash_error_message_format() {
    let s = "\\";
    let mut lx = Lexer::new(s);

    let token = lx.next_token();
    match token {
        Some(Token::Error(ErrorToken { message, span })) => {
            // メッセージに必要な情報が含まれているか確認
            assert!(message.contains("backslash") || message.contains("continuation"));
            assert!(message.contains("not supported"));
            assert!(message.contains("README.md"));
            
            // spanが正しく設定されているか
            assert_eq!(span.byte_start_idx, 0);
            assert!(span.byte_end_idx > 0);
        }
        _ => panic!("Expected Error token"),
    }
}

#[test]
fn test_lexer_backslash_at_end_of_file() {
    let s = "int x \\";
    let mut lx = Lexer::new(s);

    lx.next_token(); // int
    lx.next_token(); // x

    let token = lx.next_token();
    assert!(matches!(token, Some(Token::Error(_))));
}
