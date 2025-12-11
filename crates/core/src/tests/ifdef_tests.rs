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

        // 新しい入れ子構造: ConditionalBlockが1つ、その中にVarDeclとendifが含まれる
        assert_eq!(tu.items.len(), 1);
        
        // #ifdef
        if let Item::ConditionalBlock { directive_type, condition, items, .. } = &tu.items[0] {
            assert_eq!(directive_type, "ifdef");
            assert_eq!(condition, "DEBUG");
            assert_eq!(items.len(), 2); // VarDecl + endif
            
            // 中身: variable declaration
            if let Item::VarDecl { var_name, .. } = &items[0] {
                assert_eq!(var_name, "debug_mode");
            } else {
                panic!("Expected VarDecl inside ifdef");
            }
            
            // 中身: #endif
            if let Item::ConditionalBlock { directive_type, .. } = &items[1] {
                assert_eq!(directive_type, "endif");
            } else {
                panic!("Expected endif marker");
            }
        } else {
            panic!("Expected ConditionalBlock for ifdef");
        }
    }

    #[test]
    fn test_parser_ifndef() {
        let input = "#ifndef HEADER_H\n#define HEADER_H\n#endif\n";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let tu = parser.parse();

        // 新しい入れ子構造
        assert_eq!(tu.items.len(), 1);
        
        if let Item::ConditionalBlock { directive_type, condition, items, .. } = &tu.items[0] {
            assert_eq!(directive_type, "ifndef");
            assert_eq!(condition, "HEADER_H");
            assert_eq!(items.len(), 2); // Define + endif
            
            if let Item::Define { macro_name, .. } = &items[0] {
                assert_eq!(macro_name, "HEADER_H");
            } else {
                panic!("Expected Define inside ifndef");
            }
            
            if let Item::ConditionalBlock { directive_type, .. } = &items[1] {
                assert_eq!(directive_type, "endif");
            } else {
                panic!("Expected endif marker");
            }
        } else {
            panic!("Expected ConditionalBlock for ifndef");
        }
    }

    #[test]
    fn test_parser_if_defined() {
        let input = "#if defined(FEATURE)\nint feature_enabled = 1;\n#endif\n";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let tu = parser.parse();

        // 新しい入れ子構造
        assert_eq!(tu.items.len(), 1);
        
        if let Item::ConditionalBlock { directive_type, condition, items, .. } = &tu.items[0] {
            assert_eq!(directive_type, "if");
            assert!(condition.contains("defined(FEATURE)"));
            assert_eq!(items.len(), 2); // VarDecl + endif
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

        // 新しい入れ子構造: ifdef -> (items + elif -> (items + else -> (items + endif)))
        assert_eq!(tu.items.len(), 1);
        
        if let Item::ConditionalBlock { directive_type, condition, items, .. } = &tu.items[0] {
            assert_eq!(directive_type, "ifdef");
            assert_eq!(condition, "WINDOWS");
            assert_eq!(items.len(), 2); // VarDecl + elif block
            
            // VarDecl
            if let Item::VarDecl { var_name, .. } = &items[0] {
                assert_eq!(var_name, "os");
            } else {
                panic!("Expected VarDecl in ifdef");
            }
            
            // elif block
            if let Item::ConditionalBlock { directive_type, condition:_, items: elif_items, .. } = &items[1] {
                assert_eq!(directive_type, "elif");
                // conditionは空になっている（Lexerの実装による）
                assert_eq!(elif_items.len(), 3); // VarDecl + else block + endif
                
                // VarDecl in elif
                if let Item::VarDecl { var_name, .. } = &elif_items[0] {
                    assert_eq!(var_name, "os");
                } else {
                    panic!("Expected VarDecl in elif");
                }
                
                // else block
                if let Item::ConditionalBlock { directive_type, items: else_items, .. } = &elif_items[1] {
                    assert_eq!(directive_type, "else");
                    assert_eq!(else_items.len(), 1); // VarDecl のみ（endifは elif の sibling）
                    
                    // VarDecl in else
                    if let Item::VarDecl { var_name, .. } = &else_items[0] {
                        assert_eq!(var_name, "os");
                    } else {
                        panic!("Expected VarDecl in else");
                    }
                } else {
                    panic!("Expected else block");
                }
                
                // endif at elif level
                if let Item::ConditionalBlock { directive_type, .. } = &elif_items[2] {
                    assert_eq!(directive_type, "endif");
                } else {
                    panic!("Expected endif marker at elif level");
                }
            } else {
                panic!("Expected elif block");
            }
        } else {
            panic!("Expected ConditionalBlock for ifdef");
        }
    }

    #[test]
    fn test_parser_nested_ifdef() {
        let input = "#ifdef OUTER\n#ifdef INNER\nint x = 1;\n#endif\n#endif\n";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let tu = parser.parse();

        // 新しい入れ子構造: outer ifdef -> (inner ifdef -> (VarDecl + inner endif) + outer endif)
        assert_eq!(tu.items.len(), 1);
        
        if let Item::ConditionalBlock { directive_type, condition, items, .. } = &tu.items[0] {
            assert_eq!(directive_type, "ifdef");
            assert_eq!(condition, "OUTER");
            assert_eq!(items.len(), 2); // inner ifdef block + outer endif
            
            // inner ifdef
            if let Item::ConditionalBlock { directive_type, condition, items: inner_items, .. } = &items[0] {
                assert_eq!(directive_type, "ifdef");
                assert_eq!(condition, "INNER");
                assert_eq!(inner_items.len(), 2); // VarDecl + inner endif
                
                // VarDecl
                if let Item::VarDecl { var_name, .. } = &inner_items[0] {
                    assert_eq!(var_name, "x");
                } else {
                    panic!("Expected VarDecl");
                }
                
                // inner endif
                if let Item::ConditionalBlock { directive_type, .. } = &inner_items[1] {
                    assert_eq!(directive_type, "endif");
                } else {
                    panic!("Expected inner endif");
                }
            } else {
                panic!("Expected inner ifdef");
            }
            
            // outer endif
            if let Item::ConditionalBlock { directive_type, .. } = &items[1] {
                assert_eq!(directive_type, "endif");
            } else {
                panic!("Expected outer endif");
            }
        } else {
            panic!("Expected outer ifdef");
        }
    }
}
