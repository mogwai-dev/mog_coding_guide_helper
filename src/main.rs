use std::{fs, str::CharIndices};


enum Token<'a> {
    BlockComment{
        span: Span,
    },
    Include{
        span: Span,
        filename: String,        // ファイル名 あとで &str にしたほうがメモリ効率がいいんじゃない？
    },
    Define{
        span: Span,
        macro_name: String,      // マクロ名 あとで &str にしたほうがメモリ効率がいいんじゃない？
        macro_value: String      // マクロ値 あとで &str にしたほうがメモリ効率がいいんじゃない？
    },
    Semicolon{
        span: Span,
    },
    Equal{
        span: Span,
    },
    Ident{
        span: Span,
        name: &'a str,
    },
    // 記憶域クラス指定子
    Auto{               // C 言語ではスタックに保存するという意味の記憶域クラス指定子がある。実際に使われることはないそう。
        span: Span,
    },
    Register{           // C 言語では汎用レジスタに保存するという意味の記憶域クラス指定子がある。コンパイラは無視することがあるそう。
        span: Span,
    },
    Static{
        span: Span,     // データセグメントに配置。プログラム開始から終了まで存在。
    },
    Extern{
        span: Span,     // 他のファイルに定義されていることを示す。
    },
    Typedef{
        span: Span,
    },
    // 型修飾子
    Const{
        span: Span,
    },
    Volatile{
        span: Span,
    },
    Restrict{           // todo: C99 以降のキーワードであることを警告して使わせないようにする
        span: Span,
    },
    _Atomic{            // todo: C11 以降のキーワードであることを明示する
        span: Span,
    },
    // 型指定子
    Int { span: Span },
    Char { span: Span },
    Float { span: Span },
    Double { span: Span },
    Void { span: Span },
    Long { span: Span },
    Short { span: Span },
    Signed { span: Span },
    Unsigned { span: Span },
    // 構造体関連
    Struct { span: Span },
    LeftBrace { span: Span },    // {
    RightBrace { span: Span },   // }
}

#[derive(Debug)]
struct Lexer<'a> {
    input: &'a str,
    char_offsets: CharIndices<'a>, // 文字のバイトオフセットと文字のイテレータ
    cur: usize,               // 次に読む文字のインデックス (0..=len)
    column: usize,
    line: usize,
    now: Option<(usize, char)>,
    peeked: Option<(usize, char)>,
}

