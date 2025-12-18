use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::config::PreprocessorConfig;
use std::path::PathBuf;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_ifdef_with_config_defines() {
    let input = r#"
#ifdef _WIN32
typedef int HANDLE;
#else
typedef void* HANDLE;
#endif
"#;
    
    // _WIN32が定義されている場合
    let mut config = PreprocessorConfig::default();
    config.defines.push("_WIN32".to_string());
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new_with_config(lexer, config);
    let _ = parser.parse();
    
    // HANDLE型が登録されているか確認
    assert!(parser.get_type_table().is_type_name("HANDLE"));
}

#[test]
fn test_ifdef_without_config_defines() {
    let input = r#"
#ifdef _WIN32
typedef int HANDLE;
#else
typedef void* HANDLE;
#endif
"#;
    
    // マクロが定義されていない場合（elseブランチが有効）
    let config = PreprocessorConfig::default();
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new_with_config(lexer, config);
    let _ = parser.parse();
    
    // HANDLE型が登録されているか確認（elseブランチから）
    assert!(parser.get_type_table().is_type_name("HANDLE"));
}

#[test]
fn test_simple_ifdef_typedef() {
    let input = r#"#ifdef DEBUG
typedef int DebugInt;
#endif"#;
    
    let mut config = PreprocessorConfig::default();
    config.defines.push("DEBUG".to_string());
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new_with_config(lexer, config);
    let tu = parser.parse();
    
    eprintln!("Items: {}", tu.items.len());
    eprintln!("Types: {:?}", parser.get_type_table().get_all_types());
    
    assert!(parser.get_type_table().is_type_name("DebugInt"));
}

#[test]
fn test_nested_ifdef() {
    let input = r#"#ifdef _WIN32
#ifdef DEBUG
typedef int DebugHandle;
#endif
#endif"#;
    
    // 両方定義されている場合
    let mut config = PreprocessorConfig::default();
    config.defines.push("_WIN32".to_string());
    config.defines.push("DEBUG".to_string());
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new_with_config(lexer, config);
    let tu = parser.parse();
    
    // デバッグ出力
    eprintln!("Registered types:");
    for name in parser.get_type_table().get_all_types().iter() {
        eprintln!("  - {}", name);
    }
    eprintln!("Total items: {}", tu.items.len());
    
    assert!(parser.get_type_table().is_type_name("DebugHandle"));
}

#[test]
fn test_nested_ifdef_partial() {
    let input = r#"
#ifdef _WIN32
  #ifdef DEBUG
    typedef int DebugHandle;
  #endif
#endif
"#;
    
    // _WIN32のみ定義（DEBUGは未定義）
    let mut config = PreprocessorConfig::default();
    config.defines.push("_WIN32".to_string());
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new_with_config(lexer, config);
    let _ = parser.parse();
    
    // DebugHandleは登録されていないはず
    assert!(!parser.get_type_table().is_type_name("DebugHandle"));
}

#[test]
fn test_typedef_in_active_branch_only() {
    let input = r#"
#ifdef LINUX
typedef int LinuxInt;
#endif
#ifdef WINDOWS
typedef int WindowsInt;
#endif
"#;
    
    // WINDOWSのみ定義
    let mut config = PreprocessorConfig::default();
    config.defines.push("WINDOWS".to_string());
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new_with_config(lexer, config);
    let _ = parser.parse();
    
    assert!(!parser.get_type_table().is_type_name("LinuxInt"));
    assert!(parser.get_type_table().is_type_name("WindowsInt"));
}

#[test]
fn include_paths_resolved_against_project_root() {
    let temp = tempdir().unwrap();
    let include_dir = temp.path().join("mylib");
    fs::create_dir_all(&include_dir).unwrap();
    fs::write(include_dir.join("types.h"), "typedef int FROM_HEADER;\n").unwrap();

    let mut config = PreprocessorConfig::default();
    config.include_paths = vec![PathBuf::from("mylib")];
    let resolved = config.resolved_with_root(temp.path());

    let source = "#include <types.h>\n";
    let lexer = Lexer::new(source);
    let mut parser = Parser::new_with_config(lexer, resolved);
    parser.set_current_file_dir(temp.path());
    let _ = parser.parse();

    assert!(parser.get_type_table().is_type_name("FROM_HEADER"));
}
