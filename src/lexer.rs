use std::str::CharIndices;
use crate::token::Token;
use crate::span::Span;

#[derive(Debug)]
pub struct Lexer<'a> {
    pub input: &'a str,
    char_offsets: CharIndices<'a>, // 文字のバイトオフセットと文字のイテレータ
    cur: usize,               // 次に読む文字のインデックス (0..=len)
    pub column: usize,
    pub line: usize,
    now: Option<(usize, char)>,
    peeked: Option<(usize, char)>,
}

// Lexer の実装
// 文字列をトークンに分ける
impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
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
    pub fn next_char(&mut self) -> Option<(usize, char)> {

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
    pub fn peek(&self) -> Option<(usize, char)> {
        self.peeked
    }

    pub fn pos_index(&self) -> usize {
        self.cur
    }

    // トークンを一つ読み取る
    pub fn next_token(&mut self) -> Option<Token> {
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