// Lexer の実装
// 文字列をトークンに分ける
impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        let mut lx = Lexer {
            input,
            char_offsets: input.char_indices(),
            cur: 0,
            column: 0,
            line: 0,
            now: None,
            peeked: None,
        };
        lx.next_char(); // 初期化のために一文字進める
        lx
    }

    // 記号ではないキーワードはここで処理する
    fn keyword_to_token(&self, byte_idx_start: usize, byte_idx_end: usize, span: Span) -> Option<Token> {
        match &self.input[byte_idx_start..byte_idx_end] {
            "auto" => Some(Token::Auto { span }),
            "register" => Some(Token::Register { span }),
            "static" => Some(Token::Static { span }),
            "extern" => Some(Token::Extern { span }),
            "typedef" => Some(Token::Typedef { span }),
            "const" => Some(Token::Const { span }),
            "volatile" => Some(Token::Volatile { span }),
            "restrict" => Some(Token::Restrict { span }),
            "_Atomic" => Some(Token::_Atomic { span }),
            "int" => Some(Token::Int { span }),
            "char" => Some(Token::Char { span }),
            "float" => Some(Token::Float { span }),
            "double" => Some(Token::Double { span }),
            "void" => Some(Token::Void { span }),
            "long" => Some(Token::Long { span }),
            "short" => Some(Token::Short { span }),
            "signed" => Some(Token::Signed { span }),
            "unsigned" => Some(Token::Unsigned { span }),
            "struct" => Some(Token::Struct { span }),
            _ => Some(Token::Ident {
                span,
                name: &self.input[byte_idx_start..byte_idx_end],
            }),
        }
    }

    // 先に進めて文字を返す（存在しなければ None）
    fn next_char(&mut self) -> Option<(usize, char)> {

        self.now = self.peeked;
        self.peeked = self.char_offsets.next();

        if let Some((_, ch)) = self.now {
            self.cur += 1;
            if ch == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }
            return self.now;
        }
        None
    }

    // 次に読む文字を参照する（位置を変えない）
    fn peek(&self) -> Option<(usize, char)> {
        self.peeked
    }

    fn pos_index(&self) -> usize {
        self.cur
    }

    // トークンを一つ読み取る
    fn next_token(&mut self) -> Option<Token> {
        let start_line = self.line;
        let start_column = self.column;
        let mut start_byte_flag: Option<usize> = None;

        loop {
            match self.next_char() {
                Some((byte_idx, ' ')) | Some((byte_idx, '\t')) | Some((byte_idx, '\n')) | Some((byte_idx, '\r')) => {
                    // 空白文字はスキップ
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }

                    continue;
                },
                Some((byte_idx, ';')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };

                    let end_line = self.line;
                    let end_column = self.column;

                    return Some(Token::Semicolon {
                        span: Span {
                            start_line,
                            start_column,
                            end_line,
                            end_column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    });
                },
                Some((byte_idx, '=')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };

                    let end_line = self.line;
                    let end_column = self.column;

                    return Some(Token::Equal {
                        span: Span {
                            start_line,
                            start_column,
                            end_line,
                            end_column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    });
                },
                Some((byte_idx, '{')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };

                    return Some(Token::LeftBrace {
                        span: Span {
                            start_line,
                            start_column,
                            end_line: self.line,
                            end_column: self.column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    });
                },
                Some((byte_idx, '}')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };

                    return Some(Token::RightBrace {
                        span: Span {
                            start_line,
                            start_column,
                            end_line: self.line,
                            end_column: self.column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    });
                },
                Some((byte_idx, '/')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    if let Some((_, '*')) = self.peek() {
                        // ブロックコメントの開始
                        // '*' を消費
                        self.next_char();
    
                        // コメントの終わりを探す
                        while let Some((_, ch)) = self.next_char() {
                            if ch == '*' {
                                if let Some((_, '/')) = self.peek() {
                                    // '/' を消費してコメントを終える
                                    self.next_char();

                                    // end_byte: 次に来る文字のバイト開始位置（peeked の byte idx）か入力終端
                                    let end_byte = if let Some((b, _)) = self.peeked {
                                        b
                                    } else {
                                        self.input.len()
                                    };

                                    let end_line = self.line;
                                    let end_column = self.column;

                                    return Some(Token::BlockComment {
                                        span: Span {
                                            start_line,
                                            start_column,
                                            end_line,
                                            end_column,
                                            byte_start_idx: start_byte_flag.unwrap(),
                                            byte_end_idx: end_byte,
                                        }
                                    });
                                }
                            }
                        }
                        
                        // コメントが閉じられなかった場合は None を返す
                        return None;
                    } else {
                        // 他のトークン処理へ（ここでは省略）
                        return None
                    }
                },
                Some((byte_idx, '#')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }

                    // ディレクティブを行末まで読み取る（先頭の空白は start_char_idx でカバーされる）
                    let mut directive_text = String::new();
                    directive_text.push('#');

                    while let Some((_, ch)) = self.next_char() {
                        directive_text.push(ch);
                        if ch == '\n' {
                            break;
                        }
                    }

                    // directive の中身（# を取り除いた後）を解析用に取得（先頭空白は trim_start する）
                    let content = directive_text.trim_start_matches('#').trim_start().to_string();

                    // バイトオフセットを計算：次に来る文字のバイト位置（peeked の byte idx）か入力終端
                    let end_byte_idx = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };
                    let end_line = self.line;
                    let end_column = self.column;

                    // #include の処理（既存の挙動を保持）
                    if let Some(rest) = content.strip_prefix("include") {
                        let rest = rest.trim();
                        let filename;
                        if rest.starts_with('<') {
                            if let Some(end) = rest.find('>') {
                                if end > 1 {
                                    filename = rest[1..end].to_string();
                                } else {
                                    filename = "".to_string();
                                }
                            } else {
                                filename = rest[1..].to_string();
                            }
                        } else if rest.starts_with('"') {
                            if rest.len() >= 2 && rest.ends_with('"') {
                                filename = rest[1..rest.len()-1].to_string();
                            } else {
                                let mut acc = String::new();
                                for c in rest.chars().skip(1) {
                                    if c == '"' { break; }
                                    acc.push(c);
                                }
                                filename = acc;
                            }
                        } else {
                            filename = rest.to_string();
                        }

                        return Some(Token::Include {
                            span: Span {
                                start_line,
                                start_column,
                                end_line,
                                end_column,
                                byte_start_idx: start_byte_flag.unwrap(),
                                byte_end_idx: end_byte_idx,
                            },
                            filename: filename.to_string(),
                        });
                    }

                    // #define の処理：先頭の空白は token の offset/length に含まれる（start_byte がそれを指す）
                    if let Some(rest) = content.strip_prefix("define") {
                        let rest = rest.trim();
                        let mut parts = rest.splitn(2, ' ');
                        if let (Some(name), Some(value)) = (parts.next(), parts.next()) {
                            let macro_name = name.to_string();
                            let macro_value = value.to_string();

                            return Some(Token::Define {
                                span: Span {
                                    start_line,
                                    start_column,
                                    end_line,
                                    end_column,
                                    byte_start_idx: start_byte_flag.unwrap(),
                                    byte_end_idx: end_byte_idx,
                                },
                                macro_name,
                                macro_value,
                            });
                        }
                    }

                    // それ以外の # 系ディレクティブはとりあえず Include 風に生テキストを残す（既存互換）
                    return Some(Token::Include {
                        span: Span {
                            start_line,
                            start_column,
                            end_line,
                            end_column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte_idx,
                        },
                        filename: content.to_string(),
                    });
                },
                Some((byte_idx, ch)) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }

                    // キーワードか識別子の判定
                    if self.is_identifier_start(ch) {
                        let (actual_byte_idx_start, _) = self.now.unwrap();

                        // 識別子の終わりまで読み進める
                        while let Some((_, next_ch)) = self.peek() {
                            if self.is_identifier_start(next_ch) || next_ch.is_ascii_digit() {
                                self.next_char();
                            } else {
                                break;
                            }
                        }
                        
                        // 最後に読んだ文字の次のバイト位置を終端とする
                        let actual_byte_idx_end = if let Some((b, _)) = self.peeked {
                            b
                        } else {
                            self.input.len()
                        };

                        let byte_start_idx = start_byte_flag.unwrap();
                        let byte_end_idx = actual_byte_idx_end;
                        let span = Span {
                            start_line,
                            start_column,
                            end_line: self.line,
                            end_column: self.column,
                            byte_start_idx,
                            byte_end_idx,
                        };
                        
                        return self.keyword_to_token(actual_byte_idx_start, actual_byte_idx_end, span);
                    }
                }, 
                None => return None, // 入力の終わり
            }
        }
    }

    fn is_identifier_start(&self, ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_'
    }
}


