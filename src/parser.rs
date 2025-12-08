use crate::lexer::Lexer;
use crate::token::*;
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
                Token::BlockComment(BlockCommentToken { span }) => {
                    let text = self.lexer.input[span.byte_start_idx..span.byte_end_idx].to_string();
                    items.push(Item::BlockComment { span, text });
                },
                Token::Include(IncludeToken { span, filename }) => {
                    let text = self.lexer.input[span.byte_start_idx..span.byte_end_idx].to_string();
                    items.push(Item::Include { span, text, filename });
                },
                Token::Define(DefineToken { span, macro_name, macro_value }) => {
                    let text = self.lexer.input[span.byte_start_idx..span.byte_end_idx].to_string();
                    items.push(Item::Define { span, text, macro_name, macro_value });
                },
                // ★ 古い Token::Typedef のケースを削除（534-556行目）
                // 記憶域クラス指定子、型修飾子、型指定子で始まる変数宣言
                Token::Auto(AutoToken { span }) | Token::Register(RegisterToken { span }) | Token::Static(StaticToken { span }) | 
                Token::Extern(ExternToken { span }) | Token::Const(ConstToken { span }) | Token::Volatile(VolatileToken { span }) | 
                Token::Restrict(RestrictToken { span }) | Token::Atomic(AtomicToken { span }) |
                Token::Int(IntToken { span }) | Token::Char(CharToken { span }) | Token::Float(FloatToken { span }) | 
                Token::Double(DoubleToken { span }) | Token::Void(VoidToken { span }) | Token::Long(LongToken { span }) | 
                Token::Short(ShortToken { span }) | Token::Signed(SignedToken { span }) | Token::Unsigned(UnsignedToken { span }) => {
                    let start_byte = span.byte_start_idx;
                    let mut end_byte = span.byte_end_idx;
                    let mut var_name = String::new();
                    let mut has_initializer = false;
                    let mut is_function = false;
                    let mut function_name = String::new();
                    let mut function_name_start = 0;
                    let mut params_start_byte = 0;
                    let mut params_end_byte = 0;
                    
                    loop {
                        match self.lexer.next_token() {
                            Some(Token::Ident(IdentToken { span: id_span, name })) => {
                                var_name = name.to_string();
                                function_name = name.to_string();
                                function_name_start = id_span.byte_start_idx;
                                end_byte = id_span.byte_end_idx;
                            },
                            Some(Token::LeftParen(LeftParenToken { span: lparen_span })) => {
                                // 関数定義の可能性
                                is_function = true;
                                params_start_byte = lparen_span.byte_start_idx;
                                
                                // 括弧の中を読み飛ばす
                                let mut paren_depth = 1;
                                loop {
                                    match self.lexer.next_token() {
                                        Some(Token::LeftParen(..)) => paren_depth += 1,
                                        Some(Token::RightParen(RightParenToken { span: rparen_span })) => {
                                            paren_depth -= 1;
                                            params_end_byte = rparen_span.byte_end_idx;
                                            end_byte = rparen_span.byte_end_idx;
                                            if paren_depth == 0 {
                                                break;
                                            }
                                        },
                                        Some(_) => continue,
                                        None => break,
                                    }
                                }
                            },
                            Some(Token::LeftBrace(..)) if is_function => {
                                // 関数ブロックの開始 - 中身を読み飛ばす
                                let mut brace_depth = 1;
                                loop {
                                    match self.lexer.next_token() {
                                        Some(Token::LeftBrace(..)) => brace_depth += 1,
                                        Some(Token::RightBrace(RightBraceToken { span: rbrace_span })) => {
                                            brace_depth -= 1;
                                            end_byte = rbrace_span.byte_end_idx;
                                            if brace_depth == 0 {
                                                break;
                                            }
                                        },
                                        Some(_) => continue,
                                        None => break,
                                    }
                                }
                                break;
                            },
                            Some(Token::Equal(EqualToken { span: eq_span })) => {
                                has_initializer = true;
                                end_byte = eq_span.byte_end_idx;
                            },
                            Some(Token::Semicolon(SemicolonToken { span: semi_span })) => {
                                end_byte = semi_span.byte_end_idx;
                                break;
                            },
                            // 記憶域クラス指定子、型修飾子、型指定子は読み飛ばす
                            Some(Token::Auto(..)) | Some(Token::Register(..)) | 
                            Some(Token::Static(..)) | Some(Token::Extern(..)) |
                            Some(Token::Const(..)) | Some(Token::Volatile(..)) | 
                            Some(Token::Restrict(..)) | Some(Token::Atomic(..)) |
                            Some(Token::Int(..)) | Some(Token::Char(..)) | 
                            Some(Token::Float(..)) | Some(Token::Double(..)) | 
                            Some(Token::Void(..)) | Some(Token::Long(..)) | 
                            Some(Token::Short(..)) | Some(Token::Signed(..)) | 
                            Some(Token::Unsigned(..)) => {
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
                    
                    if is_function {
                        // 関数定義
                        let return_type = self.lexer.input[start_byte..function_name_start].trim().to_string();
                        let parameters = self.lexer.input[params_start_byte..params_end_byte].to_string();
                        
                        items.push(Item::FunctionDecl {
                            span: final_span,
                            text,
                            return_type,
                            function_name,
                            parameters,
                            storage_class: None, // TODO: 記憶域クラスの検出
                        });
                    } else {
                        // 変数宣言
                        items.push(Item::VarDecl { 
                            span: final_span, 
                            text,
                            var_name,
                            has_initializer,
                        });
                    }
                },
                Token::Struct(StructToken { span }) => {
                    // struct 宣言または構造体変数宣言
                    
                    let start_byte = span.byte_start_idx;
                    let mut end_byte = span.byte_end_idx;
                    let mut struct_name: Option<String> = None;
                    let has_typedef = false;
                    let mut found_brace = false;
                    let mut brace_depth = 0;
                    
                    loop {
                        match self.lexer.next_token() {
                            Some(Token::Ident(IdentToken { name, .. })) => {
                                // 構造体名（または変数名）
                                if struct_name.is_none() && !found_brace {
                                    struct_name = Some(name.to_string());
                                }
                            },
                            Some(Token::LeftBrace(..)) => {
                                brace_depth += 1;
                                found_brace = true;
                            },
                            Some(Token::RightBrace(..)) => {
                                brace_depth -= 1;
                            },
                            Some(Token::Semicolon(SemicolonToken { span: semi_span })) => {
                                end_byte = semi_span.byte_end_idx;
                                if brace_depth == 0 {
                                    break;
                                }
                            },
                            Some(Token::Struct(..)) => {
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
                Token::Enum(EnumToken { span }) => {
                    // enum 宣言または列挙型変数宣言
                    
                    let start_byte = span.byte_start_idx;
                    let mut end_byte = span.byte_end_idx;
                    let mut enum_name: Option<String> = None;
                    let has_typedef = false;
                    let mut found_brace = false;
                    let mut brace_depth = 0;
                    let mut variable_names = Vec::new();
                    let mut after_brace_idents = Vec::new();
                    
                    loop {
                        match self.lexer.next_token() {
                            Some(Token::Ident(IdentToken { name, .. })) => {
                                if enum_name.is_none() && !found_brace {
                                    // enum名
                                    enum_name = Some(name.to_string());
                                } else if brace_depth == 0 && found_brace {
                                    // } の後の識別子は変数名
                                    after_brace_idents.push(name.to_string());
                                }
                            },
                            Some(Token::LeftBrace(..)) => {
                                brace_depth += 1;
                                found_brace = true;
                            },
                            Some(Token::RightBrace(..)) => {
                                brace_depth -= 1;
                            },
                            Some(Token::Semicolon(SemicolonToken { span: semi_span })) => {
                                end_byte = semi_span.byte_end_idx;
                                if brace_depth == 0 {
                                    // セミコロンの前の識別子が変数名
                                    variable_names = after_brace_idents;
                                    break;
                                }
                            },
                            Some(Token::Enum(..)) => {
                                // 内部のenumキーワードはスキップ
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
                    items.push(Item::EnumDecl { 
                        span: final_span, 
                        text,
                        enum_name,
                        has_typedef,
                        variable_names,
                    });
                },
                Token::Union(UnionToken { span }) => {
                    let start_byte = span.byte_start_idx;
                    let mut end_byte = span.byte_end_idx;
                    let mut union_name: Option<String> = None;
                    let mut brace_depth = 0;
                    let mut found_brace = false;
                    let has_typedef = false;
                    let mut variable_names = Vec::new();
                    let mut after_brace_idents = Vec::new();

                    loop {
                        match self.lexer.next_token() {
                            Some(Token::Ident(IdentToken { name, .. })) => {
                                if brace_depth == 0 && !found_brace {
                                    union_name = Some(name.to_string());
                                } else if brace_depth == 0 && found_brace {
                                    after_brace_idents.push(name.to_string());
                                }
                            },
                            Some(Token::LeftBrace(..)) => {
                                brace_depth += 1;
                                found_brace = true;
                            },
                            Some(Token::RightBrace(..)) => {
                                brace_depth -= 1;
                            },
                            Some(Token::Semicolon(SemicolonToken { span: semi_span })) => {
                                end_byte = semi_span.byte_end_idx;
                                if brace_depth == 0 {
                                    variable_names = after_brace_idents;
                                    break;
                                }
                            },
                            Some(Token::Union(..)) => {
                                // 内部のunionキーワードはスキップ
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
                    items.push(Item::UnionDecl { 
                        span: final_span, 
                        text,
                        union_name,
                        has_typedef,
                        variable_names,
                    });
                },
                Token::Typedef(TypedefToken { span }) => {
                    let start_byte = span.byte_start_idx;
                    let mut end_byte = span.byte_end_idx;
                    
                    // 次のトークンが struct、enum、または union かチェック
                    match self.lexer.next_token() {
                        Some(Token::Struct(..)) => {
                            // typedef struct の処理
                            let mut struct_name: Option<String> = None;
                            let mut brace_depth = 0;
                            let mut found_brace = false;
                            
                            loop {
                                match self.lexer.next_token() {
                                    Some(Token::Ident(IdentToken { name, .. })) => {
                                        if struct_name.is_none() && !found_brace {
                                            struct_name = Some(name.to_string());
                                        }
                                    },
                                    Some(Token::LeftBrace(..)) => {
                                        brace_depth += 1;
                                        found_brace = true;
                                    },
                                    Some(Token::RightBrace(..)) => {
                                        brace_depth -= 1;
                                    },
                                    Some(Token::Semicolon(SemicolonToken { span: semi_span })) => {
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
                        Some(Token::Enum(..)) => {
                            // typedef enum の処理
                            let mut enum_name: Option<String> = None;
                            let mut brace_depth = 0;
                            let mut found_brace = false;
                            let mut variable_names = Vec::new();
                            let mut after_brace_idents = Vec::new();
                            
                            loop {
                                match self.lexer.next_token() {
                                    Some(Token::Ident(IdentToken { name, .. })) => {
                                        if enum_name.is_none() && !found_brace {
                                            enum_name = Some(name.to_string());
                                        } else if brace_depth == 0 && found_brace {
                                            after_brace_idents.push(name.to_string());
                                        }
                                    },
                                    Some(Token::LeftBrace(..)) => {
                                        brace_depth += 1;
                                        found_brace = true;
                                    },
                                    Some(Token::RightBrace(..)) => {
                                        brace_depth -= 1;
                                    },
                                    Some(Token::Semicolon(SemicolonToken { span: semi_span })) => {
                                        end_byte = semi_span.byte_end_idx;
                                        if brace_depth == 0 {
                                            variable_names = after_brace_idents;
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
                            items.push(Item::EnumDecl { 
                                span: final_span, 
                                text,
                                enum_name,
                                has_typedef: true,
                                variable_names,
                            });
                        },
                        Some(Token::Union(..)) => {
                            // typedef union の処理
                            let mut union_name: Option<String> = None;
                            let mut brace_depth = 0;
                            let mut found_brace = false;
                            let mut variable_names = Vec::new();
                            let mut after_brace_idents = Vec::new();

                            loop {
                                match self.lexer.next_token() {
                                    Some(Token::Ident(IdentToken { name, .. })) => {
                                        if brace_depth == 0 && !found_brace {
                                            union_name = Some(name.to_string());
                                        } else if brace_depth == 0 && found_brace {
                                            after_brace_idents.push(name.to_string());
                                        }
                                    },
                                    Some(Token::LeftBrace(..)) => {
                                        brace_depth += 1;
                                        found_brace = true;
                                    },
                                    Some(Token::RightBrace(..)) => {
                                        brace_depth -= 1;
                                    },
                                    Some(Token::Semicolon(SemicolonToken { span: semi_span })) => {
                                        end_byte = semi_span.byte_end_idx;
                                        if brace_depth == 0 {
                                            variable_names = after_brace_idents;
                                            break;
                                        }
                                    },
                                    None => break,
                                    _ => {}
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

                            items.push(Item::UnionDecl {
                                span: final_span,
                                text,
                                union_name,
                                has_typedef: true,
                                variable_names,
                            });
                        },
                        _ => {
                            // 通常の typedef（既存の処理）
                            loop {
                                match self.lexer.next_token() {
                                    Some(Token::Semicolon(SemicolonToken { span: semi_span })) => {
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
