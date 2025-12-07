#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::ast::Item;

    #[test]
    fn test_parser_includes_produced_items() {
        let s = "#include \"a.h\"\n#include <b.h>\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 2);

        match &tu.items[0] {
            Item::Include { text, filename, .. } => {
                assert_eq!(filename, "a.h");
                assert_eq!(text, "#include \"a.h\"\n");
            }
            _ => panic!("first item is not Include"),
        }

        match &tu.items[1] {
            Item::Include { text, filename, .. } => {
                assert_eq!(filename, "b.h");
                assert_eq!(text, "#include <b.h>\n");
            }
            _ => panic!("second item is not Include"),
        }
    }

    #[test]
    fn test_parser_defines_produced_items() {
        let s = "#define A 1\n#define B 2\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 2);

        match &tu.items[0] {
            Item::Define { text, macro_name, macro_value, .. } => {
                assert_eq!(macro_name, "A");
                assert_eq!(macro_value, "1");
                assert_eq!(text, "#define A 1\n");
            }
            _ => panic!("first item is not Define"),
        }

        match &tu.items[1] {
            Item::Define { text, macro_name, macro_value, .. } => {
                assert_eq!(macro_name, "B");
                assert_eq!(macro_value, "2");
                assert_eq!(text, "#define B 2\n");
            }
            _ => panic!("second item is not Define"),
        }
    }

    #[test]
    fn test_parser_with_japanese_content() {
        let s = "/* はじまり */\n#define A B\n#include \"XXX.h\"\n/* おわり */\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 4, "Should parse 4 items");

        for item in &tu.items {
            match item {
                Item::BlockComment { text, .. } => {
                    assert!(text.contains("はじまり") || text.contains("おわり"));
                }
                Item::Define { text, macro_name, macro_value, .. } => {
                    assert_eq!(macro_name, "A");
                    assert_eq!(macro_value, "B");
                    assert!(text.contains("#define A B"));
                }
                Item::Include { text, filename, .. } => {
                    assert_eq!(filename, "XXX.h");
                    assert!(text.contains("#include \"XXX.h\""));
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_parser_simple_var_decl() {
        let s = "int x;\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 1);

        match &tu.items[0] {
            Item::VarDecl { text, var_name, has_initializer, .. } => {
                assert_eq!(var_name, "x");
                assert_eq!(*has_initializer, false);
                assert_eq!(text, "int x;");
            }
            _ => panic!("Expected VarDecl item"),
        }
    }

    #[test]
    fn test_parser_var_decl_with_initializer() {
        let s = "int x = 10;\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 1);

        match &tu.items[0] {
            Item::VarDecl { text, var_name, has_initializer, .. } => {
                assert_eq!(var_name, "x");
                assert_eq!(*has_initializer, true);
                assert!(text.contains("int x ="));
            }
            _ => panic!("Expected VarDecl item"),
        }
    }

    #[test]
    fn test_parser_var_decl_with_storage_class() {
        let s = "static int counter;\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 1);

        match &tu.items[0] {
            Item::VarDecl { text, var_name, has_initializer, .. } => {
                assert_eq!(var_name, "counter");
                assert_eq!(*has_initializer, false);
                assert_eq!(text, "static int counter;");
            }
            _ => panic!("Expected VarDecl item"),
        }
    }

    #[test]
    fn test_parser_var_decl_with_qualifiers() {
        let s = "const volatile int value = 42;\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 1);

        match &tu.items[0] {
            Item::VarDecl { text, var_name, has_initializer, .. } => {
                assert_eq!(var_name, "value");
                assert_eq!(*has_initializer, true);
                assert!(text.contains("const"));
                assert!(text.contains("volatile"));
            }
            _ => panic!("Expected VarDecl item"),
        }
    }

    #[test]
    fn test_parser_multiple_var_decls() {
        let s = "int a;\nfloat b = 3.14;\nstatic char c;\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 3);

        match &tu.items[0] {
            Item::VarDecl { var_name, .. } => assert_eq!(var_name, "a"),
            _ => panic!("Expected VarDecl"),
        }

        match &tu.items[1] {
            Item::VarDecl { var_name, has_initializer, .. } => {
                assert_eq!(var_name, "b");
                assert_eq!(*has_initializer, true);
            }
            _ => panic!("Expected VarDecl"),
        }

        match &tu.items[2] {
            Item::VarDecl { var_name, .. } => assert_eq!(var_name, "c"),
            _ => panic!("Expected VarDecl"),
        }
    }

    #[test]
    fn test_parser_mixed_items() {
        let s = "/* comment */\n#include <stdio.h>\nint x = 5;\ntypedef int myint;\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 4);
        
        assert!(matches!(&tu.items[0], Item::BlockComment { .. }));
        assert!(matches!(&tu.items[1], Item::Include { .. }));
        assert!(matches!(&tu.items[2], Item::VarDecl { .. }));
        assert!(matches!(&tu.items[3], Item::TypedefDecl { .. }));
    }

    #[test]
    fn test_parser_simple_struct_decl() {
        let s = "struct Point { int x; int y; };\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 1);

        match &tu.items[0] {
            Item::StructDecl { text, struct_name, has_typedef, .. } => {
                assert_eq!(struct_name.as_ref().unwrap(), "Point");
                assert_eq!(*has_typedef, false);
                assert!(text.contains("struct Point"));
            }
            _ => panic!("Expected StructDecl item"),
        }
    }

    #[test]
    fn test_parser_anonymous_struct() {
        let s = "struct { int x; int y; } point;\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 1);

        match &tu.items[0] {
            Item::StructDecl { struct_name, .. } => {
                assert!(struct_name.is_none() || struct_name.as_ref().unwrap() == "point");
            }
            _ => panic!("Expected StructDecl item"),
        }
    }

    #[test]
    fn test_parser_typedef_struct() {
        let s = "typedef struct { int x; int y; } Point;\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 1);

        match &tu.items[0] {
            Item::StructDecl { text, has_typedef, .. } => {
                assert_eq!(*has_typedef, true);
                assert!(text.contains("typedef struct"));
            }
            _ => panic!("Expected StructDecl item"),
        }
    }

    #[test]
    fn test_parser_typedef_struct_with_name() {
        let s = "typedef struct Point { int x; int y; } Point;\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 1);

        match &tu.items[0] {
            Item::StructDecl { text, struct_name, has_typedef, .. } => {
                assert_eq!(struct_name.as_ref().unwrap(), "Point");
                assert_eq!(*has_typedef, true);
                assert!(text.contains("typedef struct Point"));
            }
            _ => panic!("Expected StructDecl item"),
        }
    }

    #[test]
    fn test_parser_struct_variable_decl() {
        let s = "struct Point p;\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 1);

        match &tu.items[0] {
            Item::StructDecl { text, struct_name, .. } => {
                assert_eq!(struct_name.as_ref().unwrap(), "Point");
                assert!(text.contains("struct Point p"));
            }
            _ => panic!("Expected StructDecl item"),
        }
    }

    #[test]
    fn test_parser_nested_struct() {
        let s = "struct Outer { struct Inner { int val; } inner; int x; };\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 1);

        match &tu.items[0] {
            Item::StructDecl { text, struct_name, .. } => {
                assert_eq!(struct_name.as_ref().unwrap(), "Outer");
                assert!(text.contains("struct Outer"));
                assert!(text.contains("struct Inner"));
            }
            _ => panic!("Expected StructDecl item"),
        }
    }
}
