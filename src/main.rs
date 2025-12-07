use std::fs;
use coding_guide_helper::{Lexer, Parser, Token, Item};

fn main() {
    lexer_sample();
    parser_sample();
}

// lexer_sample() 関数を修正
fn lexer_sample() {
    println!("[Lexer Sample]");
    let contents = fs::read_to_string("example.txt").unwrap();
    let mut lx = Lexer::new(&contents);
    
    while let Some(token) = lx.next_token() {
        match token {
            Token::BlockComment { span } => {
                println!("Block comment from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
            Token::Include { span, filename } => {
                println!("Include from ({}, {}) to ({}, {}): {:?} (filename: {})", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx], filename);
            },
            Token::Define { span, macro_name, macro_value } => {
                println!("Define from ({}, {}) to ({}, {}): {:?} (macro: {}, value: {})", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx], macro_name, macro_value);
            },
            Token::Typedef { span } => {
                println!("Typedef from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
            Token::Semicolon { span } => {
                println!("Semicolon from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
            Token::Equal { span } => {
                println!("Equal from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
            Token::Ident { span, name } => {
                println!("Ident from ({}, {}) to ({}, {}): {:?} (name: {})", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx], name);
            },
            // 記憶域クラス指定子
            Token::Auto { span } | Token::Register { span } | Token::Static { span } | 
            Token::Extern { span } => {
                println!("Storage class from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
            // 型修飾子
            Token::Const { span } | Token::Volatile { span } | Token::Restrict { span } | 
            Token::_Atomic { span } => {
                println!("Type qualifier from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
            // 型指定子
            Token::Int { span } | Token::Char { span } | Token::Float { span } | 
            Token::Double { span } | Token::Void { span, .. } | Token::Long { span } | 
            Token::Short { span } | Token::Signed { span } | Token::Unsigned { span } => {
                println!("Type specifier from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
            Token::Struct { span } => {
                println!("Struct from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
            Token::LeftBrace { span } => {
                println!("LeftBrace from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
            Token::RightBrace { span } => {
                println!("RightBrace from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
        }
    }
}

fn parser_sample() {
    println!("\n[Parser Sample]");
    let contents = fs::read_to_string("example.txt").unwrap();
    let lx = Lexer::new(&contents);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();

    for item in tu.items {
        match item {
            Item::BlockComment { span, text  } => {
                println!("Block comment from ({}, {}) to ({}, {}): {:?} ", span.start_line, span.start_column, span.end_line, span.end_column, text);
            },
            Item::Include { span, text, filename } => {
                println!("Include from ({}, {}) to ({}, {}): {:?} (filename: {})", span.start_line, span.start_column, span.end_line, span.end_column, text, filename);
            },
            Item::Define { span, text, macro_name, macro_value } => {
                println!("Define from ({}, {}) to ({}, {}): {:?} (macro: {}, value: {})", span.start_line, span.start_column, span.end_line, span.end_column, text, macro_name, macro_value);
            },
            Item::TypedefDecl { span, text } => {
                println!("TypedefDecl from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, text);
            },
            Item::VarDecl { span, text, var_name, has_initializer } => {
                println!("VarDecl from ({}, {}) to ({}, {}): {:?} (var_name: {}, has_initializer: {})", span.start_line, span.start_column, span.end_line, span.end_column, text, var_name, has_initializer);
            },
            Item::StructDecl { span, text, struct_name, has_typedef } => {
                println!("StructDecl from ({}, {}) to ({}, {}): {:?} (struct_name: {:?}, has_typedef: {})", 
                    span.start_line, span.start_column, span.end_line, span.end_column, text, struct_name, has_typedef);
            },
        }
    }
}
