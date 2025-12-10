use crate::lexer::Lexer;
use crate::token::*;
use crate::ast::{TranslationUnit, Item};
use crate::span::Span;
use crate::trivia::{Trivia, Comment};

// parse_items の停止理由
#[derive(Debug, Clone)]
enum StopReason {
    Elif(Span),
    Else(Span),
    Endif(Span),
    Eof,
}

#[derive(Debug)]
pub struct Parser<'a> {
    pub lexer: Lexer<'a>,
    pending_comments: Vec<Comment>,  // 次のItemに付与する予定のコメント
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Parser { 
            lexer,
            pending_comments: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> TranslationUnit {
        let (items, _) = self.parse_items(false);
        TranslationUnit { 
            items,
            leading_trivia: Trivia::empty(),  // TODO: 後で実装
        }
    }

    // stop_at_endif: true の場合、#elif/#else/#endif で停止
    // 戻り値: (items, stop_reason)
    fn parse_items(&mut self, stop_at_endif: bool) -> (Vec<Item>, StopReason) {
        let mut items = Vec::new();

        while let Some(token) = self.lexer.next_token() {
            match token {
                Token::BlockComment(BlockCommentToken { span }) => {
                    let text = self.lexer.input[span.byte_start_idx..span.byte_end_idx].to_string();
                    self.pending_comments.push(Comment::Block { text, span });
                    continue;
                },
                Token::LineComment(LineCommentToken { span }) => {
                    let text = self.lexer.input[span.byte_start_idx..span.byte_end_idx].to_string();
                    self.pending_comments.push(Comment::Line { text, span });
                    continue;
                },
                Token::Include(IncludeToken { span, filename }) => {
                    let text = self.lexer.input[span.byte_start_idx..span.byte_end_idx].to_string();
                    let trivia = self.take_trivia();
                    items.push(Item::Include { span, text, filename, trivia });
                },
                Token::Define(DefineToken { span, macro_name, macro_value }) => {
                    let text = self.lexer.input[span.byte_start_idx..span.byte_end_idx].to_string();
                    let trivia = self.take_trivia();
                    items.push(Item::Define { span, text, macro_name, macro_value, trivia });
                },
                // Stage 1: 条件コンパイルブロック
                Token::Ifdef(IfdefToken { span }) => {
                    let block = self.parse_conditional_block(span, "ifdef");
                    items.push(block);
                },
                Token::Ifndef(IfndefToken { span }) => {
                    let block = self.parse_conditional_block(span, "ifndef");
                    items.push(block);
                },
                Token::If(IfToken { span }) => {
                    let block = self.parse_conditional_block(span, "if");
                    items.push(block);
                },
                // Stage 2: #elif, #else, #endif の処理
                Token::Elif(ElifToken { span }) => {
                    if stop_at_endif {
                        // 条件コンパイルブロック内で#elifに遭遇 - 停止して理由を返す
                        return (items, StopReason::Elif(span));
                    } else {
                        // エラー：対応する #ifdef がない（とりあえず無視）
                        continue;
                    }
                },
                Token::Else(ElseToken { span }) => {
                    if stop_at_endif {
                        // 条件コンパイルブロック内で#elseに遭遇 - 停止して理由を返す
                        return (items, StopReason::Else(span));
                    } else {
                        // エラー：対応する #ifdef がない（とりあえず無視）
                        continue;
                    }
                },
                Token::Endif(EndifToken { span }) => {
                    if stop_at_endif {
                        // 条件コンパイルブロックの終わり - 停止して理由を返す
                        return (items, StopReason::Endif(span));
                    } else {
                        // エラー：対応する #ifdef がない（とりあえず無視）
                        continue;
                    }
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
                        let full_prefix = self.lexer.input[start_byte..function_name_start].trim();
                        
                        // storage class を抽出
                        let storage_class = if full_prefix.starts_with("static ") {
                            Some("static".to_string())
                        } else if full_prefix.starts_with("extern ") {
                            Some("extern".to_string())
                        } else {
                            None
                        };
                        
                        // return_type から storage class を除外
                        let return_type = if let Some(ref sc) = storage_class {
                            full_prefix.strip_prefix(sc).unwrap_or(full_prefix).trim().to_string()
                        } else {
                            full_prefix.to_string()
                        };
                        
                        let parameters = self.lexer.input[params_start_byte..params_end_byte].to_string();
                        
                        let trivia = self.take_trivia();
                        items.push(Item::FunctionDecl {
                            span: final_span,
                            text,
                            return_type,
                            function_name,
                            parameters,
                            storage_class,
                            trivia,
                        });
                    } else {
                        // 変数宣言
                        let trivia = self.take_trivia();
                        items.push(Item::VarDecl { 
                            span: final_span, 
                            text,
                            var_name,
                            has_initializer,
                            trivia,
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
                    let trivia = self.take_trivia();
                    items.push(Item::StructDecl { 
                        span: final_span, 
                        text,
                        struct_name,
                        has_typedef,
                        trivia,
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
                    let trivia = self.take_trivia();
                    items.push(Item::EnumDecl { 
                        span: final_span,
                        text,
                        enum_name,
                        has_typedef,
                        variable_names,
                        trivia,
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
                    let trivia = self.take_trivia();
                    items.push(Item::UnionDecl { 
                        span: final_span,
                        text,
                        union_name,
                        has_typedef,
                        variable_names,
                        trivia,
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
                            let trivia = self.take_trivia();
                            items.push(Item::StructDecl { 
                                span: final_span, 
                                text,
                                struct_name,
                                has_typedef: true,
                                trivia,
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
                            let trivia = self.take_trivia();
                            items.push(Item::EnumDecl { 
                                span: final_span,
                                text,
                                enum_name,
                                has_typedef: true,
                                variable_names,
                                trivia,
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

                            let trivia = self.take_trivia();
                            items.push(Item::UnionDecl { 
                                span: final_span,
                                text,
                                union_name,
                                has_typedef: true,
                                variable_names,
                                trivia,
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
                            let trivia = self.take_trivia();
                            items.push(Item::TypedefDecl { span: final_span, text, trivia });
                        }
                    }
                },
                _ => {
                    continue;
                }
            }
        }

        (items, StopReason::Eof)
    }

    // Stage 2: 条件コンパイルブロックを解析（#ifdef から #endif まで）
    // #elif/#else も子ブロックとして扱う
    fn parse_conditional_block(&mut self, start_span: Span, directive_type: &str) -> Item {
        let condition = self.extract_condition(&start_span);
        
        // このブロック（#ifdef/#ifndef/#if）内のアイテムを解析
        let (mut block_items, stop_reason) = self.parse_items(true);
        
        // parse_items が終了した理由を確認（#elif, #else, #endif のいずれか、またはEOF）
        match stop_reason {
            StopReason::Elif(span) => {
                // #elif ブロックを子アイテムとして追加し、再帰的に処理
                let elif_block = self.parse_conditional_block(span, "elif");
                
                // end_span は #elif ブロックの end_span を使う
                let end_span = if let Item::ConditionalBlock { end_span, .. } = &elif_block {
                    end_span.clone()
                } else {
                    start_span.clone()
                };
                
                block_items.push(elif_block);
                
                return Item::ConditionalBlock {
                    directive_type: directive_type.to_string(),
                    condition,
                    items: block_items,
                    start_span,
                    end_span,
                    trivia: Trivia::empty(),
                };
            },
            StopReason::Else(span) => {
                // #else ブロックを子アイテムとして追加
                let (else_items, end_reason) = self.parse_items(true);
                
                // #else の後は必ず #endif が来るはず
                let end_span = if let StopReason::Endif(span) = end_reason {
                    span
                } else {
                    span.clone()
                };
                
                let else_block = Item::ConditionalBlock {
                    directive_type: "else".to_string(),
                    condition: String::new(),
                    items: else_items,
                    start_span: span.clone(),
                    end_span: end_span.clone(),
                    trivia: Trivia::empty(),
                };
                block_items.push(else_block);
                
                // #endif を子アイテムとして追加
                block_items.push(Item::ConditionalBlock {
                    directive_type: "endif".to_string(),
                    condition: String::new(),
                    items: Vec::new(),
                    start_span: end_span.clone(),
                    end_span: end_span.clone(),
                    trivia: Trivia::empty(),
                });
                
                return Item::ConditionalBlock {
                    directive_type: directive_type.to_string(),
                    condition,
                    items: block_items,
                    start_span,
                    end_span,
                    trivia: Trivia::empty(),
                };
            },
            StopReason::Endif(span) => {
                // #endif で終了
                let end_span = span;
                
                // #endif を子アイテムとして追加
                block_items.push(Item::ConditionalBlock {
                    directive_type: "endif".to_string(),
                    condition: String::new(),
                    items: Vec::new(),
                    start_span: end_span.clone(),
                    end_span: end_span.clone(),
                    trivia: Trivia::empty(),
                });
                
                return Item::ConditionalBlock {
                    directive_type: directive_type.to_string(),
                    condition,
                    items: block_items,
                    start_span,
                    end_span,
                    trivia: Trivia::empty(),
                };
            },
            StopReason::Eof => {
                // EOF など、#endif がない場合
                return Item::ConditionalBlock {
                    directive_type: directive_type.to_string(),
                    condition,
                    items: block_items,
                    start_span: start_span.clone(),
                    end_span: start_span,
                    trivia: Trivia::empty(),
                };
            }
        }
    }

    // 条件式を抽出
    fn extract_condition(&self, span: &Span) -> String {
        let text = &self.lexer.input[span.byte_start_idx..span.byte_end_idx];
        // "#ifdef DEBUG\n" -> "DEBUG" を抽出
        let content = text.trim_start_matches('#').trim();
        
        if let Some(rest) = content.strip_prefix("ifdef") {
            rest.trim().trim_end_matches(&['\r', '\n'][..]).to_string()
        } else if let Some(rest) = content.strip_prefix("ifndef") {
            rest.trim().trim_end_matches(&['\r', '\n'][..]).to_string()
        } else if let Some(rest) = content.strip_prefix("elif") {
            rest.trim().trim_end_matches(&['\r', '\n'][..]).to_string()
        } else if content.starts_with("if") {
            content.strip_prefix("if").unwrap().trim().trim_end_matches(&['\r', '\n'][..]).to_string()
        } else {
            String::new()
        }
    }
    /// pending_commentsを取り出してTriviaを作成
    fn take_trivia(&mut self) -> Trivia {
        let leading = std::mem::take(&mut self.pending_comments);
        Trivia {
            leading,
            trailing: Vec::new(),  // TODO: 後で実装
        }
    }
}
