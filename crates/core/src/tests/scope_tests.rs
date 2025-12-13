//! スコープ管理機能のテスト

use crate::lexer::Lexer;
use crate::parser::Parser;

#[test]
fn test_global_typedef() {
    // グローバルスコープのtypedef
    let input = r#"
        typedef int MyInt;
        MyInt x;
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let _ast = parser.parse();
    
    assert!(parser.get_type_table().is_type_name("MyInt"), "MyInt should be registered");
}

#[test]
fn test_local_typedef_in_function() {
    // 関数内のローカルtypedef（現在は未サポート）
    let input = r#"
        int func() {
            typedef int LocalInt;
            LocalInt x;
            return x;
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let _ast = parser.parse();
    
    // 現在の実装では関数内のブロックはスキップされるため、typedefは登録されない
    // 将来ブロック内解析を実装すれば、このテストを変更する
    assert!(!parser.get_type_table().is_type_name("LocalInt"), "LocalInt should NOT be registered (function body skipped)");
}

#[test]
fn test_typedef_shadowing() {
    // 内側のスコープで外側のtypedefをシャドーイング
    let input = r#"
        typedef int OuterType;
        int func() {
            typedef float OuterType;
            OuterType x;
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let _ast = parser.parse();
    
    // 最後に登録された型情報を取得
    let type_info = parser.get_type_table().get_type_info("OuterType");
    assert!(type_info.is_some(), "OuterType should be registered");
}

#[test]
fn test_nested_scopes() {
    // ネストしたスコープでのtypedef（現在は関数内未サポート）
    let input = r#"
        typedef int Level0;
        int func1() {
            typedef int Level1;
            {
                typedef int Level2;
            }
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let _ast = parser.parse();
    
    // 現在の実装では関数内はスキップされる
    assert!(parser.get_type_table().is_type_name("Level0"));
    assert!(!parser.get_type_table().is_type_name("Level1"), "Level1 should NOT be registered");
    assert!(!parser.get_type_table().is_type_name("Level2"), "Level2 should NOT be registered");
}

#[test]
fn test_multiple_global_typedefs() {
    // 複数のグローバルtypedef
    let input = r#"
        typedef int MyInt;
        typedef float MyFloat;
        typedef struct { int x; } MyStruct;
        
        MyInt a;
        MyFloat b;
        MyStruct c;
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let _ast = parser.parse();
    
    assert!(parser.get_type_table().is_type_name("MyInt"));
    assert!(parser.get_type_table().is_type_name("MyFloat"));
    assert!(parser.get_type_table().is_type_name("MyStruct"));
}

#[test]
fn test_typedef_across_functions() {
    // 異なる関数間でのtypedef共有
    let input = r#"
        typedef int SharedType;
        
        int func1() {
            SharedType x;
            return x;
        }
        
        int func2() {
            SharedType y;
            return y;
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let _ast = parser.parse();
    
    assert!(parser.get_type_table().is_type_name("SharedType"));
}

#[test]
fn test_scope_depth() {
    // スコープの深さを確認
    let mut type_table = crate::type_table::TypeTable::new();
    
    assert_eq!(type_table.scope_depth(), 0, "Initial depth should be 0 (global)");
    
    type_table.push_scope();
    assert_eq!(type_table.scope_depth(), 1, "Depth should be 1 after push");
    
    type_table.push_scope();
    assert_eq!(type_table.scope_depth(), 2, "Depth should be 2 after second push");
    
    type_table.pop_scope();
    assert_eq!(type_table.scope_depth(), 1, "Depth should be 1 after pop");
    
    type_table.pop_scope();
    assert_eq!(type_table.scope_depth(), 0, "Depth should be 0 after second pop");
}

#[test]
fn test_scope_isolation() {
    // スコープ間の分離を確認
    use crate::type_table::TypeTable;
    use crate::type_system::{BaseType, Type};
    use crate::span::Span;
    
    let dummy_span = Span {
        start_line: 0,
        start_column: 0,
        end_line: 0,
        end_column: 0,
        byte_start_idx: 0,
        byte_end_idx: 0,
    };
    
    let mut type_table = TypeTable::new();
    
    // グローバルスコープに型を登録
    type_table.register_type(
        "GlobalType".to_string(),
        Type::new(BaseType::Int, dummy_span.clone())
    );
    
    assert!(type_table.is_type_name("GlobalType"));
    
    // 新しいスコープを開始
    type_table.push_scope();
    
    // ローカルスコープに型を登録
    type_table.register_type(
        "LocalType".to_string(),
        Type::new(BaseType::Float, dummy_span.clone())
    );
    
    // 両方見える
    assert!(type_table.is_type_name("GlobalType"), "Global type should be visible in local scope");
    assert!(type_table.is_type_name("LocalType"), "Local type should be visible in local scope");
    
    // スコープを抜ける
    type_table.pop_scope();
    
    // グローバルのみ見える
    assert!(type_table.is_type_name("GlobalType"), "Global type should still be visible");
    assert!(!type_table.is_type_name("LocalType"), "Local type should not be visible after pop");
}

#[test]
fn test_shadowing_in_scopes() {
    // スコープでのシャドーイングを確認
    use crate::type_table::TypeTable;
    use crate::type_system::{BaseType, Type};
    use crate::span::Span;
    
    let dummy_span = Span {
        start_line: 0,
        start_column: 0,
        end_line: 0,
        end_column: 0,
        byte_start_idx: 0,
        byte_end_idx: 0,
    };
    
    let mut type_table = TypeTable::new();
    
    // グローバルスコープにInt型を登録
    type_table.register_type(
        "MyType".to_string(),
        Type::new(BaseType::Int, dummy_span.clone())
    );
    
    let global_type = type_table.get_type_info("MyType").unwrap();
    assert!(matches!(global_type.base_type, BaseType::Int));
    
    // 新しいスコープでFloat型として再定義
    type_table.push_scope();
    type_table.register_type(
        "MyType".to_string(),
        Type::new(BaseType::Float, dummy_span.clone())
    );
    
    let local_type = type_table.get_type_info("MyType").unwrap();
    assert!(matches!(local_type.base_type, BaseType::Float), "Should see Float in local scope");
    
    // スコープを抜けると元のInt型が見える
    type_table.pop_scope();
    
    let global_type_again = type_table.get_type_info("MyType").unwrap();
    assert!(matches!(global_type_again.base_type, BaseType::Int), "Should see Int after pop");
}

#[test]
#[should_panic(expected = "Cannot pop global scope")]
fn test_cannot_pop_global_scope() {
    // グローバルスコープはpopできない
    let mut type_table = crate::type_table::TypeTable::new();
    type_table.pop_scope(); // パニックする
}
