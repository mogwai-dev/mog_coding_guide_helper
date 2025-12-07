#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::token::Token;

    #[test]
    fn test_lexer_typedef_simple() {
        let s = "typedef int MyInt;\n";
        let mut lx = Lexer::new(s);

        while let Some(token) = lx.next_token() {
            match token {
                Token::Typedef { span } => {
                    assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "typedef");
                    return;
                }
                _ => {}
            }
        }
        panic!("Typedef token not found");
    }

    #[test]
    fn test_lexer_typedef_with_leading_whitespace() {
        let s = "   typedef unsigned long ulong;\n";
        let mut lx = Lexer::new(s);

        while let Some(token) = lx.next_token() {
            match token {
                Token::Typedef { span } => {
                    assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "   typedef");
                    return;
                }
                _ => {}
            }
        }
        panic!("Typedef token not found");
    }

    #[test]
    fn test_lexer_typedef_case_sensitive() {
        let s = "typedef float FLOAT;\n";
        let mut lx = Lexer::new(s);

        while let Some(token) = lx.next_token() {
            match token {
                Token::Typedef { span } => {
                    assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "typedef");
                    return;
                }
                _ => {}
            }
        }
        panic!("Typedef token not found");
    }

    #[test]
    fn test_lexer_typedef_multiple() {
        let s = "typedef int MyInt;\ntypedef char MyChar;\n";
        let mut lx = Lexer::new(s);

        let mut typedef_count = 0;

        while let Some(token) = lx.next_token() {
            match token {
                Token::Typedef { .. } => {
                    typedef_count += 1;
                }
                _ => {}
            }
        }

        assert_eq!(typedef_count, 2);
    }
}
