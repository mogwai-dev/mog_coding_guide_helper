use crate::parser::Parser;
use crate::lexer::Lexer;

#[test]
fn test_typedef_registration() {
    let typedef_code = "typedef int MyInt;";
    let typedef_lexer = Lexer::new(typedef_code);
    let mut parser = Parser::new(typedef_lexer);
    let _ast = parser.parse();
    
    // 型テーブルにMyIntが登録されているか確認
    assert!(parser.get_type_table().is_type_name("MyInt"));
    assert!(!parser.get_type_table().is_type_name("SomeOtherType"));
}

#[test]
fn test_multiple_typedef_registration() {
    let typedef_code = "typedef int Int32;\ntypedef long Int64;";
    let typedef_lexer = Lexer::new(typedef_code);
    let mut parser = Parser::new(typedef_lexer);
    let _ast = parser.parse();
    
    println!("Registered types: {:?}", parser.get_type_table().get_all_types());
    
    // 両方の型名が登録されているか確認
    assert!(parser.get_type_table().is_type_name("Int32"), "Int32 should be registered");
    assert!(parser.get_type_table().is_type_name("Int64"), "Int64 should be registered");
}
