#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::ast::Item;

    #[test]
    fn test_parser_ifdef_simple() {
        let input = "#ifdef DEBUG\nint debug_mode = 1;\n#endif\n";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 3);
        
        // #ifdef
        if let Item::ConditionalBlock { directive_type, .. } = &tu.items[0] {
            assert_eq!(directive_type, "ifdef");
        } else {
            panic!("Expected ConditionalBlock for ifdef");
        }

        // variable declaration
        if let Item::VarDecl { var_name, .. } = &tu.items[1] {
            assert_eq!(var_name, "debug_mode");
        } else {
            panic!("Expected VarDecl");
        }

        // #endif
        if let Item::ConditionalBlock { directive_type, .. } = &tu.items[2] {
            assert_eq!(directive_type, "endif");
        } else {
            panic!("Expected ConditionalBlock for endif");
        }
    }

    #[test]
    fn test_parser_ifndef() {
        let input = "#ifndef HEADER_H\n#define HEADER_H\n#endif\n";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 3);
        
        if let Item::ConditionalBlock { directive_type, .. } = &tu.items[0] {
            assert_eq!(directive_type, "ifndef");
        } else {
            panic!("Expected ConditionalBlock for ifndef");
        }

        if let Item::Define { macro_name, .. } = &tu.items[1] {
            assert_eq!(macro_name, "HEADER_H");
        } else {
            panic!("Expected Define");
        }

        if let Item::ConditionalBlock { directive_type, .. } = &tu.items[2] {
            assert_eq!(directive_type, "endif");
        } else {
            panic!("Expected ConditionalBlock for endif");
        }
    }

    #[test]
    fn test_parser_if_defined() {
        let input = "#if defined(FEATURE)\nint feature_enabled = 1;\n#endif\n";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 3);
        
        if let Item::ConditionalBlock { directive_type, .. } = &tu.items[0] {
            assert_eq!(directive_type, "if");
        } else {
            panic!("Expected ConditionalBlock for if");
        }
    }

    #[test]
    fn test_parser_ifdef_elif_else() {
        let input = "#ifdef WINDOWS\nint os = 1;\n#elif defined(LINUX)\nint os = 2;\n#else\nint os = 0;\n#endif\n";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 7);
        
        if let Item::ConditionalBlock { directive_type, .. } = &tu.items[0] {
            assert_eq!(directive_type, "ifdef");
        } else {
            panic!("Expected ConditionalBlock for ifdef");
        }

        if let Item::ConditionalBlock { directive_type, .. } = &tu.items[2] {
            assert_eq!(directive_type, "elif");
        } else {
            panic!("Expected ConditionalBlock for elif");
        }

        if let Item::ConditionalBlock { directive_type, .. } = &tu.items[4] {
            assert_eq!(directive_type, "else");
        } else {
            panic!("Expected ConditionalBlock for else");
        }

        if let Item::ConditionalBlock { directive_type, .. } = &tu.items[6] {
            assert_eq!(directive_type, "endif");
        } else {
            panic!("Expected ConditionalBlock for endif");
        }
    }

    #[test]
    fn test_parser_nested_ifdef() {
        let input = "#ifdef OUTER\n#ifdef INNER\nint x = 1;\n#endif\n#endif\n";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 5);
        
        if let Item::ConditionalBlock { directive_type, .. } = &tu.items[0] {
            assert_eq!(directive_type, "ifdef");
        } else {
            panic!("Expected outer ifdef");
        }

        if let Item::ConditionalBlock { directive_type, .. } = &tu.items[1] {
            assert_eq!(directive_type, "ifdef");
        } else {
            panic!("Expected inner ifdef");
        }

        if let Item::VarDecl { var_name, .. } = &tu.items[2] {
            assert_eq!(var_name, "x");
        } else {
            panic!("Expected VarDecl");
        }

        if let Item::ConditionalBlock { directive_type, .. } = &tu.items[3] {
            assert_eq!(directive_type, "endif");
        } else {
            panic!("Expected inner endif");
        }

        if let Item::ConditionalBlock { directive_type, .. } = &tu.items[4] {
            assert_eq!(directive_type, "endif");
        } else {
            panic!("Expected outer endif");
        }
    }
}
