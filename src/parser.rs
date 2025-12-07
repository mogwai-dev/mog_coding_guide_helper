use crate::lexer::Lexer;
use crate::token::Token;
use crate::ast::{TranslationUnit, Item};
use crate::span::Span;

#[derive(Debug)]
pub struct Parser<'a> {
    pub lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Parser { lexer }
    }

    pub fn parse(&mut self) -> TranslationUnit {
        let mut items = Vec::new();

        while let Some(token) = self.lexer.next_token() {
            match token {
                Token::BlockComment { span } => {
                    let text = self.lexer.input[span.byte_start_idx..span.byte_end_idx].to_string();
                    items.push(Item::BlockComment { span, text });
                },
                Token::Include { span, filename } => {
                    let text = self.lexer.input[span.byte_start_idx..span.byte_end_idx].to_string();
                    items.push(Item::Include { span, text, filename });
                },
                Token::Define { span, macro_name, macro_value } => {
                    let text = self.lexer.input[span.byte_start_idx..span.byte_end_idx].to_string();
                    items.push(Item::Define { span, text, macro_name, macro_value });
                },
                // ★ 古い Token::Typedef のケースを削除（534-556行目）
                // 記憶域クラス指定子、型修飾子、型指定子で始まる変数宣言
                Token::Auto { span } | Token::Register { span } | Token::Static { span } | 
                Token::Extern { span } | Token::Const { span } | Token::Volatile { span } | 
                Token::Restrict { span } | Token::_Atomic { span } |
                Token::Int { span } | Token::Char { span } | Token::Float { span } | 
                Token::Double { span } | Token::Void { span } | Token::Long { span } | 
                Token::Short { span } | Token::Signed { span } | Token::Unsigned { span } => {
                    let start_byte = span.byte_start_idx;
                    let mut end_byte = span.byte_end_idx;
                    let mut var_name = String::new();
                    let mut has_initializer = false;
                    
                    loop {
                        match self.lexer.next_token() {
                            Some(Token::Ident { span: id_span, name }) => {
                                var_name = name.to_string();
                                end_byte = id_span.byte_end_idx;
                            },
                            Some(Token::Equal { span: eq_span }) => {
                                has_initializer = true;
                                end_byte = eq_span.byte_end_idx;
                            },
                            Some(Token::Semicolon { span: semi_span }) => {
                                end_byte = semi_span.byte_end_idx;
                            //    if brace_depth == 0 {
                                    break;
                            //    }
                            },
                            // 記憶域クラス指定子、型修飾子、型指定子は読み飛ばす
                            Some(Token::Auto { .. }) | Some(Token::Register { .. }) | 
                            Some(Token::Static { .. }) | Some(Token::Extern { .. }) |
                            Some(Token::Const { .. }) | Some(Token::Volatile { .. }) | 
                            Some(Token::Restrict { .. }) | Some(Token::_Atomic { .. }) |
                            Some(Token::Int { .. }) | Some(Token::Char { .. }) | 
                            Some(Token::Float { .. }) | Some(Token::Double { .. }) | 
                            Some(Token::Void { .. }) | Some(Token::Long { .. }) | 
                            Some(Token::Short { .. }) | Some(Token::Signed { .. }) | 
                            Some(Token::Unsigned { .. }) => {
                                continue;
                            },
                            Some(_) => {
                                continue;
                            },
                            None => {
                                break;
                            }
                        }
                    }
                    
                    let text = self.lexer.input[start_byte..end_byte].to_string();
                    let final_span = Span {
                        start_line: span.start_line,
                        start_column: span.start_column,
                        end_line: self.lexer.line,
                        end_column: self.lexer.column,
                        byte_start_idx: start_byte,
                        byte_end_idx: end_byte,
                    };
                    items.push(Item::VarDecl { 
                        span: final_span, 
                        text,
                        var_name,
                        has_initializer,
                    });
                },
                Token::Struct { span } => {
                    // struct 宣言または構造体変数宣言
                    
                    let start_byte = span.byte_start_idx;
                    let mut end_byte = span.byte_end_idx;
                    let mut struct_name: Option<String> = None;
                    let mut has_typedef = false;
                    let mut found_brace = false;
                    let mut brace_depth = 0;
                    
                    loop {
                        match self.lexer.next_token() {
                            Some(Token::Ident { name, .. }) => {
                                // 構造体名（または変数名）
                                if struct_name.is_none() && !found_brace {
                                    struct_name = Some(name.to_string());
                                }
                            },
                            Some(Token::LeftBrace { .. }) => {
                                brace_depth += 1;
                                found_brace = true;
                            },
                            Some(Token::RightBrace { .. }) => {
                                brace_depth -= 1;
                            },
                            Some(Token::Semicolon { span: semi_span }) => {
                                end_byte = semi_span.byte_end_idx;
                                if brace_depth == 0 {
                                    break;
                                }
                            },
                            Some(Token::Struct { .. }) => {
                                // 内部のstructキーワードはスキップ
                                continue;
                            },
                            Some(_) => {
                                continue;
                            },
                            None => {
                                break;
                            }
                        }
                    }
                    
                    let text = self.lexer.input[start_byte..end_byte].to_string();
                    let final_span = Span {
                        start_line: span.start_line,
                        start_column: span.start_column,
                        end_line: self.lexer.line,
                        end_column: self.lexer.column,
                        byte_start_idx: start_byte,
                        byte_end_idx: end_byte,
                    };
                    items.push(Item::StructDecl { 
                        span: final_span, 
                        text,
                        struct_name,
                        has_typedef,
                    });
                },
                Token::Typedef { span } => {
                    let start_byte = span.byte_start_idx;
                    let mut end_byte = span.byte_end_idx;
                    
                    // 次のトークンが struct かチェック
                    match self.lexer.next_token() {
                        Some(Token::Struct { .. }) => {
                            // typedef struct の処理
                            let mut struct_name: Option<String> = None;
                            let mut brace_depth = 0;
                            let mut found_brace = false;
                            
                            loop {
                                match self.lexer.next_token() {
                                    Some(Token::Ident { name, .. }) => {
                                        if struct_name.is_none() && !found_brace {
                                            struct_name = Some(name.to_string());
                                        }
                                    },
                                    Some(Token::LeftBrace { .. }) => {
                                        brace_depth += 1;
                                        found_brace = true;
                                    },
                                    Some(Token::RightBrace { .. }) => {
                                        brace_depth -= 1;
                                    },
                                    Some(Token::Semicolon { span: semi_span }) => {
                                        end_byte = semi_span.byte_end_idx;
                                        if brace_depth == 0 {
                                            break;
                                    }
                                    },
                                    Some(_) => {
                                        continue;
                                    },
                                    None => {
                                        break;
                                    }
                                }
                            }
                            
                            let text = self.lexer.input[start_byte..end_byte].to_string();
                            let final_span = Span {
                                start_line: span.start_line,
                                start_column: span.start_column,
                                end_line: self.lexer.line,
                                end_column: self.lexer.column,
                                byte_start_idx: start_byte,
                                byte_end_idx: end_byte,
                            };
                            items.push(Item::StructDecl { 
                                span: final_span, 
                                text,
                                struct_name,
                                has_typedef: true,
                            });
                        },
                        _ => {
                            // 通常の typedef（既存の処理）
                            loop {
                                match self.lexer.next_token() {
                                    Some(Token::Semicolon { span: semi_span }) => {
                                        end_byte = semi_span.byte_end_idx;
                                        break;
                                    },
                                    Some(_) => continue,
                                    None => break,
                                }
                            }
                
                            let text = self.lexer.input[start_byte..end_byte].to_string();
                            let final_span = Span {
                                start_line: span.start_line,
                                start_column: span.start_column,
                                end_line: self.lexer.line,
                                end_column: self.lexer.column,
                                byte_start_idx: start_byte,
                                byte_end_idx: end_byte,
                            };
                            items.push(Item::TypedefDecl { span: final_span, text });
                        }
                    }
                },
                _ => {
                    continue;
                }
            }
        }

        TranslationUnit { items }
    }
}
