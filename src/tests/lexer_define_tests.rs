use crate::lexer::Lexer;
use crate::token::*;

#[test]
fn test_lexer_define_simple() {
        let s = "#define MAX 10\n";
        let mut lx = Lexer::new(s);

        while let Some(token) = lx.next_token() {
            match token {
                Token::Define(DefineToken { macro_name, macro_value, span }) => {
                    assert_eq!(macro_name, "MAX");
                    assert_eq!(macro_value, "10");
                    assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "#define MAX 10\n");
                    return;
                }
                _ => {}
            }
        }
        panic!("Define token not found");
    }

    #[test]
    fn test_lexer_define_leading_whitespace_included() {
        let s = "\t \r #define X 1\n";
        let mut lx = Lexer::new(s);

        while let Some(token) = lx.next_token() {
            match token {
                Token::Define(DefineToken { macro_name, macro_value, span }) => {
                    assert_eq!(macro_name, "X");
                    assert_eq!(macro_value, "1");
                    assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "\t \r #define X 1\n");
                    return;
                }
                _ => {}
            }
        }
        panic!("Define token not found");
    }

    #[test]
    fn test_lexer_define_with_japanese_after() {
        let s = "#define A B\n#include \"XXX.h\" // XXX.h をインクルード\n";
        let mut lx = Lexer::new(s);

        let token1 = lx.next_token();
        match token1 {
            Some(Token::Define(DefineToken { macro_name, macro_value, span })) => {
                assert_eq!(macro_name, "A");
                assert_eq!(macro_value, "B");
                assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "#define A B\n");
            }
            _ => panic!("Expected Define token"),
        }

        let token2 = lx.next_token();
        match token2 {
            Some(Token::Include(IncludeToken { filename, span })) => {
                assert_eq!(filename, "XXX.h");
                let text = &s[span.byte_start_idx..span.byte_end_idx];
                assert!(text.starts_with("#include \"XXX.h\""));
            }
            _ => panic!("Expected Include token"),
        }
    }
