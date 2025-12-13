use crate::lexer::Lexer;
use crate::token::*;

#[test]
fn test_lexer_define_line_continuation_backslash_n() {
    let s = "#define LONG_MACRO \\\nVALUE_ON_NEXT_LINE\n";
    let mut lx = Lexer::new(s);

    while let Some(token) = lx.next_token() {
        match token {
            Token::Define(DefineToken { macro_name, macro_value, span: _ }) => {
                assert_eq!(macro_name, "LONG_MACRO");
                // 行継続記号は削除される
                assert_eq!(macro_value, "VALUE_ON_NEXT_LINE");
                return;
            }
            _ => {}
        }
    }
    panic!("Define token with line continuation not found");
}

#[test]
fn test_lexer_define_line_continuation_backslash_r_n() {
    let s = "#define LONG_MACRO \\\r\nVALUE_ON_NEXT_LINE\n";
    let mut lx = Lexer::new(s);

    while let Some(token) = lx.next_token() {
        match token {
            Token::Define(DefineToken { macro_name, macro_value, span: _ }) => {
                assert_eq!(macro_name, "LONG_MACRO");
                // 行継続記号は削除される
                assert_eq!(macro_value, "VALUE_ON_NEXT_LINE");
                return;
            }
            _ => {}
        }
    }
    panic!("Define token with line continuation (CRLF) not found");
}

#[test]
fn test_lexer_define_multiple_line_continuation() {
    let s = "#define MULTI \\\nLINE1 \\\nLINE2 \\\nLINE3\n";
    let mut lx = Lexer::new(s);

    while let Some(token) = lx.next_token() {
        match token {
            Token::Define(DefineToken { macro_name, macro_value, span: _ }) => {
                assert_eq!(macro_name, "MULTI");
                // 行継続記号は削除される
                assert_eq!(macro_value, "LINE1 LINE2 LINE3");
                return;
            }
            _ => {}
        }
    }
    panic!("Define token with multiple line continuations not found");
}

#[test]
fn test_lexer_include_line_continuation() {
    let s = "#include \\\n\"file.h\"\n";
    let mut lx = Lexer::new(s);

    while let Some(token) = lx.next_token() {
        match token {
            Token::Include(IncludeToken { filename, span: _ }) => {
                // 行継続記号は削除される
                assert_eq!(filename, "file.h");
                return;
            }
            _ => {}
        }
    }
    panic!("Include token with line continuation not found");
}

#[test]
fn test_lexer_ifdef_line_continuation() {
    let s = "#ifdef \\\nDEBUG\n";
    let mut lx = Lexer::new(s);

    while let Some(token) = lx.next_token() {
        match token {
            Token::Ifdef(IfdefToken { span }) => {
                // バックスラッシュ + 改行は元のテキストには含まれるが削除される
                // span で元のテキストを確認
                let content = &s[span.byte_start_idx..span.byte_end_idx];
                assert!(content.contains("DEBUG"));
                return;
            }
            _ => {}
        }
    }
    panic!("Ifdef token with line continuation not found");
}

#[test]
fn test_lexer_if_line_continuation() {
    let s = "#if defined(A) \\\n&& defined(B)\n";
    let mut lx = Lexer::new(s);

    while let Some(token) = lx.next_token() {
        match token {
            Token::If(IfToken { span }) => {
                let content = &s[span.byte_start_idx..span.byte_end_idx];
                assert!(content.contains("\\\n"));
                assert!(content.contains("&& defined(B)"));
                return;
            }
            _ => {}
        }
    }
    panic!("If token with line continuation not found");
}

#[test]
fn test_lexer_line_continuation_no_backslash() {
    // バックスラッシュがない場合は行継続されない
    let s = "#define SINGLE\nLINE\n";
    let mut lx = Lexer::new(s);

    let token1 = lx.next_token();
    match token1 {
        Some(Token::Define(DefineToken { macro_name, macro_value, .. })) => {
            assert_eq!(macro_name, "SINGLE");
            assert_eq!(macro_value, "");
        }
        _ => panic!("Expected Define token"),
    }

    // 次のトークンは "LINE" という識別子
    let token2 = lx.next_token();
    match token2 {
        Some(Token::Ident(IdentToken { name, .. })) => {
            assert_eq!(name, "LINE");
        }
        _ => panic!("Expected Ident token for LINE"),
    }
}
