use crate::lexer::Lexer;
use crate::token::*;
use crate::ast::{TranslationUnit, Item, StructMember, UnionMember, EnumVariant};
use crate::span::Span;
use crate::trivia::{Trivia, Comment};
use crate::type_system::{BaseType, Type, TypeQualifier};

// パース中のコンテキスト
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseContext {
    TopLevel,       // トップレベル（ファイル直下）
    InStruct,       // struct 内部
    InUnion,        // union 内部  
    InEnum,         // enum 内部
}

// parse_items の停止理由
#[derive(Debug, Clone)]
enum StopReason {
    Elif(Span),
    Else(Span),
    Endif(Span),
    Eof,
}

#[derive(Debug)]
pub struct Parser {
    pub lexer: Lexer,
    pending_comments: Vec<Comment>,  // 次のItemに付与する予定のコメント
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Parser { 
            lexer,
            pending_comments: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> TranslationUnit {
        let (items, _) = self.parse_items(ParseContext::TopLevel, false);
        TranslationUnit { 
            items,
            leading_trivia: Trivia::empty(),  // TODO: 後で実装
        }
    }

    // stop_at_endif: true の場合、#elif/#else/#endif で停止
    // context: パース中のコンテキスト（TopLevel/InStruct/InUnion/InEnum）
    // 戻り値: (items, stop_reason)
    fn parse_items(&mut self, context: ParseContext, stop_at_endif: bool) -> (Vec<Item>, StopReason) {
        let mut items = Vec::new();

        while let Some(token) = self.lexer.next_token() {
            // struct/union/enum内部でRightBraceを検出したら終了
            if matches!(context, ParseContext::InStruct | ParseContext::InUnion | ParseContext::InEnum) {
                if matches!(token, Token::RightBrace(..)) {
                    return (items, StopReason::Eof);
                }
            }
            
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
                    let block = self.parse_conditional_block(context, span, "ifdef");
                    items.push(block);
                },
                Token::Ifndef(IfndefToken { span }) => {
                    let block = self.parse_conditional_block(context, span, "ifndef");
                    items.push(block);
                },
                Token::If(IfToken { span }) => {
                    let block = self.parse_conditional_block(context, span, "if");
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
                        // 宣言の開始位置から型情報を解析
                        let var_type = {
                            // この宣言の開始位置から新しいlexerを作成
                            let decl_text = self.lexer.input[start_byte..end_byte].trim();
                            let type_lexer = Lexer::new(decl_text);
                            let mut type_parser = Parser::new(type_lexer);
                            type_parser.parse_type()
                        };
                        
                        let trivia = self.take_trivia();
                        items.push(Item::VarDecl { 
                            span: final_span, 
                            text,
                            var_name,
                            has_initializer,
                            var_type,
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
                    let mut members = Vec::new();
                    let mut parsed_successfully = false;
                    
                    // 次のトークンをチェック
                    let next_token = self.lexer.next_token();
                    
                    match next_token {
                        Some(Token::Ident(IdentToken { name, .. })) => {
                            struct_name = Some(name.to_string());
                            
                            // 構造体名の後をチェック
                            match self.lexer.next_token() {
                                Some(Token::LeftBrace(..)) => {
                                    // struct Name { ... }
                                    let (inner_items, _) = self.parse_items(ParseContext::InStruct, false);
                                    
                                    for item in &inner_items {
                                        if let Some(member) = Self::vardecl_to_struct_member(item) {
                                            members.push(member);
                                        }
                                    }
                                    
                                    // RightBraceの後、セミコロンまで読む
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
                                    parsed_successfully = true;
                                },
                                Some(Token::Semicolon(SemicolonToken { span: semi_span })) => {
                                    // 前方宣言: struct Foo;
                                    end_byte = semi_span.byte_end_idx;
                                    parsed_successfully = true;
                                },
                                _ => {
                                    // その他は後でスキップ
                                }
                            }
                        },
                        Some(Token::LeftBrace(..)) => {
                            // 匿名struct: struct { ... } var;
                            // 従来通りスキップ（parse_itemsは呼ばない）
                            let mut brace_depth = 1;
                            loop {
                                match self.lexer.next_token() {
                                    Some(Token::LeftBrace(..)) => brace_depth += 1,
                                    Some(Token::RightBrace(..)) => {
                                        brace_depth -= 1;
                                        if brace_depth == 0 { break; }
                                    },
                                    None => break,
                                    _ => continue,
                                }
                            }
                            // }の後、変数名とセミコロンを読む
                            loop {
                                match self.lexer.next_token() {
                                    Some(Token::Ident(IdentToken { name, .. })) => {
                                        struct_name = Some(name.to_string());
                                    },
                                    Some(Token::Semicolon(SemicolonToken { span: semi_span })) => {
                                        end_byte = semi_span.byte_end_idx;
                                        break;
                                    },
                                    None => break,
                                    _ => continue,
                                }
                            }
                            parsed_successfully = true;
                        },
                        Some(Token::Semicolon(SemicolonToken { span: semi_span })) => {
                            // struct; (エラーだが無視)
                            end_byte = semi_span.byte_end_idx;
                            parsed_successfully = true;
                        },
                        _ => {
                            // その他は後でスキップ
                        }
                    }
                    
                    // parsed_successfully=falseの場合は従来のスキップロジック
                    if !parsed_successfully {
                        let mut brace_depth = 0;
                        loop {
                            match self.lexer.next_token() {
                                Some(Token::LeftBrace(..)) => brace_depth += 1,
                                Some(Token::RightBrace(..)) => {
                                    brace_depth -= 1;
                                    if brace_depth < 0 { break; }
                                },
                                Some(Token::Semicolon(SemicolonToken { span: semi_span })) => {
                                    end_byte = semi_span.byte_end_idx;
                                    if brace_depth == 0 { break; }
                                },
                                None => break,
                                _ => continue,
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
                        members,
                        trivia,
                    });
                },
                Token::Enum(EnumToken { span }) => {
                    // enum 宣言または列挙型変数宣言
                    
                    let start_byte = span.byte_start_idx;
                    let mut end_byte = span.byte_end_idx;
                    let mut enum_name: Option<String> = None;
                    let has_typedef = false;
                    let mut variable_names = Vec::new();
                    let mut variants = Vec::new();
                    let mut parsed_successfully = false;
                    
                    // 次のトークンをチェック
                    let next_token = self.lexer.next_token();
                    
                    match next_token {
                        Some(Token::Ident(IdentToken { name, .. })) => {
                            enum_name = Some(name.to_string());
                            
                            // enum名の後をチェック
                            match self.lexer.next_token() {
                                Some(Token::LeftBrace(..)) => {
                                    // enum Name { ... }
                                    // enum内部は列挙子（カンマ区切り）
                                    let mut current_name: Option<String> = None;
                                    let mut current_value: Option<i64> = None;
                                    let mut variant_start_line = self.lexer.line;
                                    let mut variant_start_col = self.lexer.column;
                                    let mut expect_value = false;  // = の直後かどうか
                                    
                                    loop {
                                        match self.lexer.next_token() {
                                            Some(Token::Ident(IdentToken { name, span: id_span })) => {
                                                if expect_value {
                                                    // = の後の数値
                                                    if let Ok(val) = name.parse::<i64>() {
                                                        current_value = Some(val);
                                                    }
                                                    expect_value = false;
                                                } else if name == "=" {
                                                    // 次に数値が来る
                                                    expect_value = true;
                                                } else if name == "," {
                                                    // 前の列挙子を保存してリセット
                                                    if let Some(prev_name) = current_name.take() {
                                                        variants.push(EnumVariant {
                                                            name: prev_name,
                                                            value: current_value.take(),
                                                            span: Span::new(variant_start_line, variant_start_col, id_span.start_line, id_span.start_column),
                                                        });
                                                    }
                                                    variant_start_line = self.lexer.line;
                                                    variant_start_col = self.lexer.column;
                                                    current_value = None;
                                                } else {
                                                    // 列挙子名
                                                    if let Some(prev_name) = current_name.take() {
                                                        // 前の列挙子を保存
                                                        variants.push(EnumVariant {
                                                            name: prev_name,
                                                            value: current_value.take(),
                                                            span: Span::new(variant_start_line, variant_start_col, id_span.start_line, id_span.start_column),
                                                        });
                                                    }
                                                    current_name = Some(name.to_string());
                                                    variant_start_line = id_span.start_line;
                                                    variant_start_col = id_span.start_column;
                                                }
                                            },
                                            Some(Token::RightBrace(..)) => {
                                                // 最後の列挙子を保存
                                                if let Some(name) = current_name.take() {
                                                    variants.push(EnumVariant {
                                                        name,
                                                        value: current_value.take(),
                                                        span: Span::new(variant_start_line, variant_start_col, self.lexer.line, self.lexer.column),
                                                    });
                                                }
                                                break;
                                            },
                                            None => break,
                                            _ => continue,
                                        }
                                    }
                                    
                                    // }の後、変数名とセミコロンを読む
                                    loop {
                                        match self.lexer.next_token() {
                                            Some(Token::Ident(IdentToken { name, .. })) => {
                                                variable_names.push(name.to_string());
                                            },
                                            Some(Token::Semicolon(SemicolonToken { span: semi_span })) => {
                                                end_byte = semi_span.byte_end_idx;
                                                break;
                                            },
                                            None => break,
                                            _ => continue,
                                        }
                                    }
                                    parsed_successfully = true;
                                },
                                Some(Token::Semicolon(SemicolonToken { span: semi_span })) => {
                                    // 前方宣言: enum Foo;
                                    end_byte = semi_span.byte_end_idx;
                                    parsed_successfully = true;
                                },
                                _ => {}
                            }
                        },
                        Some(Token::LeftBrace(..)) => {
                            // 匿名enum（従来通りスキップ）
                            let mut brace_depth = 1;
                            loop {
                                match self.lexer.next_token() {
                                    Some(Token::LeftBrace(..)) => brace_depth += 1,
                                    Some(Token::RightBrace(..)) => {
                                        brace_depth -= 1;
                                        if brace_depth == 0 { break; }
                                    },
                                    None => break,
                                    _ => continue,
                                }
                            }
                            loop {
                                match self.lexer.next_token() {
                                    Some(Token::Ident(IdentToken { name, .. })) => {
                                        variable_names.push(name.to_string());
                                    },
                                    Some(Token::Semicolon(SemicolonToken { span: semi_span })) => {
                                        end_byte = semi_span.byte_end_idx;
                                        break;
                                    },
                                    None => break,
                                    _ => continue,
                                }
                            }
                            parsed_successfully = true;
                        },
                        _ => {}
                    }
                    
                    // 従来のスキップロジック
                    if !parsed_successfully {
                        let mut brace_depth = 0;
                        let mut found_brace = false;
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
                                None => break,
                                _ => continue,
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
                        variants,
                        trivia,
                    });
                },
                Token::Union(UnionToken { span }) => {
                    // union 宣言
                    
                    let start_byte = span.byte_start_idx;
                    let mut end_byte = span.byte_end_idx;
                    let mut union_name: Option<String> = None;
                    let has_typedef = false;
                    let mut variable_names = Vec::new();
                    let mut members = Vec::new();
                    let mut parsed_successfully = false;
                    
                    // 次のトークンをチェック
                    let next_token = self.lexer.next_token();
                    
                    match next_token {
                        Some(Token::Ident(IdentToken { name, .. })) => {
                            union_name = Some(name.to_string());
                            
                            // union名の後をチェック
                            match self.lexer.next_token() {
                                Some(Token::LeftBrace(..)) => {
                                    // union Name { ... }
                                    let (inner_items, _) = self.parse_items(ParseContext::InUnion, false);
                                    
                                    for item in &inner_items {
                                        if let Some(member) = Self::vardecl_to_union_member(item) {
                                            members.push(member);
                                        }
                                    }
                                    
                                    // RightBraceの後、セミコロンまで読む
                                    loop {
                                        match self.lexer.next_token() {
                                            Some(Token::Ident(IdentToken { name, .. })) => {
                                                variable_names.push(name.to_string());
                                            },
                                            Some(Token::Semicolon(SemicolonToken { span: semi_span })) => {
                                                end_byte = semi_span.byte_end_idx;
                                                break;
                                            },
                                            Some(_) => continue,
                                            None => break,
                                        }
                                    }
                                    parsed_successfully = true;
                                },
                                Some(Token::Semicolon(SemicolonToken { span: semi_span })) => {
                                    // 前方宣言: union Foo;
                                    end_byte = semi_span.byte_end_idx;
                                    parsed_successfully = true;
                                },
                                _ => {}
                            }
                        },
                        Some(Token::LeftBrace(..)) => {
                            // 匿名union（従来通りスキップ）
                            let mut brace_depth = 1;
                            loop {
                                match self.lexer.next_token() {
                                    Some(Token::LeftBrace(..)) => brace_depth += 1,
                                    Some(Token::RightBrace(..)) => {
                                        brace_depth -= 1;
                                        if brace_depth == 0 { break; }
                                    },
                                    None => break,
                                    _ => continue,
                                }
                            }
                            loop {
                                match self.lexer.next_token() {
                                    Some(Token::Ident(IdentToken { name, .. })) => {
                                        variable_names.push(name.to_string());
                                    },
                                    Some(Token::Semicolon(SemicolonToken { span: semi_span })) => {
                                        end_byte = semi_span.byte_end_idx;
                                        break;
                                    },
                                    None => break,
                                    _ => continue,
                                }
                            }
                            parsed_successfully = true;
                        },
                        _ => {}
                    }
                    
                    // 従来のスキップロジック
                    if !parsed_successfully {
                        let mut brace_depth = 0;
                        let mut found_brace = false;
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
                                _ => continue,
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
                        members,
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
                                members: Vec::new(),  // TODO: 後で実装
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
                                variants: Vec::new(),  // TODO: 後で実装
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
                                members: Vec::new(),  // TODO: 後で実装
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
    fn parse_conditional_block(&mut self, context: ParseContext, start_span: Span, directive_type: &str) -> Item {
        let condition = self.extract_condition(&start_span);
        
        // このブロック（#ifdef/#ifndef/#if）内のアイテムを解析
        let (mut block_items, stop_reason) = self.parse_items(context, true);
        
        // parse_items が終了した理由を確認（#elif, #else, #endif のいずれか、またはEOF）
        match stop_reason {
            StopReason::Elif(span) => {
                // #elif ブロックを子アイテムとして追加し、再帰的に処理
                let elif_block = self.parse_conditional_block(context, span, "elif");
                
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
                let (else_items, end_reason) = self.parse_items(context, true);
                
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

    /// VarDeclをStructMemberに変換
    fn vardecl_to_struct_member(item: &Item) -> Option<StructMember> {
        if let Item::VarDecl { var_name, var_type, span, .. } = item {
            Some(StructMember {
                name: var_name.clone(),
                member_type: var_type.clone(),
                bitfield_width: None,  // TODO: ビットフィールド解析
                span: span.clone(),
            })
        } else {
            None
        }
    }

    /// VarDeclをUnionMemberに変換
    fn vardecl_to_union_member(item: &Item) -> Option<UnionMember> {
        if let Item::VarDecl { var_name, var_type, span, .. } = item {
            Some(UnionMember {
                name: var_name.clone(),
                member_type: var_type.clone(),
                span: span.clone(),
            })
        } else {
            None
        }
    }

    /// Get the span from a token
    fn get_token_span(token: &Token) -> Span {
        match token {
            Token::BlockComment(t) => t.span.clone(),
            Token::Include(t) => t.span.clone(),
            Token::Define(t) => t.span.clone(),
            Token::Semicolon(t) => t.span.clone(),
            Token::Equal(t) => t.span.clone(),
            Token::Asterisk(t) => t.span.clone(),
            Token::Ident(t) => t.span.clone(),
            Token::Auto(t) => t.span.clone(),
            Token::Register(t) => t.span.clone(),
            Token::Static(t) => t.span.clone(),
            Token::Extern(t) => t.span.clone(),
            Token::Typedef(t) => t.span.clone(),
            Token::Const(t) => t.span.clone(),
            Token::Volatile(t) => t.span.clone(),
            Token::Restrict(t) => t.span.clone(),
            Token::Atomic(t) => t.span.clone(),
            Token::Int(t) => t.span.clone(),
            Token::Char(t) => t.span.clone(),
            Token::Float(t) => t.span.clone(),
            Token::Double(t) => t.span.clone(),
            Token::Void(t) => t.span.clone(),
            Token::Long(t) => t.span.clone(),
            Token::Short(t) => t.span.clone(),
            Token::Signed(t) => t.span.clone(),
            Token::Unsigned(t) => t.span.clone(),
            Token::Struct(t) => t.span.clone(),
            Token::Enum(t) => t.span.clone(),
            Token::Union(t) => t.span.clone(),
            Token::LeftBrace(t) => t.span.clone(),
            Token::RightBrace(t) => t.span.clone(),
            Token::LeftParen(t) => t.span.clone(),
            Token::RightParen(t) => t.span.clone(),
            Token::Ifdef(t) => t.span.clone(),
            Token::Ifndef(t) => t.span.clone(),
            Token::If(t) => t.span.clone(),
            Token::Elif(t) => t.span.clone(),
            Token::Else(t) => t.span.clone(),
            Token::Endif(t) => t.span.clone(),
            Token::LineComment(t) => t.span.clone(),
        }
    }

    /// Parse a C type (base type with optional qualifiers and pointer layers)
    /// Returns None if the current token doesn't start a type
    pub fn parse_type(&mut self) -> Option<Type> {
        let mut base_qualifiers = Vec::new();
        let base_type: Option<BaseType>;
        let mut last_span: Option<Span>;

        // Phase 1: Parse base qualifiers and base type
        loop {
            let token = self.lexer.next_token()?;
            let token_span = Self::get_token_span(&token);
            
            last_span = Some(token_span.clone());

            match token {
                // Type qualifiers
                Token::Const(_) => {
                    base_qualifiers.push(TypeQualifier::Const);
                }
                Token::Volatile(_) => {
                    base_qualifiers.push(TypeQualifier::Volatile);
                }
                Token::Restrict(_) => {
                    base_qualifiers.push(TypeQualifier::Restrict);
                }
                Token::Atomic(_) => {
                    base_qualifiers.push(TypeQualifier::Atomic);
                }
                // Base types
                Token::Void(_) => {
                    base_type = Some(BaseType::Void);
                    break;
                }
                Token::Char(_) => {
                    base_type = Some(BaseType::Char);
                    break;
                }
                Token::Short(_) => {
                    base_type = Some(BaseType::Short);
                    break;
                }
                Token::Int(_) => {
                    base_type = Some(BaseType::Int);
                    break;
                }
                Token::Long(_) => {
                    base_type = Some(BaseType::Long);
                    break;
                }
                Token::Float(_) => {
                    base_type = Some(BaseType::Float);
                    break;
                }
                Token::Double(_) => {
                    base_type = Some(BaseType::Double);
                    break;
                }
                Token::Signed(_) => {
                    base_type = Some(BaseType::Signed);
                    break;
                }
                Token::Unsigned(_) => {
                    base_type = Some(BaseType::Unsigned);
                    break;
                }
                _ => {
                    // Not a type token - we're done (or invalid)
                    return None;
                }
            }
        }

        let base_type = base_type?;
        let mut end_span = last_span?;

        // フェーズ2: ポインタ層を解析（アスタリスクと修飾子）
        let mut pointer_layers = Vec::new();
        
        'pointer_loop: loop {
            // Try to get next token - if there's no more tokens, we're done
            let token = match self.lexer.next_token() {
                Some(t) => t,
                None => break,
            };

            match token {
                Token::Asterisk(ast_token) => {
                    let asterisk_span = ast_token.span.clone();
                    end_span = asterisk_span.clone();
                    let mut qualifiers = Vec::new();

                    // アスタリスクの後の修飾子をチェック
                    'qualifier_loop: loop {
                        let next_token = match self.lexer.next_token() {
                            Some(t) => t,
                            None => {
                                // No more tokens - save this pointer layer and exit
                                pointer_layers.push(crate::type_system::PointerLayer::with_qualifiers(
                                    qualifiers,
                                    asterisk_span,
                                ));
                                break 'pointer_loop;
                            }
                        };

                        match next_token {
                            Token::Const(_) => {
                                qualifiers.push(TypeQualifier::Const);
                                end_span = Self::get_token_span(&next_token);
                            }
                            Token::Volatile(_) => {
                                qualifiers.push(TypeQualifier::Volatile);
                                end_span = Self::get_token_span(&next_token);
                            }
                            Token::Restrict(_) => {
                                qualifiers.push(TypeQualifier::Restrict);
                                end_span = Self::get_token_span(&next_token);
                            }
                            Token::Atomic(_) => {
                                qualifiers.push(TypeQualifier::Atomic);
                                end_span = Self::get_token_span(&next_token);
                            }
                            Token::Asterisk(next_ast) => {
                                // Another asterisk - save current layer and continue with outer loop
                                pointer_layers.push(crate::type_system::PointerLayer::with_qualifiers(
                                    qualifiers,
                                    asterisk_span,
                                ));
                                
                                // Prepare for next asterisk processing
                                let new_asterisk_span = next_ast.span.clone();
                                end_span = new_asterisk_span.clone();
                                let mut new_qualifiers = Vec::new();

                                // Parse qualifiers for this asterisk
                                loop {
                                    let qual_token = match self.lexer.next_token() {
                                        Some(t) => t,
                                        None => {
                                            pointer_layers.push(crate::type_system::PointerLayer::with_qualifiers(
                                                new_qualifiers,
                                                new_asterisk_span,
                                            ));
                                            break 'pointer_loop;
                                        }
                                    };

                                    match qual_token {
                                        Token::Const(_) => {
                                            new_qualifiers.push(TypeQualifier::Const);
                                            end_span = Self::get_token_span(&qual_token);
                                        }
                                        Token::Volatile(_) => {
                                            new_qualifiers.push(TypeQualifier::Volatile);
                                            end_span = Self::get_token_span(&qual_token);
                                        }
                                        Token::Restrict(_) => {
                                            new_qualifiers.push(TypeQualifier::Restrict);
                                            end_span = Self::get_token_span(&qual_token);
                                        }
                                        Token::Atomic(_) => {
                                            new_qualifiers.push(TypeQualifier::Atomic);
                                            end_span = Self::get_token_span(&qual_token);
                                        }
                                        Token::Asterisk(third_ast) => {
                                            // Third asterisk - save current and continue outer loop
                                            pointer_layers.push(crate::type_system::PointerLayer::with_qualifiers(
                                                new_qualifiers,
                                                new_asterisk_span,
                                            ));
                                            
                                            // Process third asterisk by continuing outer loop
                                            let third_span = third_ast.span.clone();
                                            end_span = third_span.clone();
                                            pointer_layers.push(crate::type_system::PointerLayer::with_qualifiers(
                                                Vec::new(),
                                                third_span,
                                            ));
                                            
                                            // Continue outer loop to process more tokens
                                            break 'qualifier_loop;
                                        }
                                        _ => {
                                            // Not a qualifier - save this layer and exit
                                            pointer_layers.push(crate::type_system::PointerLayer::with_qualifiers(
                                                new_qualifiers,
                                                new_asterisk_span,
                                            ));
                                            break 'pointer_loop;
                                        }
                                    }
                                }
                            }
                            _ => {
                                // Not a qualifier or asterisk - save this layer and exit
                                pointer_layers.push(crate::type_system::PointerLayer::with_qualifiers(
                                    qualifiers,
                                    asterisk_span,
                                ));
                                break 'pointer_loop;
                            }
                        }
                    }
                }
                _ => {
                    // アスタリスクではない - ポインタ層の解析終了
                    break;
                }
            }
        }

        Some(Type::with_pointers(
            base_type,
            base_qualifiers,
            pointer_layers,
            end_span,
        ))
    }
}
