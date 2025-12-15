use crate::lexer::Lexer;
use crate::token::*;
use crate::ast::{TranslationUnit, Item, StructMember, UnionMember, EnumVariant};
use crate::span::Span;
use crate::trivia::{Trivia, Comment};
use crate::type_system::{BaseType, Type, TypeQualifier};
use crate::type_table::TypeTable;
use std::collections::HashMap;

// パース中のコンテキスト
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseContext {
    TopLevel,       // トップレベル（ファイル直下）
    InStruct,       // struct 内部
    InUnion,        // union 内部  
    #[allow(dead_code)]
    InEnum,         // enum 内部（将来の拡張用に予約）
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
    type_table: TypeTable,           // typedef名を管理
    defined_macros: HashMap<String, String>,  // #define で定義されたマクロ
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        let mut type_table = TypeTable::new();
        
        // 組み込み型名を事前登録（文字列のみ）
        let predefined_types = [
            "VU8", "VU16", "VU32", "VU64",
            "VS8", "VS16", "VS32", "VS64",
            "CU8", "CU16", "CU32", "CU64",
            "CS8", "CS16", "CS32", "CS64",
        ];
        
        for type_name in &predefined_types {
            // Spanはダミー値を使用
            let dummy_span = crate::span::Span::new(0, 0, 0, 0);
            type_table.register_type(
                type_name.to_string(),
                crate::type_system::Type::new(
                    crate::type_system::BaseType::Int,  // プレースホルダー
                    dummy_span
                )
            );
        }
        
        Parser { 
            lexer,
            pending_comments: Vec::new(),
            type_table,
            defined_macros: HashMap::new(),
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
                    // マクロを登録
                    self.defined_macros.insert(macro_name.clone(), macro_value.clone());
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
                    let mut has_function_body = false;
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
                                // 関数本体の開始
                                has_function_body = true;
                                // LeftBraceはすでにnext_token()で消費済み
                                // 関数本体全体をスキップ
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
                        
                        // 関数本体があるかチェック
                        let body = if has_function_body {
                            // textから関数本体部分を抽出して再解析
                            // text全体から{ }を見つけて、その中を解析
                            let full_text = &self.lexer.input[start_byte..end_byte];
                            if let Some(brace_start) = full_text.find('{') {
                                if let Some(brace_end) = full_text.rfind('}') {
                                    let body_text = &full_text[brace_start+1..brace_end];
                                    // 新しいlexerとparserでbody_textを解析
                                    let body_lexer = Lexer::new(body_text);
                                    let mut body_parser = Parser::new(body_lexer);
                                    // body_textは{}の中身なので、直接ステートメントを解析
                                    let mut statements = Vec::new();
                                    body_parser.type_table.push_scope();
                                    loop {
                                        if body_parser.lexer.peek_token().is_none() {
                                            break;
                                        }
                                        if let Some(stmt) = body_parser.parse_statement() {
                                            statements.push(stmt);
                                        } else {
                                            // 解析できない場合はスキップ
                                            body_parser.lexer.next_token();
                                        }
                                    }
                                    body_parser.type_table.pop_scope();
                                    Some(statements)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        
                        let trivia = self.take_trivia();
                        items.push(Item::FunctionDecl {
                            span: final_span,
                            text,
                            return_type,
                            function_name,
                            parameters,
                            storage_class,
                            body,
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
                        variable_names: Vec::new(),  // TODO: 後で実装
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
                                            Some(Token::NumberLiteral(NumberLiteralToken { value, .. })) => {
                                                if expect_value {
                                                    // = の後の数値リテラル
                                                    // suffixを除去 (u, U, l, L, ll, LL等)
                                                    let value_without_suffix = value.trim_end_matches(|c: char| {
                                                        c == 'u' || c == 'U' || c == 'l' || c == 'L'
                                                    });
                                                    
                                                    // 16進数 (0x), 8進数 (0), 10進数に対応
                                                    let parsed_value = if value_without_suffix.starts_with("0x") || value_without_suffix.starts_with("0X") {
                                                        i64::from_str_radix(&value_without_suffix[2..], 16).ok()
                                                    } else if value_without_suffix.starts_with("0") && value_without_suffix.len() > 1 {
                                                        i64::from_str_radix(&value_without_suffix[1..], 8).ok()
                                                    } else {
                                                        value_without_suffix.parse::<i64>().ok()
                                                    };
                                                    current_value = parsed_value;
                                                    expect_value = false;
                                                }
                                            },
                                            Some(Token::Equal(..)) => {
                                                // = の後に数値が来る
                                                expect_value = true;
                                            },
                                            Some(Token::Ident(IdentToken { name, span: id_span })) => {
                                                if expect_value {
                                                    // 後方互換性: Identとして数値が来る場合（本来はNumberLiteralが来るべき）
                                                    if let Ok(val) = name.parse::<i64>() {
                                                        current_value = Some(val);
                                                    }
                                                    expect_value = false;
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
                    let next_tok = self.lexer.next_token();
                    match next_tok {
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
                                text: text.clone(),
                                struct_name: struct_name.clone(),
                                has_typedef: true,
                                variable_names: Vec::new(),  // TODO: 後で実装
                                members: Vec::new(),  // TODO: 後で実装
                                trivia,
                            });
                            // 型テーブルに登録
                            self.register_typedef_name(&text);
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
                                text: text.clone(),
                                enum_name: enum_name.clone(),
                                has_typedef: true,
                                variable_names: variable_names.clone(),
                                variants: Vec::new(),  // TODO: 後で実装
                                trivia,
                            });
                            // 型テーブルに登録
                            self.register_typedef_name(&text);
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
                                text: text.clone(),
                                union_name: union_name.clone(),
                                has_typedef: true,
                                variable_names: variable_names.clone(),
                                members: Vec::new(),  // TODO: 後で実装
                                trivia,
                            });
                            // 型テーブルに登録
                            self.register_typedef_name(&text);
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
                            items.push(Item::TypedefDecl { span: final_span, text: text.clone(), trivia });
                            // 型テーブルに登録
                            self.register_typedef_name(&text);
                        }
                    }
                },
                Token::Ident(IdentToken { span, name }) => {
                    // 識別子が型名として登録されているかチェック
                    if self.type_table.is_type_name(&name) {
                        // typedef型を使った変数宣言として処理
                        let start_byte = span.byte_start_idx;
                        let mut end_byte = span.byte_end_idx;
                        let mut var_name = String::new();
                        let mut has_initializer = false;
                        
                        // 次の識別子が変数名
                        if let Some(Token::Ident(IdentToken { span: var_span, name: vname })) = self.lexer.next_token() {
                            var_name = vname;
                            end_byte = var_span.byte_end_idx;
                            
                            // セミコロンまたは初期化子を探す
                            loop {
                                match self.lexer.next_token() {
                                    Some(Token::Equal(..)) => {
                                        has_initializer = true;
                                    },
                                    Some(Token::Semicolon(SemicolonToken { span: semi_span })) => {
                                        end_byte = semi_span.byte_end_idx;
                                        break;
                                    },
                                    Some(_) => {
                                        // 初期化式の中身は無視
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
                            
                            // 型情報を作成（簡易版 - Intをプレースホルダーとして使用）
                            let var_type = Some(crate::type_system::Type::new(
                                crate::type_system::BaseType::Int,
                                span.clone()
                            ));
                            
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
                    } else {
                        // 型名ではない識別子は無視
                        continue;
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
        
        // 条件を評価
        let condition_result = self.evaluate_condition(directive_type, &condition);
        
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
                    condition: condition.clone(),
                    condition_result,
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
                    condition_result: !condition_result,  // elseはifの逆
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
                    condition_result: true,  // endifは常にtrue
                    items: Vec::new(),
                    start_span: end_span.clone(),
                    end_span: end_span.clone(),
                    trivia: Trivia::empty(),
                });
                
                return Item::ConditionalBlock {
                    directive_type: directive_type.to_string(),
                    condition: condition.clone(),
                    condition_result,
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
                    condition_result: true,  // endifは常にtrue
                    items: Vec::new(),
                    start_span: end_span.clone(),
                    end_span: end_span.clone(),
                    trivia: Trivia::empty(),
                });
                
                return Item::ConditionalBlock {
                    directive_type: directive_type.to_string(),
                    condition: condition.clone(),
                    condition_result,
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
                    condition: condition.clone(),
                    condition_result,
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

    // 条件式を評価（#ifdef, #ifndef, #if）
    fn evaluate_condition(&self, directive_type: &str, condition: &str) -> bool {
        match directive_type {
            "ifdef" => {
                // マクロが定義されていればtrue
                self.defined_macros.contains_key(condition)
            },
            "ifndef" => {
                // マクロが定義されていなければtrue
                !self.defined_macros.contains_key(condition)
            },
            "if" | "elif" => {
                // 簡易的な式評価
                self.evaluate_if_expression(condition)
            },
            _ => true, // 不明なディレクティブはtrueとして扱う
        }
    }

    // #if の式を評価（簡易実装）
    fn evaluate_if_expression(&self, expr: &str) -> bool {
        let expr = expr.trim();
        
        // 論理演算子 && と || のサポート（最優先で処理）
        if expr.contains("&&") {
            let parts: Vec<&str> = expr.split("&&").collect();
            return parts.iter().all(|p| self.evaluate_if_expression(p.trim()));
        }
        
        if expr.contains("||") {
            let parts: Vec<&str> = expr.split("||").collect();
            return parts.iter().any(|p| self.evaluate_if_expression(p.trim()));
        }
        
        // defined(MACRO) のパターンをチェック
        if let Some(rest) = expr.strip_prefix("defined(") {
            if let Some(macro_name) = rest.strip_suffix(')') {
                return self.defined_macros.contains_key(macro_name.trim());
            }
        }
        
        // defined MACRO のパターンをチェック
        if let Some(macro_name) = expr.strip_prefix("defined ") {
            return self.defined_macros.contains_key(macro_name.trim());
        }
        
        // !defined(MACRO) のパターン
        if let Some(rest) = expr.strip_prefix("!defined(") {
            if let Some(macro_name) = rest.strip_suffix(')') {
                return !self.defined_macros.contains_key(macro_name.trim());
            }
        }
        
        // 数値リテラルの評価
        if let Ok(num) = expr.parse::<i64>() {
            return num != 0;
        }
        
        // マクロ名の展開と評価
        if let Some(value) = self.defined_macros.get(expr) {
            if let Ok(num) = value.parse::<i64>() {
                return num != 0;
            }
            return !value.is_empty();
        }
        
        // 比較演算子のサポート（簡易版）
        for op in &["==", "!=", ">=", "<=", ">", "<"] {
            if expr.contains(op) {
                let parts: Vec<&str> = expr.split(op).collect();
                if parts.len() == 2 {
                    let left = self.evaluate_macro_value(parts[0].trim());
                    let right = self.evaluate_macro_value(parts[1].trim());
                    
                    return match *op {
                        "==" => left == right,
                        "!=" => left != right,
                        ">" => left > right,
                        "<" => left < right,
                        ">=" => left >= right,
                        "<=" => left <= right,
                        _ => false,
                    };
                }
            }
        }
        
        // デフォルトはfalse（未定義のマクロ）
        false
    }

    // マクロ値を数値として評価
    fn evaluate_macro_value(&self, expr: &str) -> i64 {
        let expr = expr.trim();
        
        // 数値リテラル
        if let Ok(num) = expr.parse::<i64>() {
            return num;
        }
        
        // マクロ名の展開
        if let Some(value) = self.defined_macros.get(expr) {
            if let Ok(num) = value.parse::<i64>() {
                return num;
            }
        }
        
        // 評価できない場合は0
        0
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
            Token::NumberLiteral(t) => t.span.clone(),
            Token::FloatLiteral(t) => t.span.clone(),
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
            Token::Plus(t) => t.span.clone(),
            Token::Minus(t) => t.span.clone(),
            Token::Slash(t) => t.span.clone(),
            Token::Percent(t) => t.span.clone(),
            Token::EqualEqual(t) => t.span.clone(),
            Token::NotEqual(t) => t.span.clone(),
            Token::LessThan(t) => t.span.clone(),
            Token::LessThanOrEqual(t) => t.span.clone(),
            Token::GreaterThan(t) => t.span.clone(),
            Token::GreaterThanOrEqual(t) => t.span.clone(),
            Token::Ampersand(t) => t.span.clone(),
            Token::AmpersandAmpersand(t) => t.span.clone(),
            Token::Pipe(t) => t.span.clone(),
            Token::PipePipe(t) => t.span.clone(),
            Token::Caret(t) => t.span.clone(),
            Token::Tilde(t) => t.span.clone(),
            Token::Exclamation(t) => t.span.clone(),
            Token::LeftShift(t) => t.span.clone(),
            Token::RightShift(t) => t.span.clone(),
            Token::LeftBracket(t) => t.span.clone(),
            Token::RightBracket(t) => t.span.clone(),
            Token::Question(t) => t.span.clone(),
            Token::Colon(t) => t.span.clone(),
            Token::Comma(t) => t.span.clone(),
            Token::Dot(t) => t.span.clone(),
            Token::Arrow(t) => t.span.clone(),
            Token::PlusPlus(t) => t.span.clone(),
            Token::MinusMinus(t) => t.span.clone(),
            Token::Return(t) => t.span.clone(),
            Token::IfKeyword(t) => t.span.clone(),
            Token::ElseKeyword(t) => t.span.clone(),
            Token::While(t) => t.span.clone(),
            Token::For(t) => t.span.clone(),
            Token::Error(t) => t.span.clone(),
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
                Token::Struct(_) => {
                    // struct [name] を解析
                    let struct_name = if let Some(Token::Ident(IdentToken { name, .. })) = self.lexer.next_token() {
                        Some(name)
                    } else {
                        None
                    };
                    base_type = Some(BaseType::Struct(struct_name));
                    break;
                }
                Token::Union(_) => {
                    // union [name] を解析
                    let union_name = if let Some(Token::Ident(IdentToken { name, .. })) = self.lexer.next_token() {
                        Some(name)
                    } else {
                        None
                    };
                    base_type = Some(BaseType::Union(union_name));
                    break;
                }
                Token::Enum(_) => {
                    // enum [name] を解析
                    let enum_name = if let Some(Token::Ident(IdentToken { name, .. })) = self.lexer.next_token() {
                        Some(name)
                    } else {
                        None
                    };
                    base_type = Some(BaseType::Enum(enum_name));
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
            // peek_token()を使用して、ポインタでない場合はトークンを消費しない
            let token = match self.lexer.peek_token() {
                Some(t) => t,
                None => break,
            };

            match token {
                Token::Asterisk(ast_token) => {
                    // *が確認できたので、ここでnext_token()で消費する
                    self.lexer.next_token();
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
                    // このトークンが識別子の場合、それは型名なので消費しないためにNoneを返す
                    // （呼び出し側で型名を取得する必要がある場合は別のメソッドを使用）
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
    
    /// typedef宣言用：型と型名（declarator）を両方パース
    /// parse_type()と違い、型の後の識別子も返す
    pub fn parse_type_and_declarator(&mut self) -> Option<(Type, String)> {
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
                Token::Struct(_) => {
                    // struct [name] を解析
                    // 次のトークンを確認（波括弧の場合は匿名struct）
                    match self.lexer.next_token() {
                        Some(Token::Ident(IdentToken { name, .. })) => {
                            base_type = Some(BaseType::Struct(Some(name)));
                            break;
                        }
                        Some(Token::LeftBrace(_)) => {
                            // 匿名struct { ... } - parse_type_and_declaratorでは処理できない
                            return None;
                        }
                        _ => {
                            // その他の場合も失敗
                            return None;
                        }
                    }
                }
                Token::Union(_) => {
                    // union [name] を解析
                    // 次のトークンを確認（波括弧の場合は匿名union）
                    match self.lexer.next_token() {
                        Some(Token::Ident(IdentToken { name, .. })) => {
                            base_type = Some(BaseType::Union(Some(name)));
                            break;
                        }
                        Some(Token::LeftBrace(_)) => {
                            // 匿名union { ... } - parse_type_and_declaratorでは処理できない
                            return None;
                        }
                        _ => {
                            // その他の場合も失敗
                            return None;
                        }
                    }
                }
                Token::Enum(_) => {
                    // enum [name] を解析
                    // 次のトークンを確認（波括弧の場合は匿名enum）
                    match self.lexer.next_token() {
                        Some(Token::Ident(IdentToken { name, .. })) => {
                            base_type = Some(BaseType::Enum(Some(name)));
                            break;
                        }
                        Some(Token::LeftBrace(_)) => {
                            // 匿名enum { ... } - parse_type_and_declaratorでは処理できない
                            return None;
                        }
                        _ => {
                            // その他の場合も失敗
                            return None;
                        }
                    }
                }
                _ => {
                    return None;
                }
            }
        }

        let base_type = base_type?;
        let mut end_span = last_span?;

        // フェーズ2: ポインタ層を解析
        let mut pointer_layers = Vec::new();
        let mut declarator_name: Option<String> = None;
        
        'pointer_loop: loop {
            let token = match self.lexer.next_token() {
                Some(t) => t,
                None => break,
            };

            match token {
                Token::Asterisk(ast_token) => {
                    let asterisk_span = ast_token.span.clone();
                    end_span = asterisk_span.clone();
                    let mut qualifiers = Vec::new();

                    'qualifier_loop: loop {
                        let next_token = match self.lexer.next_token() {
                            Some(t) => t,
                            None => {
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
                            Token::Asterisk(_) => {
                                pointer_layers.push(crate::type_system::PointerLayer::with_qualifiers(
                                    qualifiers,
                                    asterisk_span,
                                ));
                                continue 'pointer_loop;
                            }
                            Token::Ident(IdentToken { name, .. }) => {
                                pointer_layers.push(crate::type_system::PointerLayer::with_qualifiers(
                                    qualifiers,
                                    asterisk_span,
                                ));
                                declarator_name = Some(name);
                                break 'pointer_loop;
                            }
                            _ => {
                                pointer_layers.push(crate::type_system::PointerLayer::with_qualifiers(
                                    qualifiers,
                                    asterisk_span,
                                ));
                                break 'pointer_loop;
                            }
                        }
                    }
                }
                Token::Ident(IdentToken { name, .. }) => {
                    declarator_name = Some(name);
                    break;
                }
                _ => {
                    break;
                }
            }
        }

        let type_info = Type::with_pointers(
            base_type,
            base_qualifiers,
            pointer_layers,
            end_span,
        );
        
        declarator_name.map(|name| (type_info, name))
    }
    
    /// typedef宣言から型名と型情報を抽出して型テーブルに登録
    fn register_typedef_name(&mut self, typedef_text: &str) {
        // typedef宣言全体をパースして型情報と型名を取得
        // 例: "typedef int MyInt;" -> 型名: MyInt, 型情報: int
        // 例: "typedef int *IntPtr;" -> 型名: IntPtr, 型情報: int*
        // 例: "typedef struct { int x; } Point;" -> 型名: Point
        
        // typedef宣言全体を再度パース
        let typedef_lexer = Lexer::new(typedef_text);
        let mut typedef_parser = Parser::new(typedef_lexer);
        
        // typedefキーワードをスキップ
        if !matches!(typedef_parser.lexer.next_token(), Some(Token::Typedef(_))) {
            return;
        }
        
        // まず parse_type_and_declarator で試す（基本型の場合）
        if let Some((type_info, type_name)) = typedef_parser.parse_type_and_declarator() {
            self.type_table.register_type(type_name, type_info);
            return;
        }
        
        // parse_type_and_declaratorが失敗した場合（struct/union/enum/関数ポインタ/配列の場合）
        // セミコロンの直前の識別子を型名として取得
        let typedef_lexer2 = Lexer::new(typedef_text);
        let mut typedef_parser2 = Parser::new(typedef_lexer2);
        
        let mut last_ident: Option<String> = None;
        let mut brace_depth = 0; // 波括弧の深さを追跡
        let mut paren_depth = 0; // 丸括弧の深さを追跡
        let mut bracket_depth = 0; // 角括弧の深さを追跡
        
        loop {
            match typedef_parser2.lexer.next_token() {
                Some(Token::LeftBrace(_)) => {
                    brace_depth += 1;
                }
                Some(Token::RightBrace(_)) => {
                    brace_depth -= 1;
                }
                Some(Token::LeftParen(_)) => {
                    paren_depth += 1;
                }
                Some(Token::RightParen(_)) => {
                    paren_depth -= 1;
                }
                Some(Token::LeftBracket(_)) => {
                    bracket_depth += 1;
                }
                Some(Token::RightBracket(_)) => {
                    bracket_depth -= 1;
                }
                Some(Token::Ident(IdentToken { name, .. })) => {
                    // 波括弧の外の識別子のみを型名候補とする
                    // （丸括弧・角括弧内は型名の可能性あり：関数ポインタや配列）
                    if brace_depth == 0 {
                        last_ident = Some(name);
                    }
                }
                Some(Token::Semicolon(_)) => {
                    // 全ての括弧の外のセミコロンで終了
                    if brace_depth == 0 && paren_depth == 0 && bracket_depth == 0 {
                        break;
                    }
                }
                None => {
                    break;
                }
                _ => {
                    // 他のトークンは無視
                }
            }
        }
        
        // セミコロン直前の識別子を型名として登録（型情報はintをデフォルトとする）
        if let Some(type_name) = last_ident {
            // struct/union/enum/関数ポインタ/配列の場合、詳細な型情報は保持しないが、型名は登録する
            // TODO: 将来的には完全な型情報も保存する
            let dummy_span = Span {
                start_line: 0,
                start_column: 0,
                end_line: 0,
                end_column: 0,
                byte_start_idx: 0,
                byte_end_idx: 0,
            };
            self.type_table.register_type(type_name, Type::new(BaseType::Int, dummy_span));
        }
    }
    
    /// 型テーブルへの参照を取得（ExpressionParserで使用）
    pub fn get_type_table(&self) -> &TypeTable {
        &self.type_table
    }

    /// ブロック文 { ... } を解析
    /// LeftBraceトークンは既に消費されている前提
    pub fn parse_block(&mut self) -> Vec<crate::ast::Statement> {
        use crate::ast::Statement;
        
        let mut statements = Vec::new();
        self.type_table.push_scope();  // ブロック開始でスコープ追加
        
        loop {
            match self.lexer.peek_token() {
                Some(Token::RightBrace(_)) => {
                    self.lexer.next_token(); // } を消費
                    break;
                }
                None => break,
                _ => {
                    if let Some(stmt) = self.parse_statement() {
                        statements.push(stmt);
                    } else {
                        // 解析できない場合はスキップ
                        self.lexer.next_token();
                    }
                }
            }
        }
        
        self.type_table.pop_scope();  // ブロック終了でスコープ削除
        statements
    }

    /// 1つのステートメントを解析（空文、ブロック文、式文、変数宣言文、return文をサポート）
    pub fn parse_statement(&mut self) -> Option<crate::ast::Statement> {
        use crate::ast::Statement;
        
        match self.lexer.peek_token()? {
            Token::Semicolon(token) => {
                let span = token.span.clone();
                self.lexer.next_token();
                Some(Statement::Empty { span })
            }
            Token::LeftBrace(token) => {
                let start_span = token.span.clone();
                self.lexer.next_token(); // { を消費
                let statements = self.parse_block();
                Some(Statement::Block {
                    statements,
                    span: start_span,
                })
            }
            Token::Return(_) => {
                self.parse_return_statement()
            }
            Token::IfKeyword(_) => {
                self.parse_if_statement()
            }
            Token::While(_) => {
                self.parse_while_statement()
            }
            Token::For(_) => {
                self.parse_for_statement()
            }
            // 型指定子で始まる場合は変数宣言として扱う
            Token::Int(_) | Token::Float(_) | Token::Double(_) | Token::Char(_) | 
            Token::Void(_) | Token::Long(_) | Token::Short(_) | Token::Signed(_) | 
            Token::Unsigned(_) | Token::Struct(_) | Token::Union(_) | Token::Enum(_) |
            Token::Const(_) | Token::Volatile(_) => {
                self.parse_var_decl_statement()
            }
            Token::Ident(ident_token) => {
                let name = ident_token.name.clone();
                // typedefされた型名で始まる場合は変数宣言
                if self.type_table.is_type_name(&name) {
                    self.parse_var_decl_statement()
                } else {
                    // それ以外は式文として扱う
                    self.parse_expression_statement()
                }
            }
            _ => {
                // それ以外は式文として扱う
                self.parse_expression_statement()
            }
        }
    }

    /// 式文を解析（式; のパターン）
    fn parse_expression_statement(&mut self) -> Option<crate::ast::Statement> {
        use crate::ast::Statement;
        use crate::expression_parser::ExpressionParser;
        
        let mut expr_parser = ExpressionParser::new(&mut self.lexer);
        let expr = expr_parser.parse_expression();
        expr_parser.finish();  // current_tokenをLexerに戻す
        let expr = expr?;
        
        let span = expr.span().clone();
        
        // セミコロンを消費（オプション）
        if let Some(Token::Semicolon(_)) = self.lexer.peek_token() {
            self.lexer.next_token();
        }
        
        Some(Statement::Expression { expr, span })
    }

    /// return文を解析（return; または return 式;）
    fn parse_return_statement(&mut self) -> Option<crate::ast::Statement> {
        use crate::ast::Statement;
        use crate::expression_parser::ExpressionParser;
        
        // returnトークンを消費
        let return_token = self.lexer.next_token()?;
        let start_span = if let Token::Return(t) = return_token {
            t.span
        } else {
            return None;
        };
        
        // セミコロンまでの内容を式として解析
        let value = match self.lexer.peek_token() {
            Some(Token::Semicolon(_)) => None,
            _ => {
                let mut expr_parser = ExpressionParser::new(&mut self.lexer);
                let expr = expr_parser.parse_expression();
                expr_parser.finish();  // current_tokenをLexerに戻す
                expr
            }
        };
        
        // セミコロンを消費（オプション）
        if let Some(Token::Semicolon(_)) = self.lexer.peek_token() {
            self.lexer.next_token();
        }
        
        Some(Statement::Return {
            value,
            span: start_span,
        })
    }

    /// 変数宣言文を解析（型 変数名; または 型 変数名 = 式;）
    fn parse_var_decl_statement(&mut self) -> Option<crate::ast::Statement> {
        use crate::ast::Statement;
        use crate::expression_parser::ExpressionParser;
        
        let start_span = self.lexer.peek_token()?.span();
        
        // 型を解析
        let var_type = self.parse_type()?;
        
        // 変数名を取得
        let var_name = if let Some(Token::Ident(ident_token)) = self.lexer.next_token() {
            ident_token.name
        } else {
            return None;
        };
        
        // 初期化式があるか確認
        let initializer = if let Some(Token::Equal(_)) = self.lexer.peek_token() {
            self.lexer.next_token(); // = を消費
            let mut expr_parser = ExpressionParser::new(&mut self.lexer);
            let expr = expr_parser.parse_expression();
            expr_parser.finish();  // current_tokenをLexerに戻す
            expr
        } else {
            None
        };
        
        // セミコロンを消費（オプション）
        if let Some(Token::Semicolon(_)) = self.lexer.peek_token() {
            self.lexer.next_token();
        }
        
        Some(Statement::VarDecl {
            var_type: Some(var_type),
            var_name,
            initializer,
            span: start_span,
        })
    }

    /// if文を解析（if (condition) { statements } else { statements }）
    fn parse_if_statement(&mut self) -> Option<crate::ast::Statement> {
        use crate::ast::Statement;
        use crate::expression_parser::ExpressionParser;
        
        // if トークンを消費
        let start_span = if let Some(Token::IfKeyword(token)) = self.lexer.next_token() {
            token.span
        } else {
            return None;
        };
        
        // ( を期待
        if !matches!(self.lexer.next_token(), Some(Token::LeftParen(_))) {
            return None;
        }
        
        // 条件式を解析
        let mut expr_parser = ExpressionParser::new(&mut self.lexer);
        let condition = expr_parser.parse_expression();
        expr_parser.finish();  // current_tokenをLexerに戻す
        let condition = condition?;
        
        // ) を期待
        if !matches!(self.lexer.next_token(), Some(Token::RightParen(_))) {
            return None;
        }
        
        // then ブロックを解析
        let then_block = if matches!(self.lexer.peek_token(), Some(Token::LeftBrace(_))) {
            self.lexer.next_token(); // { を消費
            self.parse_block()
        } else {
            // ブロックではない場合、単一のステートメントを解析
            if let Some(stmt) = self.parse_statement() {
                vec![stmt]
            } else {
                Vec::new()
            }
        };
        
        // else 句があるかチェック
        let else_block = if matches!(self.lexer.peek_token(), Some(Token::ElseKeyword(_))) {
            self.lexer.next_token(); // else を消費
            
            if matches!(self.lexer.peek_token(), Some(Token::LeftBrace(_))) {
                self.lexer.next_token(); // { を消費
                Some(self.parse_block())
            } else {
                // ブロックではない場合、単一のステートメントを解析
                if let Some(stmt) = self.parse_statement() {
                    Some(vec![stmt])
                } else {
                    Some(Vec::new())
                }
            }
        } else {
            None
        };
        
        Some(Statement::If {
            condition,
            then_block,
            else_block,
            span: start_span,
        })
    }

    /// while文を解析（while (condition) { statements }）
    fn parse_while_statement(&mut self) -> Option<crate::ast::Statement> {
        use crate::ast::Statement;
        use crate::expression_parser::ExpressionParser;
        
        // while トークンを消費
        let start_span = if let Some(Token::While(token)) = self.lexer.next_token() {
            token.span
        } else {
            return None;
        };
        
        // ( を期待
        if !matches!(self.lexer.next_token(), Some(Token::LeftParen(_))) {
            return None;
        }
        
        // 条件式を解析
        let mut expr_parser = ExpressionParser::new(&mut self.lexer);
        let condition = expr_parser.parse_expression();
        expr_parser.finish();  // current_tokenをLexerに戻す
        let condition = condition?;
        
        // ) を期待
        if !matches!(self.lexer.next_token(), Some(Token::RightParen(_))) {
            return None;
        }
        
        // ボディブロックを解析
        let body = if matches!(self.lexer.peek_token(), Some(Token::LeftBrace(_))) {
            self.lexer.next_token(); // { を消費
            self.parse_block()
        } else {
            // ブロックではない場合、単一のステートメントを解析
            if let Some(stmt) = self.parse_statement() {
                vec![stmt]
            } else {
                Vec::new()
            }
        };
        
        Some(Statement::While {
            condition,
            body,
            span: start_span,
        })
    }

    /// for文を解析（for (init; condition; update) { statements }）
    fn parse_for_statement(&mut self) -> Option<crate::ast::Statement> {
        use crate::ast::Statement;
        use crate::expression_parser::ExpressionParser;
        
        // for トークンを消費
        let start_span = if let Some(Token::For(token)) = self.lexer.next_token() {
            token.span
        } else {
            return None;
        };
        
        // ( を期待
        if !matches!(self.lexer.next_token(), Some(Token::LeftParen(_))) {
            return None;
        }
        
        // init 部分を解析（セミコロンまで、または最初からセミコロンの場合は None）
        let init = if matches!(self.lexer.peek_token(), Some(Token::Semicolon(_))) {
            self.lexer.next_token(); // ; を消費
            None
        } else {
            // 変数宣言か式文を解析
            let stmt = self.parse_statement();
            // セミコロンが消費されていない場合は消費する
            if matches!(self.lexer.peek_token(), Some(Token::Semicolon(_))) {
                self.lexer.next_token();
            }
            stmt.map(Box::new)
        };
        
        // condition 部分を解析（セミコロンまで、または最初からセミコロンの場合は None）
        let condition = if matches!(self.lexer.peek_token(), Some(Token::Semicolon(_))) {
            self.lexer.next_token(); // ; を消費
            None
        } else {
            let mut expr_parser = ExpressionParser::new(&mut self.lexer);
            let expr = expr_parser.parse_expression();
            expr_parser.finish();  // current_tokenをLexerに戻す
            // セミコロンを消費
            if matches!(self.lexer.peek_token(), Some(Token::Semicolon(_))) {
                self.lexer.next_token();
            }
            expr
        };
        
        // update 部分を解析（) まで、または最初から ) の場合は None）
        let update = if matches!(self.lexer.peek_token(), Some(Token::RightParen(_))) {
            None
        } else {
            let mut expr_parser = ExpressionParser::new(&mut self.lexer);
            let expr = expr_parser.parse_expression();
            expr_parser.finish();  // current_tokenをLexerに戻す
            expr
        };
        
        // ) を期待
        if !matches!(self.lexer.next_token(), Some(Token::RightParen(_))) {
            return None;
        }
        
        // ボディブロックを解析
        let body = if matches!(self.lexer.peek_token(), Some(Token::LeftBrace(_))) {
            self.lexer.next_token(); // { を消費
            self.parse_block()
        } else {
            // ブロックではない場合、単一のステートメントを解析
            if let Some(stmt) = self.parse_statement() {
                vec![stmt]
            } else {
                Vec::new()
            }
        };
        
        Some(Statement::For {
            init,
            condition,
            update,
            body,
            span: start_span,
        })
    }
}

#[cfg(test)]
mod statement_tests {
    use super::*;

    #[test]
    fn test_parse_empty_statement() {
        let input = ";";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let stmt = parser.parse_statement();
        assert!(stmt.is_some());
        
        if let Some(crate::ast::Statement::Empty { .. }) = stmt {
            // Success
        } else {
            panic!("Expected Empty statement");
        }
    }

    #[test]
    fn test_parse_empty_block() {
        let input = "{}";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        // { を消費
        parser.lexer.next_token();
        let statements = parser.parse_block();
        assert_eq!(statements.len(), 0);
    }

    #[test]
    fn test_parse_block_with_empty_statements() {
        let input = "{;;;}";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        // { を消費
        parser.lexer.next_token();
        let statements = parser.parse_block();
        assert_eq!(statements.len(), 3);
        
        for stmt in statements {
            if let crate::ast::Statement::Empty { .. } = stmt {
                // Success
            } else {
                panic!("Expected Empty statement");
            }
        }
    }

    #[test]
    fn test_parse_nested_blocks() {
        let input = "{{;}}";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        // 外側の { を消費
        parser.lexer.next_token();
        let statements = parser.parse_block();
        assert_eq!(statements.len(), 1);
        
        if let crate::ast::Statement::Block { statements: inner, .. } = &statements[0] {
            assert_eq!(inner.len(), 1);
            if let crate::ast::Statement::Empty { .. } = inner[0] {
                // Success
            } else {
                panic!("Expected Empty statement in nested block");
            }
        } else {
            panic!("Expected Block statement");
        }
    }

    #[test]
    fn test_scope_management_in_blocks() {
        let input = "{}";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let initial_depth = parser.type_table.scope_depth();
        
        // { を消費
        parser.lexer.next_token();
        parser.parse_block();
        
        // ブロック終了後はスコープが元に戻っている
        assert_eq!(parser.type_table.scope_depth(), initial_depth);
    }

    #[test]
    fn test_parse_expression_statement() {
        let input = "123;";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let stmt = parser.parse_statement();
        assert!(stmt.is_some());
        
        if let Some(crate::ast::Statement::Expression { .. }) = stmt {
            // Success
        } else {
            panic!("Expected Expression statement, got {:?}", stmt);
        }
    }

    #[test]
    fn test_parse_expression_statement_with_identifier() {
        let input = "x;";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let stmt = parser.parse_statement();
        assert!(stmt.is_some());
        
        if let Some(crate::ast::Statement::Expression { .. }) = stmt {
            // Success
        } else {
            panic!("Expected Expression statement");
        }
    }

    #[test]
    fn test_parse_block_with_expression_statements() {
        let input = "{1; 2; 3;}";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        // { を消費
        parser.lexer.next_token();
        let statements = parser.parse_block();
        assert_eq!(statements.len(), 3);
        
        for stmt in statements {
            if let crate::ast::Statement::Expression { .. } = stmt {
                // Success
            } else {
                panic!("Expected Expression statement");
            }
        }
    }

    #[test]
    fn test_parse_var_decl_simple() {
        let input = "int x;";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let stmt = parser.parse_statement();
        assert!(stmt.is_some());
        
        if let Some(crate::ast::Statement::VarDecl { var_name, initializer, .. }) = stmt {
            assert_eq!(var_name, "x");
            assert!(initializer.is_none());
        } else {
            panic!("Expected VarDecl statement");
        }
    }

    #[test]
    fn test_parse_var_decl_with_initializer() {
        let input = "int x = 42;";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let stmt = parser.parse_statement();
        assert!(stmt.is_some());
        
        if let Some(crate::ast::Statement::VarDecl { var_name, initializer, .. }) = stmt {
            assert_eq!(var_name, "x");
            assert!(initializer.is_some());
        } else {
            panic!("Expected VarDecl statement");
        }
    }

    #[test]
    fn test_parse_block_with_var_decls() {
        let input = "{int x; int y = 10;}";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        // { を消費
        parser.lexer.next_token();
        let statements = parser.parse_block();
        assert_eq!(statements.len(), 2);
        
        for stmt in statements {
            if let crate::ast::Statement::VarDecl { .. } = stmt {
                // Success
            } else {
                panic!("Expected VarDecl statement");
            }
        }
    }

    #[test]
    fn test_parse_mixed_statements() {
        let input = "{int x; x; int y = 5; y;}";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        // { を消費
        parser.lexer.next_token();
        let statements = parser.parse_block();
        assert_eq!(statements.len(), 4);
        
        // 順番: VarDecl, Expression, VarDecl, Expression
        match &statements[0] {
            crate::ast::Statement::VarDecl { .. } => {},
            _ => panic!("Expected VarDecl at position 0"),
        }
        match &statements[1] {
            crate::ast::Statement::Expression { .. } => {},
            _ => panic!("Expected Expression at position 1"),
        }
        match &statements[2] {
            crate::ast::Statement::VarDecl { .. } => {},
            _ => panic!("Expected VarDecl at position 2"),
        }
        match &statements[3] {
            crate::ast::Statement::Expression { .. } => {},
            _ => panic!("Expected Expression at position 3"),
        }
    }

    #[test]
    fn test_parse_return_statement_empty() {
        let input = "return;";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let stmt = parser.parse_statement();
        assert!(stmt.is_some());
        
        if let Some(crate::ast::Statement::Return { value, .. }) = stmt {
            assert!(value.is_none());
        } else {
            panic!("Expected Return statement");
        }
    }

    #[test]
    fn test_parse_return_statement_with_value() {
        let input = "return 42;";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let stmt = parser.parse_statement();
        assert!(stmt.is_some());
        
        if let Some(crate::ast::Statement::Return { value, .. }) = stmt {
            assert!(value.is_some());
        } else {
            panic!("Expected Return statement");
        }
    }

    #[test]
    fn test_parse_block_with_return() {
        let input = "{int x = 5; return x;}";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        // { を消費
        parser.lexer.next_token();
        let statements = parser.parse_block();
        assert_eq!(statements.len(), 2);
        
        match &statements[0] {
            crate::ast::Statement::VarDecl { .. } => {},
            _ => panic!("Expected VarDecl at position 0"),
        }
        match &statements[1] {
            crate::ast::Statement::Return { .. } => {},
            _ => panic!("Expected Return at position 1"),
        }
    }

    #[test]
    fn test_parse_function_with_empty_body() {
        let input = "int main() {}";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let tu = parser.parse();
        assert_eq!(tu.items.len(), 1);
        
        if let crate::ast::Item::FunctionDecl { function_name, body, .. } = &tu.items[0] {
            assert_eq!(function_name, "main");
            assert!(body.is_some());
            assert_eq!(body.as_ref().unwrap().len(), 0);
        } else {
            panic!("Expected FunctionDecl");
        }
    }

    #[test]
    fn test_parse_function_with_statements() {
        let input = "int add(int a, int b) { int result; return result; }";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let tu = parser.parse();
        assert_eq!(tu.items.len(), 1);
        
        if let crate::ast::Item::FunctionDecl { function_name, body, .. } = &tu.items[0] {
            assert_eq!(function_name, "add");
            assert!(body.is_some());
            let statements = body.as_ref().unwrap();
            assert_eq!(statements.len(), 2); // VarDecl, Return
        } else {
            panic!("Expected FunctionDecl");
        }
    }

    #[test]
    fn test_parse_function_with_nested_scopes() {
        let input = "void test() { int x; { int y; } }";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let tu = parser.parse();
        assert_eq!(tu.items.len(), 1);
        
        if let crate::ast::Item::FunctionDecl { body, .. } = &tu.items[0] {
            assert!(body.is_some());
            let statements = body.as_ref().unwrap();
            assert_eq!(statements.len(), 2); // VarDecl, Block
            
            // 2番目がネストされたブロック
            match &statements[1] {
                crate::ast::Statement::Block { statements: nested, .. } => {
                    assert_eq!(nested.len(), 1); // int y;
                },
                _ => panic!("Expected Block statement"),
            }
        } else {
            panic!("Expected FunctionDecl");
        }
    }

    #[test]
    fn test_function_declaration_without_body() {
        let input = "int add(int a, int b);";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let tu = parser.parse();
        assert_eq!(tu.items.len(), 1);
        
        if let crate::ast::Item::FunctionDecl { function_name, body, .. } = &tu.items[0] {
            assert_eq!(function_name, "add");
            assert!(body.is_none()); // 宣言のみなのでbodyなし
        } else {
            panic!("Expected FunctionDecl");
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let input = "if (x) { return 1; }";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let stmt = parser.parse_statement();
        assert!(stmt.is_some());
        
        if let Some(crate::ast::Statement::If { then_block, else_block, .. }) = stmt {
            assert_eq!(then_block.len(), 1);
            assert!(else_block.is_none());
        } else {
            panic!("Expected If statement");
        }
    }

    #[test]
    fn test_parse_if_else_statement() {
        let input = "if (x) { return 1; } else { return 0; }";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let stmt = parser.parse_statement();
        assert!(stmt.is_some());
        
        if let Some(crate::ast::Statement::If { then_block, else_block, .. }) = stmt {
            assert_eq!(then_block.len(), 1);
            assert!(else_block.is_some());
            assert_eq!(else_block.unwrap().len(), 1);
        } else {
            panic!("Expected If statement");
        }
    }

    #[test]
    fn test_parse_while_statement() {
        let input = "while (x) { return 1; }";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let stmt = parser.parse_statement();
        assert!(stmt.is_some());
        
        if let Some(crate::ast::Statement::While { body, .. }) = stmt {
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected While statement");
        }
    }

    #[test]
    fn test_parse_for_statement_full() {
        let input = "for (int i = 0; i; i) { return i; }";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let stmt = parser.parse_statement();
        assert!(stmt.is_some());
        
        if let Some(crate::ast::Statement::For { init, condition, update, body, .. }) = stmt {
            assert!(init.is_some());
            assert!(condition.is_some());
            assert!(update.is_some());
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected For statement");
        }
    }

    #[test]
    fn test_parse_for_statement_minimal() {
        let input = "for (;;) { return 0; }";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let stmt = parser.parse_statement();
        assert!(stmt.is_some());
        
        if let Some(crate::ast::Statement::For { init, condition, update, body, .. }) = stmt {
            assert!(init.is_none());
            assert!(condition.is_none());
            assert!(update.is_none());
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected For statement");
        }
    }

    #[test]
    fn test_parse_nested_control_flow() {
        let input = "if (x) { while (y) { return 1; } }";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let stmt = parser.parse_statement();
        assert!(stmt.is_some());
        
        if let Some(crate::ast::Statement::If { then_block, .. }) = stmt {
            assert_eq!(then_block.len(), 1);
            if let crate::ast::Statement::While { body, .. } = &then_block[0] {
                assert_eq!(body.len(), 1);
            } else {
                panic!("Expected While inside If");
            }
        } else {
            panic!("Expected If statement");
        }
    }

    #[test]
    fn test_parse_function_with_control_flow() {
        let input = "int test() { if (x) return 1; else return 0; }";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let tu = parser.parse();
        assert_eq!(tu.items.len(), 1);
        
        if let crate::ast::Item::FunctionDecl { body, .. } = &tu.items[0] {
            assert!(body.is_some());
            let statements = body.as_ref().unwrap();
            assert_eq!(statements.len(), 1);
            
            if let crate::ast::Statement::If { .. } = &statements[0] {
                // OK
            } else {
                panic!("Expected If statement in function body");
            }
        } else {
            panic!("Expected FunctionDecl");
        }
    }

    #[test]
    fn test_ifdef_evaluation_true() {
        let input = "#define DEBUG\n#ifdef DEBUG\nint x;\n#endif";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let tu = parser.parse();
        assert_eq!(tu.items.len(), 2); // #define と #ifdef ブロック
        
        if let crate::ast::Item::ConditionalBlock { condition_result, .. } = &tu.items[1] {
            assert_eq!(*condition_result, true); // DEBUG is defined
        } else {
            panic!("Expected ConditionalBlock");
        }
    }

    #[test]
    fn test_ifdef_evaluation_false() {
        let input = "#ifdef UNDEFINED\nint x;\n#endif";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let tu = parser.parse();
        assert_eq!(tu.items.len(), 1); // #ifdef ブロックのみ
        
        if let crate::ast::Item::ConditionalBlock { condition_result, .. } = &tu.items[0] {
            assert_eq!(*condition_result, false); // UNDEFINED is not defined
        } else {
            panic!("Expected ConditionalBlock");
        }
    }

    #[test]
    fn test_ifndef_evaluation() {
        let input = "#ifndef UNDEFINED\nint x;\n#endif";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let tu = parser.parse();
        assert_eq!(tu.items.len(), 1);
        
        if let crate::ast::Item::ConditionalBlock { condition_result, .. } = &tu.items[0] {
            assert_eq!(*condition_result, true); // UNDEFINED is not defined
        } else {
            panic!("Expected ConditionalBlock");
        }
    }

    #[test]
    fn test_if_defined_evaluation() {
        let input = "#define FLAG\n#if defined(FLAG)\nint x;\n#endif";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let tu = parser.parse();
        assert_eq!(tu.items.len(), 2);
        
        if let crate::ast::Item::ConditionalBlock { condition_result, .. } = &tu.items[1] {
            assert_eq!(*condition_result, true);
        } else {
            panic!("Expected ConditionalBlock");
        }
    }

    #[test]
    fn test_if_numeric_evaluation() {
        let input = "#define VALUE 1\n#if VALUE\nint x;\n#endif";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let tu = parser.parse();
        assert_eq!(tu.items.len(), 2);
        
        if let crate::ast::Item::ConditionalBlock { condition_result, .. } = &tu.items[1] {
            assert_eq!(*condition_result, true); // VALUE == 1, non-zero is true
        } else {
            panic!("Expected ConditionalBlock");
        }
    }

    #[test]
    fn test_if_comparison_evaluation() {
        let input = "#define VERSION 2\n#if VERSION >= 2\nint x;\n#endif";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let tu = parser.parse();
        assert_eq!(tu.items.len(), 2);
        
        if let crate::ast::Item::ConditionalBlock { condition_result, .. } = &tu.items[1] {
            assert_eq!(*condition_result, true); // VERSION >= 2 is true
        } else {
            panic!("Expected ConditionalBlock");
        }
    }

    #[test]
    fn test_if_logical_and_evaluation() {
        let input = "#define A 1\n#define B 1\n#if defined(A) && defined(B)\nint x;\n#endif";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let tu = parser.parse();
        assert_eq!(tu.items.len(), 3); // 2 defines + 1 conditional block
        
        if let crate::ast::Item::ConditionalBlock { condition_result, .. } = &tu.items[2] {
            assert_eq!(*condition_result, true); // both defined
        } else {
            panic!("Expected ConditionalBlock");
        }
    }
}

