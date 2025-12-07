#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::token::*;

    #[test]
    fn test_lexer_block_comment() {
        let s = "/* comment */";
        let mut lx = Lexer::new(s);

        while let Some(token) = lx.next_token() {
            match token {
                Token::BlockComment(BlockCommentToken { span }) => {
                    assert_eq!(span.start_line, 0);
                    assert_eq!(span.start_column, 0);
                    assert_eq!(span.end_line, 0);
                    assert_eq!(span.end_column, 13);
                    assert_eq!(span.byte_start_idx, 0);
                    assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "/* comment */");
                    return;
                },
                _ => {
                    panic!("Unexpected token");
                }
            }
        }
        panic!("Block comment token not found");
    }

    #[test]
    fn test_lexer_block_comment_japanese() {
        let s = "/* コメント */";
        let mut lx = Lexer::new(s);

        while let Some(token) = lx.next_token() {
            match token {
                Token::BlockComment(BlockCommentToken { span }) => {
                    assert_eq!(span.start_line, 0);
                    assert_eq!(span.start_column, 0);
                    assert_eq!(span.end_line, 0);
                    assert_eq!(span.end_column, 10);
                    assert_eq!(span.byte_start_idx, 0);
                    assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "/* コメント */");
                    return;
                }
                _ => {
                    panic!("Unexpected token");
                }
            }
        }
        panic!("Block comment token not found");
    }

    #[test]
    fn test_lexer_block_comment_japanese_with_spaces() {
        let s = "\t\r\n /* コメント*/";
        let mut lx = Lexer::new(s);

        while let Some(token) = lx.next_token() {
            match token {
                Token::BlockComment(BlockCommentToken { span }) => {
                    assert_eq!(span.start_line, 0);
                    assert_eq!(span.start_column, 0);
                    assert_eq!(span.end_line, 1);
                    assert_eq!(span.end_column, 11); // '/' の次の位置
                    assert_eq!(span.byte_start_idx, 0);
                    assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "\t\r\n /* コメント*/");
                    return;
                },
                _ => {
                    panic!("Unexpected token");
                }
            }
        }
        panic!("Block comment token not found");
    }
}
