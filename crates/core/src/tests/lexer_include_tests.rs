use crate::lexer::Lexer;
use crate::token::*;

#[test]
fn test_lexer_include_angle() {
        let s = "#include <stdio.h>\n";
        let mut lx = Lexer::new(s);

        while let Some(token) = lx.next_token() {
            match token {
                Token::Include(IncludeToken { span, filename }) => {
                    assert_eq!(span.start_line, 0);
                    assert_eq!(span.start_column, 0);
                    assert_eq!(span.end_line, 1);
                    assert_eq!(span.end_column, 0);
                    assert_eq!(filename, "stdio.h");
                    assert_eq!(span.byte_start_idx, 0);
                    assert_eq!(span.byte_end_idx, s.len());
                    assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "#include <stdio.h>\n");
                    return;
                }
                _ => {}
            }
        }
        panic!("Include token not found");
    }

    #[test]
    fn test_lexer_include_quote() {
        let s = "#include \"file.h\"\n";
        let mut lx = Lexer::new(s);

        while let Some(token) = lx.next_token() {
            match token {
                Token::Include(IncludeToken { filename, span }) => {
                    assert_eq!(filename, "file.h");
                    assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "#include \"file.h\"\n");
                    return;
                }
                _ => {}
            }
        }
        panic!("Include token not found");
    }

    #[test]
    fn test_lexer_include_missing_closer() {
        let s1 = "#include <path/to/file\n";
        let mut lx1 = Lexer::new(s1);
        while let Some(token) = lx1.next_token() {
            match token {
                Token::Include(IncludeToken { filename, span }) => {
                    assert_eq!(filename, "path/to/file");
                    assert_eq!(&s1[span.byte_start_idx..span.byte_end_idx], "#include <path/to/file\n");
                    break;
                }
                _ => {}
            }
        }

        let s2 = "#include \"another/path\n";
        let mut lx2 = Lexer::new(s2);
        while let Some(token) = lx2.next_token() {
            match token {
                Token::Include(IncludeToken { filename, span }) => {
                    assert_eq!(filename, "another/path");
                    assert_eq!(&s2[span.byte_start_idx..span.byte_end_idx], "#include \"another/path\n");
                    break;
                }
                _ => {}
            }
        }
    }
