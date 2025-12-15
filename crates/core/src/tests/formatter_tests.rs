use crate::formatter::Formatter;
use crate::ast::{TranslationUnit, Item};
use crate::span::Span;
use crate::trivia::Trivia;
use crate::type_system::{BaseType, Type, TypeQualifier, PointerLayer};

// BlockCommentはTriviaに移行したためコメントアウト
/*
#[test]
fn test_formatter_format_tu_trims_leading_whitespace() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        let item = Item::BlockComment { span, text: String::from("   /* hello */") };
        let tu = TranslationUnit { items: vec![item], leading_trivia: Trivia::empty() };
        let fmt = Formatter::new_no_header();
        let out = fmt.format_tu(&tu);
        assert_eq!(out, "/* hello */");
    }

    #[test]
    fn test_formatter_original_tu_preserves_texts() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        let item1 = Item::BlockComment { span: span.clone(), text: String::from("/* one */") };
        let item2 = Item::BlockComment { span, text: String::from("/* two */") };
        let tu = TranslationUnit { items: vec![item1, item2], leading_trivia: Trivia::empty() };
        let fmt = Formatter::new_no_header();
        let out = fmt.original_tu(&tu);
        assert_eq!(out, "/* one *//* two */");
    }

    #[test]
    fn test_formatter_keeps_newline_in_leading_whitespace() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        let item = Item::BlockComment { span, text: String::from("\t\r\n /* hello */") };
        let tu = TranslationUnit { items: vec![item], leading_trivia: Trivia::empty() };
        let fmt = Formatter::new_no_header();
        let out = fmt.format_tu(&tu);
        assert_eq!(out, "\n/* hello */");
    }

    #[test]
    fn test_formatter_keeps_multiple_newlines() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        let item = Item::BlockComment { span, text: String::from("\n\n  /* ok */") };
        let tu = TranslationUnit { items: vec![item], leading_trivia: Trivia::empty() };
        let fmt = Formatter::new_no_header();
        let out = fmt.format_tu(&tu);
        assert_eq!(out, "\n\n/* ok */");
    }
*/

    #[test]
    fn test_formatter_format_define_keeps_newline_only() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        let text = String::from("\t\r\n  #define Z 42\n");
        let item = Item::Define { span, text: text.clone(), macro_name: "Z".into(), macro_value: "42".into(), trivia: Trivia::empty() };
        let tu = TranslationUnit { items: vec![item], leading_trivia: Trivia::empty() };
        let fmt = Formatter::new_no_header();
        let out = fmt.format_tu(&tu);
        assert_eq!(out, "\n#define Z 42\n");
    }

    #[test]
    fn test_formatter_var_decl() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        let item = Item::VarDecl { 
            span, 
            text: String::from("  int x;"),
            var_name: String::from("x"),
            has_initializer: false,
            var_type: None,
            trivia: Trivia::empty(),
        };
        let tu = TranslationUnit { items: vec![item], leading_trivia: Trivia::empty() };
        let fmt = Formatter::new_no_header();
        let out = fmt.format_tu(&tu);
        assert_eq!(out, "int x;");
    }

    #[test]
    fn test_formatter_struct_decl() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        let item = Item::StructDecl {
            span,
            text: String::from("  struct Point { int x; };"),
            struct_name: Some(String::from("Point")),
            has_typedef: false,
            variable_names: Vec::new(),
            members: Vec::new(),
            trivia: Trivia::empty(),
        };
        let tu = TranslationUnit { items: vec![item], leading_trivia: Trivia::empty() };
        let fmt = Formatter::new_no_header();
        let out = fmt.format_tu(&tu);
        assert_eq!(out, "struct Point { int x; };");
    }

    #[test]
    fn test_formatter_var_decl_with_type_info() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        
        // Create type: int *
        let pointer_layer = PointerLayer::new(span.clone());
        let ty = Type::with_pointers(
            BaseType::Int,
            Vec::new(),
            vec![pointer_layer],
            span.clone(),
        );
        
        let item = Item::VarDecl { 
            span, 
            text: String::from("int *ptr;"),
            var_name: String::from("ptr"),
            has_initializer: false,
            var_type: Some(ty),
            trivia: Trivia::empty(),
        };
        
        let tu = TranslationUnit { items: vec![item], leading_trivia: Trivia::empty() };
        let fmt = Formatter::new_with_options(false, true);
        let out = fmt.format_tu(&tu);
        assert_eq!(out, "int * ptr;");
    }

    #[test]
    fn test_formatter_var_decl_with_const_pointer_type() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        
        // Create type: int *const
        let pointer_layer = PointerLayer::with_qualifiers(
            vec![TypeQualifier::Const],
            span.clone(),
        );
        let ty = Type::with_pointers(
            BaseType::Int,
            Vec::new(),
            vec![pointer_layer],
            span.clone(),
        );
        
        let item = Item::VarDecl { 
            span, 
            text: String::from("int *const ptr;"),
            var_name: String::from("ptr"),
            has_initializer: false,
            var_type: Some(ty),
            trivia: Trivia::empty(),
        };
        
        let tu = TranslationUnit { items: vec![item], leading_trivia: Trivia::empty() };
        let fmt = Formatter::new_with_options(false, true);
        let out = fmt.format_tu(&tu);
        assert_eq!(out, "int *const ptr;");
    }

    #[test]
    fn test_formatter_var_decl_without_type_info() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        
        let item = Item::VarDecl { 
            span, 
            text: String::from("int *ptr;"),
            var_name: String::from("ptr"),
            has_initializer: false,
            var_type: None,
            trivia: Trivia::empty(),
        };
        
        let tu = TranslationUnit { items: vec![item], leading_trivia: Trivia::empty() };
        // use_type_info=true but var_type is None, should fall back to text
        let fmt = Formatter::new_with_options(false, true);
        let out = fmt.format_tu(&tu);
        assert_eq!(out, "int *ptr;");
    }
