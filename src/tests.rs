#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::formatter::Formatter;
    use crate::token::Token;
    use crate::ast::{TranslationUnit, Item};
    use crate::span::Span;

    #[test]
    fn test_basic_iteration_and_pos() {
        let s = "ab\nc";
        let mut lx = Lexer::new(s);

        let (_, ch) = lx.peek().unwrap();
        assert_eq!(ch, 'a');
        // assert_eq!(s[byte_idx], 'a');

        // read 'a'
        assert_eq!(lx.next_char(), Some((0, 'a')));
        assert_eq!(lx.pos_index(), 1);
        assert_eq!(lx.line, 0);
        assert_eq!(lx.column, 1);

        // read 'b'
        assert_eq!(lx.next_char(), Some((1, 'b')));
        assert_eq!(lx.pos_index(), 2);
        assert_eq!(lx.line, 0);
        assert_eq!(lx.column, 2);

        // read '\n'
        assert_eq!(lx.next_char(), Some((2, '\n')));
        assert_eq!(lx.pos_index(), 3);
        assert_eq!(lx.line, 1);
        assert_eq!(lx.column, 0);

        // read 'c'
        assert_eq!(lx.next_char(), Some((3, 'c')));
        assert_eq!(lx.pos_index(), 4);
        assert_eq!(lx.line, 1);
        assert_eq!(lx.column, 1);

    }

    #[test]
    fn test_multibyte_chars() {
        // 'é' is multibyte in UTF-8
        let s = "aéb";
        let mut lx = Lexer::new(s);

        let mut got = Vec::new();
        while let Some((_, ch)) = lx.next_char() {
            got.push(ch);
        }
        assert_eq!(got, vec!['a', 'é', 'b']);
        assert_eq!(lx.pos_index(), 3);
    }

    
    #[test]
    fn test_lexer_block_comment() {
        let s = "/* comment */";
        let mut lx = Lexer::new(s);

        // Skip to the block comment
        while let Some(token) = lx.next_token() {
            match token {
                Token::BlockComment { span } => {
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

        // Skip to the block comment
        while let Some(token) = lx.next_token() {
            match token {
                Token::BlockComment { span } => {
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

        // Skip to the block comment
        while let Some(token) = lx.next_token() {
            match token {
                Token::BlockComment { span } => {
                    assert_eq!(span.start_line, 0);
                    assert_eq!(span.start_column, 0);
                    assert_eq!(span.end_line, 1);
                    assert_eq!(span.end_column, 10);
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

    #[test]
    fn test_formatter_format_tu_trims_leading_whitespace() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        let item = Item::BlockComment { span, text: String::from("   /* hello */") };
        let tu = TranslationUnit { items: vec![item] };
        let fmt = Formatter::new();
        let out = fmt.format_tu(&tu);
        assert_eq!(out, "/* hello */");
    }

    #[test]
    fn test_formatter_original_tu_preserves_texts() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        let item1 = Item::BlockComment { span: span.clone(), text: String::from("/* one */") };
        let item2 = Item::BlockComment { span, text: String::from("/* two */") };
        let tu = TranslationUnit { items: vec![item1, item2] };
        let fmt = Formatter::new();
        let out = fmt.original_tu(&tu);
        assert_eq!(out, "/* one *//* two */");
    }

    #[test]
    fn test_formatter_keeps_newline_in_leading_whitespace() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        let item = Item::BlockComment { span, text: String::from("\t\r\n /* hello */") };
        let tu = TranslationUnit { items: vec![item] };
        let fmt = Formatter::new();
        let out = fmt.format_tu(&tu);
        assert_eq!(out, "\n/* hello */");
    }

    #[test]
    fn test_formatter_keeps_multiple_newlines() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        let item = Item::BlockComment { span, text: String::from("\n\n  /* ok */") };
        let tu = TranslationUnit { items: vec![item] };
        let fmt = Formatter::new();
        let out = fmt.format_tu(&tu);
        assert_eq!(out, "\n\n/* ok */");
    }

    #[test]
    fn test_lexer_include_angle() {
        let s = "#include <stdio.h>\n";
        let mut lx = Lexer::new(s);

        while let Some(token) = lx.next_token() {
            match token {
                Token::Include { span, filename } => {
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
                Token::Include { filename, span } => {
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
        // missing closing '>' or '"', lexer should take rest as filename
        let s1 = "#include <path/to/file\n";
        let mut lx1 = Lexer::new(s1);
        while let Some(token) = lx1.next_token() {
            match token {
                Token::Include { filename, span } => {
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
                Token::Include { filename, span } => {
                    assert_eq!(filename, "another/path");
                    assert_eq!(&s2[span.byte_start_idx..span.byte_end_idx], "#include \"another/path\n");
                    break;
                }
                _ => {}
            }
        }
    }

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
    fn test_lexer_define_simple() {
        let s = "#define MAX 10\n";
        let mut lx = Lexer::new(s);

        while let Some(token) = lx.next_token() {
            match token {
                Token::Define { macro_name, macro_value, span } => {
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
                Token::Define { macro_name, macro_value, span } => {
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
    fn test_formatter_format_define_keeps_newline_only() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        let text = String::from("\t\r\n  #define Z 42\n");
        let item = Item::Define { span, text: text.clone(), macro_name: "Z".into(), macro_value: "42".into() };
        let tu = TranslationUnit { items: vec![item] };
        let fmt = Formatter::new();
        let out = fmt.format_tu(&tu);
        // leading \t\r and spaces removed, newline kept, then the rest starts with '#'
        assert_eq!(out, "\n#define Z 42\n");
    }

    #[test]
    fn test_lexer_typedef_simple() {
        let s = "typedef";
        let mut lx = Lexer::new(s);

        while let Some(token) = lx.next_token() {
            match token {
                Token::Typedef { span } => {
                    assert_eq!(span.start_line, 0);
                    assert_eq!(span.start_column, 0);
                    assert_eq!(span.end_line, 0);
                    assert_eq!(span.end_column, s.len());
                    assert_eq!(span.byte_start_idx, 0);
                    assert_eq!(span.byte_end_idx, s.len());
                    assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "typedef");
                    return;
                }
                _ => { panic!("Unexpected token"); }
            }
        }
        panic!("Typedef token not found");
    }

    #[test]
    fn test_lexer_typedef_with_leading_whitespace() {
        let s = "  \t typedef int MyInt;";
        let mut lx = Lexer::new(s);

        while let Some(token) = lx.next_token() {
            match token {
                Token::Typedef { span } => {
                    assert_eq!(span.start_line, 0);
                    assert_eq!(span.start_column, 0);
                    // 先頭の空白も含まれる
                    assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "  \t typedef");
                    return;
                }
                _ => {}
            }
        }
        panic!("Typedef token not found");
    }

    #[test]
    fn test_lexer_typedef_case_sensitive() {
        // TYPEDEF（大文字）は識別子扱いになるはず
        let s = "TYPEDEF int MyInt;";
        let mut lx = Lexer::new(s);

        let token = lx.next_token();

        match token {
            Some(Token::Ident { span, name }) => {
                assert_eq!(name, "TYPEDEF");
                assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "TYPEDEF");
            }
            _ => {
                panic!("Expected Ident token for 'TYPEDEF'");
            }
        }
    }

    #[test]
    fn test_lexer_typedef_multiple() {
        let s = "typedef int A;\ntypedef float B;";
        let mut lx = Lexer::new(s);

        let mut typedef_count = 0;
        while let Some(token) = lx.next_token() {
            if let Token::Typedef { .. } = token {
                typedef_count += 1;
            }
        }
        assert_eq!(typedef_count, 2, "Should find two typedef keywords");
    }

    #[test]
    fn test_lexer_define_with_japanese_after() {
        // 日本語文字の直前でトークンが終わるケースをテスト
        let s = "#define A B\n#include \"XXX.h\" // XXX.h をインクルード\n";
        let mut lx = Lexer::new(s);

        // 最初のトークンは #define
        let token1 = lx.next_token();
        match token1 {
            Some(Token::Define { macro_name, macro_value, span }) => {
                assert_eq!(macro_name, "A");
                assert_eq!(macro_value, "B");
                // バイト境界が正しいことを確認（パニックしないこと）
                assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "#define A B\n");
            }
            _ => panic!("Expected Define token"),
        }

        // 次のトークンは #include
        let token2 = lx.next_token();
        match token2 {
            Some(Token::Include { filename, span }) => {
                assert_eq!(filename, "XXX.h");
                // バイト境界が正しいことを確認（パニックしないこと）
                let text = &s[span.byte_start_idx..span.byte_end_idx];
                assert!(text.starts_with("#include \"XXX.h\""));
            }
            _ => panic!("Expected Include token"),
        }
    }

    #[test]
    fn test_lexer_multiple_directives_with_japanese() {
        // example.txt と同じ構造をテスト
        let s = "/* はじまり */\n#define A B\n#include \"XXX.h\" // XXX.h をインクルード\n#include <YYY.h> /* YYY.h をインクルード */\n/* おわり */";
        let mut lx = Lexer::new(s);

        let mut token_count = 0;
        while let Some(token) = lx.next_token() {
            token_count += 1;
            match token {
                Token::BlockComment { span } => {
                    // パニックしないことを確認
                    let _ = &s[span.byte_start_idx..span.byte_end_idx];
                }
                Token::Define { span, .. } => {
                    // パニックしないことを確認
                    let _ = &s[span.byte_start_idx..span.byte_end_idx];
                }
                Token::Include { span, .. } => {
                    // パニックしないことを確認
                    let _ = &s[span.byte_start_idx..span.byte_end_idx];
                }
                Token::Ident { span, .. } => {
                    // パニックしないことを確認
                    let _ = &s[span.byte_start_idx..span.byte_end_idx];
                }
                _ => {}
            }
        }
        
        // すべてのトークンが正しく読み取れたことを確認
        assert!(token_count >= 5, "Should tokenize at least 5 tokens (2 comments, 1 define, 2 includes)");
    }

    #[test]
    fn test_parser_with_japanese_content() {
        // パーサーが日本語を含むコンテンツを正しく処理できることを確認
        let s = "/* はじまり */\n#define A B\n#include \"XXX.h\"\n/* おわり */\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 4, "Should parse 4 items");

        // 各アイテムのテキストがUTF-8境界で正しく切り出されていることを確認
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
    fn test_lexer_int_keyword() {
        let s = "int x;";
        let mut lx = Lexer::new(s);

        let token1 = lx.next_token();
        match token1 {
            Some(Token::Int { span }) => {
                assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "int");
            }
            _ => panic!("Expected Int token"),
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
    fn test_formatter_var_decl() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        let item = Item::VarDecl { 
            span, 
            text: String::from("  int x;"),
            var_name: String::from("x"),
            has_initializer: false,
        };
        let tu = TranslationUnit { items: vec![item] };
        let fmt = Formatter::new();
        let out = fmt.format_tu(&tu);
        assert_eq!(out, "int x;");
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
    fn test_lexer_struct_keyword() {
        let s = "struct Point { int x; int y; };";
        let mut lx = Lexer::new(s);

        let token1 = lx.next_token();
        match token1 {
            Some(Token::Struct { span }) => {
                assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "struct");
            }
            _ => panic!("Expected Struct token"),
        }
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
                // 無名構造体は point が変数名なので struct_name は None か point
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

    #[test]
    fn test_formatter_struct_decl() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        let item = Item::StructDecl {
            span,
            text: String::from("  struct Point { int x; };"),
            struct_name: Some(String::from("Point")),
            has_typedef: false,
        };
        let tu = TranslationUnit { items: vec![item] };
        let fmt = Formatter::new();
        let out = fmt.format_tu(&tu);
        assert_eq!(out, "struct Point { int x; };");
    }
}
