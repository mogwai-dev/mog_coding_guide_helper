use std::fs;
use std::env;
use coding_guide_helper::{Lexer, Parser, Item};
use coding_guide_helper::token::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = if args.len() > 1 {
        &args[1]
    } else {
        "example.txt"
    };
    
    lexer_sample(filename);
    parser_sample(filename);
}

// lexer_sample() 関数を修正
fn lexer_sample(filename: &str) {
    println!("[Lexer Sample]");
    let contents = fs::read_to_string(filename).unwrap();
    let mut lx = Lexer::new(&contents);
    
    while let Some(token) = lx.next_token() {
        match token {
            Token::BlockComment(BlockCommentToken { span }) => {
                println!("Block comment from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
            Token::Include(IncludeToken { span, filename }) => {
                println!("Include from ({}, {}) to ({}, {}): {:?} (filename: {})", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx], filename);
            },
            Token::Define(DefineToken { span, macro_name, macro_value }) => {
                println!("Define from ({}, {}) to ({}, {}): {:?} (macro: {}, value: {})", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx], macro_name, macro_value);
            },
            Token::Typedef(TypedefToken { span }) => {
                println!("Typedef from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
            Token::Semicolon(SemicolonToken { span }) => {
                println!("Semicolon from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
            Token::Equal(EqualToken { span }) => {
                println!("Equal from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
            Token::Ident(IdentToken { span, name }) => {
                println!("Ident from ({}, {}) to ({}, {}): {:?} (name: {})", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx], name);
            },
            // 記憶域クラス指定子
            Token::Auto(AutoToken { span }) | Token::Register(RegisterToken { span }) | Token::Static(StaticToken { span }) | 
            Token::Extern(ExternToken { span }) => {
                println!("Storage class from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
            // 型修飾子
            Token::Const(ConstToken { span }) | Token::Volatile(VolatileToken { span }) | Token::Restrict(RestrictToken { span }) | 
            Token::Atomic(AtomicToken { span }) => {
                println!("Type qualifier from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
            // 型指定子
            Token::Int(IntToken { span }) | Token::Char(CharToken { span }) | Token::Float(FloatToken { span }) | 
            Token::Double(DoubleToken { span }) | Token::Void(VoidToken { span }) | Token::Long(LongToken { span }) | 
            Token::Short(ShortToken { span }) | Token::Signed(SignedToken { span }) | Token::Unsigned(UnsignedToken { span }) => {
                println!("Type specifier from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
            Token::Struct(StructToken { span }) => {
                println!("Struct from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
            Token::Enum(EnumToken { span }) => {
                println!("Enum from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
            Token::Union(UnionToken { span }) => {
                println!("Union from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
            Token::LeftBrace(LeftBraceToken { span }) => {
                println!("LeftBrace from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
            Token::RightBrace(RightBraceToken { span }) => {
                println!("RightBrace from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
            Token::LeftParen(LeftParenToken { span }) => {
                println!("LeftParen from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
            Token::RightParen(RightParenToken { span }) => {
                println!("RightParen from ({}, {}) to ({}, {}): {:?}", span.start_line, span.start_column, span.end_line, span.end_column, &contents[span.byte_start_idx..span.byte_end_idx]);
            },
        }
    }
}

fn parser_sample(filename: &str) {
    println!("\n[Parser Sample]");
    let contents = fs::read_to_string(filename).unwrap();
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
            Item::FunctionDecl { span, return_type, function_name, parameters, .. } => {
                println!("FunctionDecl from ({}, {}) to ({}, {}): {} {} {}",
                    span.start_line, span.start_column, span.end_line, span.end_column, 
                    return_type, function_name, parameters);
            },
            Item::EnumDecl { span, text, enum_name, variable_names, .. } => {
                println!("EnumDecl from ({}, {}) to ({}, {}): {:?} (enum_name: {:?}, variables: {:?})", 
                    span.start_line, span.start_column, span.end_line, span.end_column, text, enum_name, variable_names);
            },
            Item::UnionDecl { span, text, union_name, variable_names, .. } => {
                println!("UnionDecl from ({}, {}) to ({}, {}): {:?} (union_name: {:?}, variables: {:?})", 
                    span.start_line, span.start_column, span.end_line, span.end_column, text, union_name, variable_names);
            },
        }
    }
}
