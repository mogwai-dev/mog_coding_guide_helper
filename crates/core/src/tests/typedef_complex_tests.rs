use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::expression_parser::ExpressionParser;
use crate::expression::Expression;

/// Phase 2: 複雑なtypedef（関数ポインタ、配列）のテスト

#[test]
fn test_function_pointer_typedef() {
    // 関数ポインタのtypedef
    let typedef_code = "typedef int (*FuncPtr)(int, char*);";
    let typedef_lexer = Lexer::new(typedef_code);
    let mut parser = Parser::new(typedef_lexer);
    let _ast = parser.parse();
    
    // 型テーブルにFuncPtrが登録されているか確認
    assert!(parser.get_type_table().is_type_name("FuncPtr"), "FuncPtr should be registered");
    
    // FuncPtrを使ったキャスト
    let input = "(FuncPtr) func";
    let mut lexer = Lexer::new(input);
    let mut expr_parser = ExpressionParser::new(&mut lexer).with_type_table(parser.get_type_table());
    let expr = expr_parser.parse_expression().unwrap();
    
    assert!(matches!(expr, Expression::Cast { .. }), "Should be recognized as cast");
}

#[test]
fn test_array_typedef() {
    // 配列のtypedef
    let typedef_code = "typedef int IntArray[10];";
    let typedef_lexer = Lexer::new(typedef_code);
    let mut parser = Parser::new(typedef_lexer);
    let _ast = parser.parse();
    
    // 型テーブルにIntArrayが登録されているか確認
    assert!(parser.get_type_table().is_type_name("IntArray"), "IntArray should be registered");
    
    // IntArrayを使ったキャスト
    let input = "(IntArray) arr";
    let mut lexer = Lexer::new(input);
    let mut expr_parser = ExpressionParser::new(&mut lexer).with_type_table(parser.get_type_table());
    let expr = expr_parser.parse_expression().unwrap();
    
    assert!(matches!(expr, Expression::Cast { .. }), "Should be recognized as cast");
}

#[test]
fn test_multidimensional_array_typedef() {
    // 多次元配列のtypedef
    let typedef_code = "typedef int Matrix[3][4];";
    let typedef_lexer = Lexer::new(typedef_code);
    let mut parser = Parser::new(typedef_lexer);
    let _ast = parser.parse();
    
    // 型テーブルにMatrixが登録されているか確認
    assert!(parser.get_type_table().is_type_name("Matrix"), "Matrix should be registered");
}

#[test]
fn test_function_pointer_returning_pointer() {
    // ポインタを返す関数ポインタ
    let typedef_code = "typedef char* (*StringFunc)(int);";
    let typedef_lexer = Lexer::new(typedef_code);
    let mut parser = Parser::new(typedef_lexer);
    let _ast = parser.parse();
    
    assert!(parser.get_type_table().is_type_name("StringFunc"), "StringFunc should be registered");
}

#[test]
fn test_array_of_pointers_typedef() {
    // ポインタの配列
    let typedef_code = "typedef int* PtrArray[5];";
    let typedef_lexer = Lexer::new(typedef_code);
    let mut parser = Parser::new(typedef_lexer);
    let _ast = parser.parse();
    
    assert!(parser.get_type_table().is_type_name("PtrArray"), "PtrArray should be registered");
}

#[test]
fn test_complex_function_pointer() {
    // 複雑な関数ポインタ：関数ポインタを引数に取る関数ポインタ
    let typedef_code = "typedef int (*ComplexFunc)(int (*)(char*), void*);";
    let typedef_lexer = Lexer::new(typedef_code);
    let mut parser = Parser::new(typedef_lexer);
    let _ast = parser.parse();
    
    assert!(parser.get_type_table().is_type_name("ComplexFunc"), "ComplexFunc should be registered");
}

#[test]
fn test_multiple_complex_typedefs() {
    // 複数の複雑なtypedefを一度に処理
    let typedef_code = r#"
        typedef int (*FuncPtr)(int);
        typedef int Array[10];
        typedef char* (*StrFunc)(void);
    "#;
    let typedef_lexer = Lexer::new(typedef_code);
    let mut parser = Parser::new(typedef_lexer);
    let _ast = parser.parse();
    
    assert!(parser.get_type_table().is_type_name("FuncPtr"));
    assert!(parser.get_type_table().is_type_name("Array"));
    assert!(parser.get_type_table().is_type_name("StrFunc"));
}
