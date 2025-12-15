use crate::lexer::Lexer;
use crate::parser::Parser;

#[test]
fn debug_typedef_then_var() {
    let source = "typedef unsigned char VU8;\nVU8 counter;";
    
    println!("\n=== Lexer Tokens ===");
    let mut lexer = Lexer::new(source);
    let mut count = 0;
    while let Some(tok) = lexer.next_token() {
        println!("{}: {:?}", count, tok);
        count += 1;
        if count > 20 {
            println!("... (truncated)");
            break;
        }
    }
    println!("Total tokens: {}", count);
    
    println!("\n=== Parsed Items ===");
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    println!("Total: {}", tu.items.len());
    
    for (i, item) in tu.items.iter().enumerate() {
        match item {
            crate::ast::Item::TypedefDecl { text, .. } => {
                println!("{}. TypedefDecl: {}", i, text);
            }
            crate::ast::Item::VarDecl { var_name, var_type, text, .. } => {
                println!("{}. VarDecl: {}", i, var_name);
                println!("   text: '{}'", text);
                println!("   var_type: {:?}", var_type);
            }
            _ => {
                println!("{}. {:?}", i, item);
            }
        }
    }
}

#[test]
fn debug_typedef_then_var_separate() {
    // 別々にパースしてみる
    let typedef_source = "typedef unsigned char VU8;";
    let var_source = "VU8 counter;";
    
    println!("\n=== Typedef Only ===");
    let lexer1 = Lexer::new(typedef_source);
    let mut parser1 = Parser::new(lexer1);
    let tu1 = parser1.parse();
    println!("Items: {}", tu1.items.len());
    
    println!("\n=== Variable Only (with pre-registered type) ===");
    let lexer2 = Lexer::new(var_source);
    let mut parser2 = Parser::new(lexer2);
    let tu2 = parser2.parse();
    println!("Items: {}", tu2.items.len());
    for item in &tu2.items {
        match item {
            crate::ast::Item::VarDecl { var_name, text, .. } => {
                println!("VarDecl: {} | text: '{}'", var_name, text);
            }
            _ => {
                println!("Other: {:?}", item);
            }
        }
    }
}