#[derive(Debug)]
pub struct TranslationUnit {
    items: Vec<Item>,
}

#[derive(Debug)]
enum Item {
    BlockComment { span: Span, text: String },
    Include { span: Span, text: String, filename: String },
    Define { span: Span, text: String, macro_name: String, macro_value: String },
    TypedefDecl { span: Span, text: String },
    VarDecl { 
        span: Span, 
        text: String,
        var_name: String,
        has_initializer: bool,
    },
    StructDecl {
        span: Span,
        text: String,
        struct_name: Option<String>,  // 無名構造体の場合は None
        has_typedef: bool,            // typedef struct の場合 true
    },
}

// ルートとノードを定義。所有する Span を持たせる（ライフタイム回避のため String/span を所有）
#[derive(Debug, Clone)]
pub struct Span {
    start_line: usize,
    start_column: usize,
    end_line: usize,
    end_column: usize,
    byte_start_idx: usize, // オリジナルの文字列におけるバイトオフセット
    byte_end_idx: usize,   // オリジナルの文字列におけるバイトオフセットの終端
}


#[derive(Debug)]
struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    fn new(lexer: Lexer<'a>) -> Self {
        Parser { lexer }
    }

    fn parse(&mut self) -> TranslationUnit {
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

#[derive(Debug)]
pub struct Formatter {

}

impl Formatter {
    pub fn new() -> Self {
        Formatter {

        }
    }

    pub fn format_tu(&self, tu: &TranslationUnit) -> String {
        let mut s = String::new();
        for item in &tu.items {
            match item {
                Item::BlockComment { text, .. } => {
                    // 先頭の空白系文字列を見つける（スペース/タブ/CR/LF を含む）
                    let first_non_ws = text
                        .char_indices()
                        .find(|&(_, ch)| !ch.is_whitespace())
                        .map(|(i, _)| i)
                        .unwrap_or(text.len());

                    // 先頭の空白部分から改行だけ取り出して保持する
                    let leading = &text[..first_non_ws];
                    let kept_newlines: String = leading.chars().filter(|&c| c == '\n').collect();

                    // 改行を先頭に残し、それ以外の先頭空白は削除して残りを追加
                    s.push_str(&kept_newlines);
                    s.push_str(&text[first_non_ws..]);
                },
                Item::Include { text, ..} => {
                    // 先頭の空白系文字列を見つける（スペース/タブ/CR/LF を含む）
                    let first_non_ws = text
                        .char_indices()
                        .find(|&(_, ch)| !ch.is_whitespace())
                        .map(|(i, _)| i)
                        .unwrap_or(text.len());

                    // 先頭の空白部分から改行だけ取り出して保持する
                    let leading = &text[..first_non_ws];
                    let kept_newlines: String = leading.chars().filter(|&c| c == '\n').collect();

                    // 改行を先頭に残し、それ以外の先頭空白は削除して残りを追加
                    s.push_str(&kept_newlines);
                    s.push_str(&text[first_non_ws..]);       
                },
                Item::Define { text, ..} => {
                    // 先頭の空白系文字列を見つける（スペース/タブ/CR/LF を含む）
                    let first_non_ws = text
                        .char_indices()
                        .find(|&(_, ch)| !ch.is_whitespace())
                        .map(|(i, _)| i)
                        .unwrap_or(text.len());

                    // 先頭の空白部分から改行だけ取り出して保持する
                    let leading = &text[..first_non_ws];
                    let kept_newlines: String = leading.chars().filter(|&c| c == '\n').collect();

                    // 改行を先頭に残し、それ以外の先頭空白は削除して残りを追加
                    s.push_str(&kept_newlines);
                    s.push_str(&text[first_non_ws..]);      
                },
                Item::TypedefDecl { text, .. } => {
                    // 先頭の空白系文字列を見つける（スペース/タブ/CR/LF を含む）
                    let first_non_ws = text
                        .char_indices()
                        .find(|&(_, ch)| !ch.is_whitespace())
                        .map(|(i, _)| i)
                        .unwrap_or(text.len());

                    // 先頭の空白部分から改行だけ取り出して保持する
                    let leading = &text[..first_non_ws];
                    let kept_newlines: String = leading.chars().filter(|&c| c == '\n').collect();

                    // 改行を先頭に残し、それ以外の先頭空白は削除して残りを追加
                    s.push_str(&kept_newlines);
                    s.push_str(&text[first_non_ws..]);
                },
                Item::VarDecl { text, .. } => {
                    // 先頭の空白系文字列を見つける（スペース/タブ/CR/LF を含む）
                    let first_non_ws = text
                        .char_indices()
                        .find(|&(_, ch)| !ch.is_whitespace())
                        .map(|(i, _)| i)
                        .unwrap_or(text.len());

                    // 先頭の空白部分から改行だけ取り出して保持する
                    let leading = &text[..first_non_ws];
                    let kept_newlines: String = leading.chars().filter(|&c| c == '\n').collect();

                    // 改行を先頭に残し、それ以外の先頭空白は削除して残りを追加
                    s.push_str(&kept_newlines);
                    s.push_str(&text[first_non_ws..]);
                },
                Item::StructDecl { text, .. } => {
                    // 先頭の空白系文字列を見つける（スペース/タブ/CR/LF を含む）
                    let first_non_ws = text
                        .char_indices()
                        .find(|&(_, ch)| !ch.is_whitespace())
                        .map(|(i, _)| i)
                        .unwrap_or(text.len());

                    // 先頭の空白部分から改行だけ取り出して保持する
                    let leading = &text[..first_non_ws];
                    let kept_newlines: String = leading.chars().filter(|&c| c == '\n').collect();

                    // 改行を先頭に残し、それ以外の先頭空白は削除して残りを追加
                    s.push_str(&kept_newlines);
                    s.push_str(&text[first_non_ws..]);
                }
            }
        }
        s
    }

    // AST から元のコードを再構築
    pub fn original_tu(&self, tu: &TranslationUnit) -> String {
        // 元のコードを再構築するロジックをここに実装
        let mut s = String::new();
        for item in &tu.items {
            match item {
                Item::BlockComment { text, .. } | Item::Include { text, .. } | 
                Item::Define { text, .. } | Item::TypedefDecl { text, .. } |
                Item::VarDecl { text, .. } | Item::StructDecl { text, .. } => {
                    s.push_str(text);
                }
            }
        }

        s
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_iteration_and_pos() {
        let s = "ab\nc";
        let mut lx = Lexer::new(s);

        let (_, ch) = lx.peek().unwrap();
        assert_eq!(ch, 'a');
        // assert_eq!(s[byte_idx], 'a');

        // read 'a'
        assert_eq!(lx.next_char(), Some((0, 'a')));
        assert_eq!(lx.pos_index(), 1);
        assert_eq!(lx.line, 0);
        assert_eq!(lx.column, 1);

        // read 'b'
        assert_eq!(lx.next_char(), Some((1, 'b')));
        assert_eq!(lx.pos_index(), 2);
        assert_eq!(lx.line, 0);
        assert_eq!(lx.column, 2);

        // read '\n'
        assert_eq!(lx.next_char(), Some((2, '\n')));
        assert_eq!(lx.pos_index(), 3);
        assert_eq!(lx.line, 1);
        assert_eq!(lx.column, 0);

        // read 'c'
        assert_eq!(lx.next_char(), Some((3, 'c')));
        assert_eq!(lx.pos_index(), 4);
        assert_eq!(lx.line, 1);
        assert_eq!(lx.column, 1);

    }

    #[test]
    fn test_multibyte_chars() {
        // 'é' is multibyte in UTF-8
        let s = "aéb";
        let mut lx = Lexer::new(s);

        let mut got = Vec::new();
        while let Some((_, ch)) = lx.next_char() {
            got.push(ch);
        }
        assert_eq!(got, vec!['a', 'é', 'b']);
        assert_eq!(lx.pos_index(), 3);
    }

    
    #[test]
    fn test_lexer_block_comment() {
        let s = "/* comment */";
        let mut lx = Lexer::new(s);

        // Skip to the block comment
        while let Some(token) = lx.next_token() {
            match token {
                Token::BlockComment { span } => {
                    assert_eq!(span.start_line, 0);
                    assert_eq!(span.start_column, 0);
                    assert_eq!(span.end_line, 0);
                    assert_eq!(span.end_column, 13);
                    assert_eq!(span.byte_start_idx, 0);
                    assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "/* comment */");
                    return;
                },
                _ => {
                    panic!("Unexpected token");
                }
            }
        }
        panic!("Block comment token not found");
    }

    #[test]
    fn test_lexer_block_comment_japanese() {
        let s = "/* コメント */";
        let mut lx = Lexer::new(s);

        // Skip to the block comment
        while let Some(token) = lx.next_token() {
            match token {
                Token::BlockComment { span } => {
                    assert_eq!(span.start_line, 0);
                    assert_eq!(span.start_column, 0);
                    assert_eq!(span.end_line, 0);
                    assert_eq!(span.end_column, 10);
                    assert_eq!(span.byte_start_idx, 0);
                    assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "/* コメント */");
                    return;
                }
                _ => {
                    panic!("Unexpected token");
                }
            }
        }
        panic!("Block comment token not found");
    }

    #[test]
    fn test_lexer_block_comment_japanese_with_spaces() {
        let s = "\t\r\n /* コメント*/";
        let mut lx = Lexer::new(s);

        // Skip to the block comment
        while let Some(token) = lx.next_token() {
            match token {
                Token::BlockComment { span } => {
                    assert_eq!(span.start_line, 0);
                    assert_eq!(span.start_column, 0);
                    assert_eq!(span.end_line, 1);
                    assert_eq!(span.end_column, 10);
                    assert_eq!(span.byte_start_idx, 0);
                    assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "\t\r\n /* コメント*/");
                    return;
                },
                _ => {
                    panic!("Unexpected token");
                }
            }
        }
        panic!("Block comment token not found");
    }

    #[test]
    fn test_formatter_format_tu_trims_leading_whitespace() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        let item = Item::BlockComment { span, text: String::from("   /* hello */") };
        let tu = TranslationUnit { items: vec![item] };
        let fmt = Formatter::new();
        let out = fmt.format_tu(&tu);
        assert_eq!(out, "/* hello */");
    }

    #[test]
    fn test_formatter_original_tu_preserves_texts() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        let item1 = Item::BlockComment { span: span.clone(), text: String::from("/* one */") };
        let item2 = Item::BlockComment { span, text: String::from("/* two */") };
        let tu = TranslationUnit { items: vec![item1, item2] };
        let fmt = Formatter::new();
        let out = fmt.original_tu(&tu);
        assert_eq!(out, "/* one *//* two */");
    }

    #[test]
    fn test_formatter_keeps_newline_in_leading_whitespace() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        let item = Item::BlockComment { span, text: String::from("\t\r\n /* hello */") };
        let tu = TranslationUnit { items: vec![item] };
        let fmt = Formatter::new();
        let out = fmt.format_tu(&tu);
        assert_eq!(out, "\n/* hello */");
    }

    #[test]
    fn test_formatter_keeps_multiple_newlines() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        let item = Item::BlockComment { span, text: String::from("\n\n  /* ok */") };
        let tu = TranslationUnit { items: vec![item] };
        let fmt = Formatter::new();
        let out = fmt.format_tu(&tu);
        assert_eq!(out, "\n\n/* ok */");
    }

    #[test]
    fn test_lexer_include_angle() {
        let s = "#include <stdio.h>\n";
        let mut lx = Lexer::new(s);

        while let Some(token) = lx.next_token() {
            match token {
                Token::Include { span, filename } => {
                    assert_eq!(span.start_line, 0);
                    assert_eq!(span.start_column, 0);
                    assert_eq!(span.end_line, 1);
                    assert_eq!(span.end_column, 0);
                    assert_eq!(filename, "stdio.h");
                    assert_eq!(span.byte_start_idx, 0);
                    assert_eq!(span.byte_end_idx, s.len());
                    assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "#include <stdio.h>\n");
                    return;
                }
                _ => {}
            }
        }
        panic!("Include token not found");
    }

    #[test]
    fn test_lexer_include_quote() {
        let s = "#include \"file.h\"\n";
        let mut lx = Lexer::new(s);

        while let Some(token) = lx.next_token() {
            match token {
                Token::Include { filename, span } => {
                    assert_eq!(filename, "file.h");
                    assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "#include \"file.h\"\n");
                    return;
                }
                _ => {}
            }
        }
        panic!("Include token not found");
    }

    #[test]
    fn test_lexer_include_missing_closer() {
        // missing closing '>' or '"', lexer should take rest as filename
        let s1 = "#include <path/to/file\n";
        let mut lx1 = Lexer::new(s1);
        while let Some(token) = lx1.next_token() {
            match token {
                Token::Include { filename, span } => {
                    assert_eq!(filename, "path/to/file");
                    assert_eq!(&s1[span.byte_start_idx..span.byte_end_idx], "#include <path/to/file\n");
                    break;
                }
                _ => {}
            }
        }

        let s2 = "#include \"another/path\n";
        let mut lx2 = Lexer::new(s2);
        while let Some(token) = lx2.next_token() {
            match token {
                Token::Include { filename, span } => {
                    assert_eq!(filename, "another/path");
                    assert_eq!(&s2[span.byte_start_idx..span.byte_end_idx], "#include \"another/path\n");
                    break;
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_parser_includes_produced_items() {
        let s = "#include \"a.h\"\n#include <b.h>\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 2);

        match &tu.items[0] {
            Item::Include { text, filename, .. } => {
                assert_eq!(filename, "a.h");
                assert_eq!(text, "#include \"a.h\"\n");
            }
            _ => panic!("first item is not Include"),
        }

        match &tu.items[1] {
            Item::Include { text, filename, .. } => {
                assert_eq!(filename, "b.h");
                assert_eq!(text, "#include <b.h>\n");
            }
            _ => panic!("second item is not Include"),
        }
    }

    #[test]
    fn test_lexer_define_simple() {
        let s = "#define MAX 10\n";
        let mut lx = Lexer::new(s);

        while let Some(token) = lx.next_token() {
            match token {
                Token::Define { macro_name, macro_value, span } => {
                    assert_eq!(macro_name, "MAX");
                    assert_eq!(macro_value, "10");
                    assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "#define MAX 10\n");
                    return;
                }
                _ => {}
            }
        }
        panic!("Define token not found");
    }

    #[test]
    fn test_lexer_define_leading_whitespace_included() {
        let s = "\t \r #define X 1\n";
        let mut lx = Lexer::new(s);

        while let Some(token) = lx.next_token() {
            match token {
                Token::Define { macro_name, macro_value, span } => {
                    assert_eq!(macro_name, "X");
                    assert_eq!(macro_value, "1");
                    assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "\t \r #define X 1\n");
                    return;
                }
                _ => {}
            }
        }
        panic!("Define token not found");
    }

    #[test]
    fn test_parser_defines_produced_items() {
        let s = "#define A 1\n#define B 2\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 2);

        match &tu.items[0] {
            Item::Define { text, macro_name, macro_value, .. } => {
                assert_eq!(macro_name, "A");
                assert_eq!(macro_value, "1");
                assert_eq!(text, "#define A 1\n");
            }
            _ => panic!("first item is not Define"),
        }

        match &tu.items[1] {
            Item::Define { text, macro_name, macro_value, .. } => {
                assert_eq!(macro_name, "B");
                assert_eq!(macro_value, "2");
                assert_eq!(text, "#define B 2\n");
            }
            _ => panic!("second item is not Define"),
        }
    }

    #[test]
    fn test_formatter_format_define_keeps_newline_only() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        let text = String::from("\t\r\n  #define Z 42\n");
        let item = Item::Define { span, text: text.clone(), macro_name: "Z".into(), macro_value: "42".into() };
        let tu = TranslationUnit { items: vec![item] };
        let fmt = Formatter::new();
        let out = fmt.format_tu(&tu);
        // leading \t\r and spaces removed, newline kept, then the rest starts with '#'
        assert_eq!(out, "\n#define Z 42\n");
    }

    #[test]
    fn test_lexer_typedef_simple() {
        let s = "typedef";
        let mut lx = Lexer::new(s);

        while let Some(token) = lx.next_token() {
            match token {
                Token::Typedef { span } => {
                    assert_eq!(span.start_line, 0);
                    assert_eq!(span.start_column, 0);
                    assert_eq!(span.end_line, 0);
                    assert_eq!(span.end_column, s.len());
                    assert_eq!(span.byte_start_idx, 0);
                    assert_eq!(span.byte_end_idx, s.len());
                    assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "typedef");
                    return;
                }
                _ => { panic!("Unexpected token"); }
            }
        }
        panic!("Typedef token not found");
    }

    #[test]
    fn test_lexer_typedef_with_leading_whitespace() {
        let s = "  \t typedef int MyInt;";
        let mut lx = Lexer::new(s);

        while let Some(token) = lx.next_token() {
            match token {
                Token::Typedef { span } => {
                    assert_eq!(span.start_line, 0);
                    assert_eq!(span.start_column, 0);
                    // 先頭の空白も含まれる
                    assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "  \t typedef");
                    return;
                }
                _ => {}
            }
        }
        panic!("Typedef token not found");
    }

    #[test]
    fn test_lexer_typedef_case_sensitive() {
        // TYPEDEF（大文字）は識別子扱いになるはず
        let s = "TYPEDEF int MyInt;";
        let mut lx = Lexer::new(s);

        let token = lx.next_token();

        match token {
            Some(Token::Ident { span, name }) => {
                assert_eq!(name, "TYPEDEF");
                assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "TYPEDEF");
            }
            _ => {
                panic!("Expected Ident token for 'TYPEDEF'");
            }
        }
    }

    #[test]
    fn test_lexer_typedef_multiple() {
        let s = "typedef int A;\ntypedef float B;";
        let mut lx = Lexer::new(s);

        let mut typedef_count = 0;
        while let Some(token) = lx.next_token() {
            if let Token::Typedef { .. } = token {
                typedef_count += 1;
            }
        }
        assert_eq!(typedef_count, 2, "Should find two typedef keywords");
    }

    #[test]
    fn test_lexer_define_with_japanese_after() {
        // 日本語文字の直前でトークンが終わるケースをテスト
        let s = "#define A B\n#include \"XXX.h\" // XXX.h をインクルード\n";
        let mut lx = Lexer::new(s);

        // 最初のトークンは #define
        let token1 = lx.next_token();
        match token1 {
            Some(Token::Define { macro_name, macro_value, span }) => {
                assert_eq!(macro_name, "A");
                assert_eq!(macro_value, "B");
                // バイト境界が正しいことを確認（パニックしないこと）
                assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "#define A B\n");
            }
            _ => panic!("Expected Define token"),
        }

        // 次のトークンは #include
        let token2 = lx.next_token();
        match token2 {
            Some(Token::Include { filename, span }) => {
                assert_eq!(filename, "XXX.h");
                // バイト境界が正しいことを確認（パニックしないこと）
                let text = &s[span.byte_start_idx..span.byte_end_idx];
                assert!(text.starts_with("#include \"XXX.h\""));
            }
            _ => panic!("Expected Include token"),
        }
    }

    #[test]
    fn test_lexer_multiple_directives_with_japanese() {
        // example.txt と同じ構造をテスト
        let s = "/* はじまり */\n#define A B\n#include \"XXX.h\" // XXX.h をインクルード\n#include <YYY.h> /* YYY.h をインクルード */\n/* おわり */";
        let mut lx = Lexer::new(s);

        let mut token_count = 0;
        while let Some(token) = lx.next_token() {
            token_count += 1;
            match token {
                Token::BlockComment { span } => {
                    // パニックしないことを確認
                    let _ = &s[span.byte_start_idx..span.byte_end_idx];
                }
                Token::Define { span, .. } => {
                    // パニックしないことを確認
                    let _ = &s[span.byte_start_idx..span.byte_end_idx];
                }
                Token::Include { span, .. } => {
                    // パニックしないことを確認
                    let _ = &s[span.byte_start_idx..span.byte_end_idx];
                }
                Token::Ident { span, .. } => {
                    // パニックしないことを確認
                    let _ = &s[span.byte_start_idx..span.byte_end_idx];
                }
                _ => {}
            }
        }
        
        // すべてのトークンが正しく読み取れたことを確認
        assert!(token_count >= 5, "Should tokenize at least 5 tokens (2 comments, 1 define, 2 includes)");
    }

    #[test]
    fn test_parser_with_japanese_content() {
        // パーサーが日本語を含むコンテンツを正しく処理できることを確認
        let s = "/* はじまり */\n#define A B\n#include \"XXX.h\"\n/* おわり */\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 4, "Should parse 4 items");

        // 各アイテムのテキストがUTF-8境界で正しく切り出されていることを確認
        for item in &tu.items {
            match item {
                Item::BlockComment { text, .. } => {
                    assert!(text.contains("はじまり") || text.contains("おわり"));
                }
                Item::Define { text, macro_name, macro_value, .. } => {
                    assert_eq!(macro_name, "A");
                    assert_eq!(macro_value, "B");
                    assert!(text.contains("#define A B"));
                }
                Item::Include { text, filename, .. } => {
                    assert_eq!(filename, "XXX.h");
                    assert!(text.contains("#include \"XXX.h\""));
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_lexer_int_keyword() {
        let s = "int x;";
        let mut lx = Lexer::new(s);

        let token1 = lx.next_token();
        match token1 {
            Some(Token::Int { span }) => {
                assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "int");
            }
            _ => panic!("Expected Int token"),
        }
    }

    #[test]
    fn test_parser_simple_var_decl() {
        let s = "int x;\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 1);

        match &tu.items[0] {
            Item::VarDecl { text, var_name, has_initializer, .. } => {
                assert_eq!(var_name, "x");
                assert_eq!(*has_initializer, false);
                assert_eq!(text, "int x;");
            }
            _ => panic!("Expected VarDecl item"),
        }
    }

    #[test]
    fn test_parser_var_decl_with_initializer() {
        let s = "int x = 10;\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 1);

        match &tu.items[0] {
            Item::VarDecl { text, var_name, has_initializer, .. } => {
                assert_eq!(var_name, "x");
                assert_eq!(*has_initializer, true);
                assert!(text.contains("int x ="));
            }
            _ => panic!("Expected VarDecl item"),
        }
    }

    #[test]
    fn test_parser_var_decl_with_storage_class() {
        let s = "static int counter;\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 1);

        match &tu.items[0] {
            Item::VarDecl { text, var_name, has_initializer, .. } => {
                assert_eq!(var_name, "counter");
                assert_eq!(*has_initializer, false);
                assert_eq!(text, "static int counter;");
            }
            _ => panic!("Expected VarDecl item"),
        }
    }

    #[test]
    fn test_parser_var_decl_with_qualifiers() {
        let s = "const volatile int value = 42;\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 1);

        match &tu.items[0] {
            Item::VarDecl { text, var_name, has_initializer, .. } => {
                assert_eq!(var_name, "value");
                assert_eq!(*has_initializer, true);
                assert!(text.contains("const"));
                assert!(text.contains("volatile"));
            }
            _ => panic!("Expected VarDecl item"),
        }
    }

    #[test]
    fn test_parser_multiple_var_decls() {
        let s = "int a;\nfloat b = 3.14;\nstatic char c;\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 3);

        match &tu.items[0] {
            Item::VarDecl { var_name, .. } => assert_eq!(var_name, "a"),
            _ => panic!("Expected VarDecl"),
        }

        match &tu.items[1] {
            Item::VarDecl { var_name, has_initializer, .. } => {
                assert_eq!(var_name, "b");
                assert_eq!(*has_initializer, true);
            }
            _ => panic!("Expected VarDecl"),
        }

        match &tu.items[2] {
            Item::VarDecl { var_name, .. } => assert_eq!(var_name, "c"),
            _ => panic!("Expected VarDecl"),
        }
    }

    #[test]
    fn test_formatter_var_decl() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        let item = Item::VarDecl { 
            span, 
            text: String::from("  int x;"),
            var_name: String::from("x"),
            has_initializer: false,
        };
        let tu = TranslationUnit { items: vec![item] };
        let fmt = Formatter::new();
        let out = fmt.format_tu(&tu);
        assert_eq!(out, "int x;");
    }

    #[test]
    fn test_parser_mixed_items() {
        let s = "/* comment */\n#include <stdio.h>\nint x = 5;\ntypedef int myint;\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 4);
        
        assert!(matches!(&tu.items[0], Item::BlockComment { .. }));
        assert!(matches!(&tu.items[1], Item::Include { .. }));
        assert!(matches!(&tu.items[2], Item::VarDecl { .. }));
        assert!(matches!(&tu.items[3], Item::TypedefDecl { .. }));
    }

    #[test]
    fn test_lexer_struct_keyword() {
        let s = "struct Point { int x; int y; };";
        let mut lx = Lexer::new(s);

        let token1 = lx.next_token();
        match token1 {
            Some(Token::Struct { span }) => {
                assert_eq!(&s[span.byte_start_idx..span.byte_end_idx], "struct");
            }
            _ => panic!("Expected Struct token"),
        }
    }

    #[test]
    fn test_parser_simple_struct_decl() {
        let s = "struct Point { int x; int y; };\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 1);

        match &tu.items[0] {
            Item::StructDecl { text, struct_name, has_typedef, .. } => {
                assert_eq!(struct_name.as_ref().unwrap(), "Point");
                assert_eq!(*has_typedef, false);
                assert!(text.contains("struct Point"));
            }
            _ => panic!("Expected StructDecl item"),
        }
    }

    #[test]
    fn test_parser_anonymous_struct() {
        let s = "struct { int x; int y; } point;\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 1);

        match &tu.items[0] {
            Item::StructDecl { struct_name, .. } => {
                // 無名構造体は point が変数名なので struct_name は None か point
                assert!(struct_name.is_none() || struct_name.as_ref().unwrap() == "point");
            }
            _ => panic!("Expected StructDecl item"),
        }
    }

    #[test]
    fn test_parser_typedef_struct() {
        let s = "typedef struct { int x; int y; } Point;\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 1);

        match &tu.items[0] {
            Item::StructDecl { text, has_typedef, .. } => {
                assert_eq!(*has_typedef, true);
                assert!(text.contains("typedef struct"));
            }
            _ => panic!("Expected StructDecl item"),
        }
    }

    #[test]
    fn test_parser_typedef_struct_with_name() {
        let s = "typedef struct Point { int x; int y; } Point;\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 1);

        match &tu.items[0] {
            Item::StructDecl { text, struct_name, has_typedef, .. } => {
                assert_eq!(struct_name.as_ref().unwrap(), "Point");
                assert_eq!(*has_typedef, true);
                assert!(text.contains("typedef struct Point"));
            }
            _ => panic!("Expected StructDecl item"),
        }
    }

    #[test]
    fn test_parser_struct_variable_decl() {
        let s = "struct Point p;\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 1);

        match &tu.items[0] {
            Item::StructDecl { text, struct_name, .. } => {
                assert_eq!(struct_name.as_ref().unwrap(), "Point");
                assert!(text.contains("struct Point p"));
            }
            _ => panic!("Expected StructDecl item"),
        }
    }

    #[test]
    fn test_parser_nested_struct() {
        let s = "struct Outer { struct Inner { int val; } inner; int x; };\n";
        let lx = Lexer::new(s);
        let mut parser = Parser::new(lx);
        let tu = parser.parse();

        assert_eq!(tu.items.len(), 1);

        match &tu.items[0] {
            Item::StructDecl { text, struct_name, .. } => {
                assert_eq!(struct_name.as_ref().unwrap(), "Outer");
                assert!(text.contains("struct Outer"));
                assert!(text.contains("struct Inner"));
            }
            _ => panic!("Expected StructDecl item"),
        }
    }

    #[test]
    fn test_formatter_struct_decl() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 };
        let item = Item::StructDecl {
            span,
            text: String::from("  struct Point { int x; };"),
            struct_name: Some(String::from("Point")),
            has_typedef: false,
        };
        let tu = TranslationUnit { items: vec![item] };
        let fmt = Formatter::new();
        let out = fmt.format_tu(&tu);
        assert_eq!(out, "struct Point { int x; };");
    }
}
