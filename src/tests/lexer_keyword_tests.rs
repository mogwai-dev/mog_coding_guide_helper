#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::token::Token;

    #[test]
    fn test_lexer_int_keyword() {
        let s = "int x;\n";
        let mut lx = Lexer::new(s);

        let mut found_int = false;
        while let Some(token) = lx.next_token() {
            match token {
                Token::Int { .. } => {
                    found_int = true;
                }
                _ => {}
            }
        }

        assert!(found_int, "int keyword not found");
    }

    #[test]
    fn test_lexer_struct_keyword() {
        let s = "struct Point { int x; };\n";
        let mut lx = Lexer::new(s);

        let mut found_struct = false;
        while let Some(token) = lx.next_token() {
            match token {
                Token::Struct { .. } => {
                    found_struct = true;
                }
                _ => {}
            }
        }

        assert!(found_struct, "struct keyword not found");
    }

    #[test]
    fn test_lexer_multiple_directives_with_japanese() {
        let s = r#"
#include "aaa.h" // あああ
#define X 10
typedef int MyInt;
int main() { return 0; }
"#;
        let mut lx = Lexer::new(s);

        let mut include_count = 0;
        let mut define_count = 0;
        let mut typedef_count = 0;
        let mut int_count = 0;

        while let Some(token) = lx.next_token() {
            match token {
                Token::Include { .. } => include_count += 1,
                Token::Define { .. } => define_count += 1,
                Token::Typedef { .. } => typedef_count += 1,
                Token::Int { .. } => int_count += 1,
                _ => {}
            }
        }

        assert_eq!(include_count, 1);
        assert_eq!(define_count, 1);
        assert_eq!(typedef_count, 1);
        assert!(int_count >= 1); // at least one "int"
    }
}
