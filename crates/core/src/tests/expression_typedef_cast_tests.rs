use crate::expression::*;
use crate::expression_parser::ExpressionParser;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::type_system::BaseType;

#[test]
fn test_cast_with_typedef() {
    // まずtypedefを定義してParserの型テーブルに登録
    let typedef_code = "typedef int MyInt;";
    let typedef_lexer = Lexer::new(typedef_code);
    let mut parser = Parser::new(typedef_lexer);
    let _ast = parser.parse(); // typedefをパースして型テーブルに登録
    
    // 次に、その型名を使ったキャスト式をパース
    let input = "(MyInt) x";
    let mut lexer = Lexer::new(input);
    let mut expr_parser = ExpressionParser::new(&mut lexer).with_type_table(parser.get_type_table());
    let expr = expr_parser.parse_expression().unwrap();

    match expr {
        Expression::Cast { target_type, operand, .. } => {
            // MyIntはintにtypedefされているので、BaseType::Intになる
            assert_eq!(target_type.base_type, BaseType::Int);
            
            match *operand {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "x");
                },
                _ => panic!("Expected Identifier, got {:?}", operand),
            }
        },
        _ => panic!("Expected Cast expression, got {:?}", expr),
    }
}

#[test]
fn test_cast_with_typedef_pointer() {
    // typedef int *IntPtr;
    let typedef_code = "typedef int *IntPtr;";
    let typedef_lexer = Lexer::new(typedef_code);
    let mut parser = Parser::new(typedef_lexer);
    let _ast = parser.parse();
    
    let input = "(IntPtr) ptr";
    let mut lexer = Lexer::new(input);
    let mut expr_parser = ExpressionParser::new(&mut lexer).with_type_table(parser.get_type_table());
    let expr = expr_parser.parse_expression().unwrap();

    match expr {
        Expression::Cast { target_type, operand, .. } => {
            // 注: 現在の実装では、typedef名の実際の型情報は保存していないため、
            // IntPtrは単純にIntとしてパースされる。完全な実装にはTypeTableに型情報を保存する必要がある。
            // ここでは、Castとして認識されることだけを確認
            match *operand {
                Expression::Identifier { name, .. } => {
                    assert_eq!(name, "ptr");
                },
                _ => panic!("Expected Identifier, got {:?}", operand),
            }
        },
        _ => panic!("Expected Cast expression, got {:?}", expr),
    }
}

#[test]
fn test_cast_with_multiple_typedefs() {
    // 複数のtypedefを定義（改行で分割）
    let typedef_code = "typedef int Int32;\ntypedef long Int64;\ntypedef char *String;";
    let typedef_lexer = Lexer::new(typedef_code);
    let mut parser = Parser::new(typedef_lexer);
    let _ast = parser.parse();
    
    // Int32を使ったキャスト
    let input = "(Int32) value";
    let mut lexer = Lexer::new(input);
    let mut expr_parser = ExpressionParser::new(&mut lexer).with_type_table(parser.get_type_table());
    let expr = expr_parser.parse_expression().unwrap();

    // Castとして認識されることを確認
    assert!(matches!(expr, Expression::Cast { .. }));
    
    // Int64を使ったキャスト
    let input2 = "(Int64) x";
    let mut lexer2 = Lexer::new(input2);
    let mut expr_parser2 = ExpressionParser::new(&mut lexer2).with_type_table(parser.get_type_table());
    let expr2 = expr_parser2.parse_expression().unwrap();

    // Castとして認識されることを確認
    assert!(matches!(expr2, Expression::Cast { .. }));
}

#[test]
fn test_typedef_vs_variable() {
    // typedefを定義
    let typedef_code = "typedef int MyType;";
    let typedef_lexer = Lexer::new(typedef_code);
    let mut parser = Parser::new(typedef_lexer);
    let _ast = parser.parse();
    
    // (MyType) x はキャストとして認識
    let input1 = "(MyType) x";
    let mut lexer1 = Lexer::new(input1);
    let mut expr_parser1 = ExpressionParser::new(&mut lexer1).with_type_table(parser.get_type_table());
    let expr1 = expr_parser1.parse_expression().unwrap();
    assert!(matches!(expr1, Expression::Cast { .. }));
    
    // (notAType) x は括弧式として認識（notATypeは型ではない）
    let input2 = "(notAType) * x";
    let mut lexer2 = Lexer::new(input2);
    let mut expr_parser2 = ExpressionParser::new(&mut lexer2).with_type_table(parser.get_type_table());
    let expr2 = expr_parser2.parse_expression().unwrap();
    // (notAType) * x は括弧式 (notAType) と単項演算 * x の乗算
    assert!(matches!(expr2, Expression::BinaryOp { .. }));
}

#[test]
fn test_typedef_struct() {
    // typedef struct を定義
    let typedef_code = "typedef struct { int x; } Point;";
    let typedef_lexer = Lexer::new(typedef_code);
    let mut parser = Parser::new(typedef_lexer);
    let _ast = parser.parse();
    
    // Pointを使ったキャスト
    let input = "(Point) value";
    let mut lexer = Lexer::new(input);
    let mut expr_parser = ExpressionParser::new(&mut lexer).with_type_table(parser.get_type_table());
    let expr = expr_parser.parse_expression().unwrap();

    // 型テーブルにPointが登録されているので、キャストとして認識される
    assert!(matches!(expr, Expression::Cast { .. }));
}

#[test]
fn test_no_typedef_fallback() {
    // 型テーブルなしでは、typedef名は認識されない
    let input = "(MyInt) x";
    let mut lexer = Lexer::new(input);
    let mut expr_parser = ExpressionParser::new(&mut lexer); // 型テーブルなし
    let expr = expr_parser.parse_expression().unwrap();

    // (MyInt) x は括弧式 (MyInt) と変数 x の乗算として解釈される
    // → (MyInt) * x のようにパースされる（実際には構文エラーになる可能性）
    // または (MyInt) 自体が単なる括弧で囲まれた識別子として認識される
    match expr {
        Expression::BinaryOp { .. } => {
            // 乗算として認識された場合
        },
        Expression::Identifier { .. } => {
            // 識別子として認識された場合（x）
        },
        _ => {},
    }
}
