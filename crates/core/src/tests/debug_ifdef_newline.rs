use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::config::PreprocessorConfig;

#[test]
fn debug_newline_issue() {
    // 改行ありバージョン
    let input_with_newlines = r#"
#ifdef _WIN32
  #ifdef DEBUG
    typedef int DebugHandle;
  #endif
#endif
"#;

    // 改行なしバージョン
    let input_without_newlines = r#"#ifdef _WIN32
#ifdef DEBUG
typedef int DebugHandle;
#endif
#endif"#;

    let mut config = PreprocessorConfig::default();
    config.defines.push("_WIN32".to_string());
    config.defines.push("DEBUG".to_string());

    // 改行ありの場合
    eprintln!("\n=== With leading/trailing newlines ===");
    let lexer1 = Lexer::new(input_with_newlines);
    let mut parser1 = Parser::new_with_config(lexer1, config.clone());
    let tu1 = parser1.parse();
    
    eprintln!("Input length: {}", input_with_newlines.len());
    eprintln!("Total items: {}", tu1.items.len());
    for (i, item) in tu1.items.iter().enumerate() {
        eprintln!("Item {}: {:?}", i, item);
    }
    eprintln!("Registered types: {:?}", parser1.get_type_table().get_all_types());
    eprintln!("Has DebugHandle: {}", parser1.get_type_table().is_type_name("DebugHandle"));

    // 改行なしの場合
    eprintln!("\n=== Without leading/trailing newlines ===");
    let lexer2 = Lexer::new(input_without_newlines);
    let mut parser2 = Parser::new_with_config(lexer2, config);
    let tu2 = parser2.parse();
    
    eprintln!("Input length: {}", input_without_newlines.len());
    eprintln!("Total items: {}", tu2.items.len());
    for (i, item) in tu2.items.iter().enumerate() {
        eprintln!("Item {}: {:?}", i, item);
    }
    eprintln!("Registered types: {:?}", parser2.get_type_table().get_all_types());
    eprintln!("Has DebugHandle: {}", parser2.get_type_table().is_type_name("DebugHandle"));
}

#[test]
fn debug_simple_newline() {
    let input_with_newline = "\n#ifdef DEBUG\ntypedef int DebugInt;\n#endif\n";
    let input_no_newline = "#ifdef DEBUG\ntypedef int DebugInt;\n#endif";

    let mut config = PreprocessorConfig::default();
    config.defines.push("DEBUG".to_string());

    // 改行ありの場合
    eprintln!("\n=== Simple case with newlines ===");
    let lexer1 = Lexer::new(input_with_newline);
    let mut parser1 = Parser::new_with_config(lexer1, config.clone());
    let tu1 = parser1.parse();
    eprintln!("Total items: {}", tu1.items.len());
    eprintln!("Has DebugInt: {}", parser1.get_type_table().is_type_name("DebugInt"));

    // 改行なしの場合
    eprintln!("\n=== Simple case without leading newline ===");
    let lexer2 = Lexer::new(input_no_newline);
    let mut parser2 = Parser::new_with_config(lexer2, config);
    let tu2 = parser2.parse();
    eprintln!("Total items: {}", tu2.items.len());
    eprintln!("Has DebugInt: {}", parser2.get_type_table().is_type_name("DebugInt"));
}
