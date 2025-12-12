use std::str::CharIndices;
use crate::token::*;
use crate::span::Span;

#[derive(Debug)]
pub struct Lexer {
    pub input: String,
    char_offsets: CharIndices<'static>,
    cur: usize,
    pub column: usize,
    pub line: usize,
    now: Option<(usize, char)>,
    peeked: Option<(usize, char)>,
}

// Lexer の実装
// 文字列をトークンに分ける
impl Lexer {
    pub fn new(input: &str) -> Self {
        let input_string = input.to_string();
        // SAFETY: Stringを所有しているため'staticライフタイムに変換
        // Lexerが生きている間は有効
        let static_str: &'static str = unsafe {
            std::mem::transmute(input_string.as_str())
        };
        
        let mut lx = Lexer {
            input: input_string,
            char_offsets: static_str.char_indices(),
            cur: 0,
            column: 0,
            line: 0,
            now: None,
            peeked: None,
        };
        // peeked に最初の文字を入れる
        lx.peeked = lx.char_offsets.next();
        lx
    }

    // 記号ではないキーワードはここで処理する
    fn keyword_to_token(&self, byte_idx_start: usize, byte_idx_end: usize, span: Span) -> Option<Token> {
        match &self.input[byte_idx_start..byte_idx_end] {
            "auto" => Some(Token::Auto(AutoToken { span })),
            "register" => Some(Token::Register(RegisterToken { span })),
            "static" => Some(Token::Static(StaticToken { span })),
            "extern" => Some(Token::Extern(ExternToken { span })),
            "typedef" => Some(Token::Typedef(TypedefToken { span })),
            "const" => Some(Token::Const(ConstToken { span })),
            "volatile" => Some(Token::Volatile(VolatileToken { span })),
            "restrict" => Some(Token::Restrict(RestrictToken { span })),
            "_Atomic" => Some(Token::Atomic(AtomicToken { span })),
            "int" => Some(Token::Int(IntToken { span })),
            "char" => Some(Token::Char(CharToken { span })),
            "float" => Some(Token::Float(FloatToken { span })),
            "double" => Some(Token::Double(DoubleToken { span })),
            "void" => Some(Token::Void(VoidToken { span })),
            "long" => Some(Token::Long(LongToken { span })),
            "short" => Some(Token::Short(ShortToken { span })),
            "signed" => Some(Token::Signed(SignedToken { span })),
            "unsigned" => Some(Token::Unsigned(UnsignedToken { span })),
            "struct" => Some(Token::Struct(StructToken { span })),
            "enum" => Some(Token::Enum(EnumToken { span })),
            "union" => Some(Token::Union(UnionToken { span })),
            _ => Some(Token::Ident(IdentToken {
                span,
                name: self.input[byte_idx_start..byte_idx_end].to_string(),
            })),
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
        // 初回呼び出し時に now を初期化
        if self.now.is_none() && self.peeked.is_some() {
            self.next_char();
            // 初回呼び出し時に位置情報をリセット
            self.line = 0;
            self.column = 0;
        }
        
        let start_line = self.line;
        let start_column = self.column;
        let mut start_byte_flag: Option<usize> = None;

        loop {
            match self.now {
                Some((byte_idx, ' ')) | Some((byte_idx, '\t')) | Some((byte_idx, '\n')) | Some((byte_idx, '\r')) => {
                    // 空白文字はスキップ
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();
                    continue;
                },
                Some((byte_idx, ';')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };

                    let end_line = self.line;
                    let end_column = self.column;

                    return Some(Token::Semicolon(SemicolonToken {
                        span: Span {
                            start_line,
                            start_column,
                            end_line,
                            end_column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    }));
                },
                Some((byte_idx, '=')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    // Check for ==
                    if let Some((_, '=')) = self.now {
                        self.next_char();
                        let end_byte = if let Some((b, _)) = self.peeked {
                            b
                        } else {
                            self.input.len()
                        };
                        return Some(Token::EqualEqual(EqualEqualToken {
                            span: Span {
                                start_line,
                                start_column,
                                end_line: self.line,
                                end_column: self.column,
                                byte_start_idx: start_byte_flag.unwrap(),
                                byte_end_idx: end_byte,
                            }
                        }));
                    }

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };

                    let end_line = self.line;
                    let end_column = self.column;

                    return Some(Token::Equal(EqualToken {
                        span: Span {
                            start_line,
                            start_column,
                            end_line,
                            end_column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    }));
                },
                Some((byte_idx, '*')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };

                    let end_line = self.line;
                    let end_column = self.column;

                    return Some(Token::Asterisk(AsteriskToken {
                        span: Span {
                            start_line,
                            start_column,
                            end_line,
                            end_column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    }));
                },
                Some((byte_idx, '+')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    // Check for ++
                    if let Some((_, '+')) = self.now {
                        self.next_char();
                        let end_byte = if let Some((b, _)) = self.peeked {
                            b
                        } else {
                            self.input.len()
                        };
                        return Some(Token::PlusPlus(PlusPlusToken {
                            span: Span {
                                start_line,
                                start_column,
                                end_line: self.line,
                                end_column: self.column,
                                byte_start_idx: start_byte_flag.unwrap(),
                                byte_end_idx: end_byte,
                            }
                        }));
                    }

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };
                    return Some(Token::Plus(PlusToken {
                        span: Span {
                            start_line,
                            start_column,
                            end_line: self.line,
                            end_column: self.column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    }));
                },
                Some((byte_idx, '-')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    // Check for -- or ->
                    match self.now {
                        Some((_, '-')) => {
                            self.next_char();
                            let end_byte = if let Some((b, _)) = self.peeked {
                                b
                            } else {
                                self.input.len()
                            };
                            return Some(Token::MinusMinus(MinusMinusToken {
                                span: Span {
                                    start_line,
                                    start_column,
                                    end_line: self.line,
                                    end_column: self.column,
                                    byte_start_idx: start_byte_flag.unwrap(),
                                    byte_end_idx: end_byte,
                                }
                            }));
                        },
                        Some((_, '>')) => {
                            self.next_char();
                            let end_byte = if let Some((b, _)) = self.peeked {
                                b
                            } else {
                                self.input.len()
                            };
                            return Some(Token::Arrow(ArrowToken {
                                span: Span {
                                    start_line,
                                    start_column,
                                    end_line: self.line,
                                    end_column: self.column,
                                    byte_start_idx: start_byte_flag.unwrap(),
                                    byte_end_idx: end_byte,
                                }
                            }));
                        },
                        _ => {
                            let end_byte = if let Some((b, _)) = self.peeked {
                                b
                            } else {
                                self.input.len()
                            };
                            return Some(Token::Minus(MinusToken {
                                span: Span {
                                    start_line,
                                    start_column,
                                    end_line: self.line,
                                    end_column: self.column,
                                    byte_start_idx: start_byte_flag.unwrap(),
                                    byte_end_idx: end_byte,
                                }
                            }));
                        }
                    }
                },
                Some((byte_idx, '%')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };
                    return Some(Token::Percent(PercentToken {
                        span: Span {
                            start_line,
                            start_column,
                            end_line: self.line,
                            end_column: self.column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    }));
                },
                Some((byte_idx, '!')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    // Check for !=
                    if let Some((_, '=')) = self.now {
                        self.next_char();
                        let end_byte = if let Some((b, _)) = self.peeked {
                            b
                        } else {
                            self.input.len()
                        };
                        return Some(Token::NotEqual(NotEqualToken {
                            span: Span {
                                start_line,
                                start_column,
                                end_line: self.line,
                                end_column: self.column,
                                byte_start_idx: start_byte_flag.unwrap(),
                                byte_end_idx: end_byte,
                            }
                        }));
                    }

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };
                    return Some(Token::Exclamation(ExclamationToken {
                        span: Span {
                            start_line,
                            start_column,
                            end_line: self.line,
                            end_column: self.column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    }));
                },
                Some((byte_idx, '<')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    // Check for <= or <<
                    match self.now {
                        Some((_, '=')) => {
                            self.next_char();
                            let end_byte = if let Some((b, _)) = self.peeked {
                                b
                            } else {
                                self.input.len()
                            };
                            return Some(Token::LessThanOrEqual(LessThanOrEqualToken {
                                span: Span {
                                    start_line,
                                    start_column,
                                    end_line: self.line,
                                    end_column: self.column,
                                    byte_start_idx: start_byte_flag.unwrap(),
                                    byte_end_idx: end_byte,
                                }
                            }));
                        },
                        Some((_, '<')) => {
                            self.next_char();
                            let end_byte = if let Some((b, _)) = self.peeked {
                                b
                            } else {
                                self.input.len()
                            };
                            return Some(Token::LeftShift(LeftShiftToken {
                                span: Span {
                                    start_line,
                                    start_column,
                                    end_line: self.line,
                                    end_column: self.column,
                                    byte_start_idx: start_byte_flag.unwrap(),
                                    byte_end_idx: end_byte,
                                }
                            }));
                        },
                        _ => {
                            let end_byte = if let Some((b, _)) = self.peeked {
                                b
                            } else {
                                self.input.len()
                            };
                            return Some(Token::LessThan(LessThanToken {
                                span: Span {
                                    start_line,
                                    start_column,
                                    end_line: self.line,
                                    end_column: self.column,
                                    byte_start_idx: start_byte_flag.unwrap(),
                                    byte_end_idx: end_byte,
                                }
                            }));
                        }
                    }
                },
                Some((byte_idx, '>')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    // Check for >= or >>
                    match self.now {
                        Some((_, '=')) => {
                            self.next_char();
                            let end_byte = if let Some((b, _)) = self.peeked {
                                b
                            } else {
                                self.input.len()
                            };
                            return Some(Token::GreaterThanOrEqual(GreaterThanOrEqualToken {
                                span: Span {
                                    start_line,
                                    start_column,
                                    end_line: self.line,
                                    end_column: self.column,
                                    byte_start_idx: start_byte_flag.unwrap(),
                                    byte_end_idx: end_byte,
                                }
                            }));
                        },
                        Some((_, '>')) => {
                            self.next_char();
                            let end_byte = if let Some((b, _)) = self.peeked {
                                b
                            } else {
                                self.input.len()
                            };
                            return Some(Token::RightShift(RightShiftToken {
                                span: Span {
                                    start_line,
                                    start_column,
                                    end_line: self.line,
                                    end_column: self.column,
                                    byte_start_idx: start_byte_flag.unwrap(),
                                    byte_end_idx: end_byte,
                                }
                            }));
                        },
                        _ => {
                            let end_byte = if let Some((b, _)) = self.peeked {
                                b
                            } else {
                                self.input.len()
                            };
                            return Some(Token::GreaterThan(GreaterThanToken {
                                span: Span {
                                    start_line,
                                    start_column,
                                    end_line: self.line,
                                    end_column: self.column,
                                    byte_start_idx: start_byte_flag.unwrap(),
                                    byte_end_idx: end_byte,
                                }
                            }));
                        }
                    }
                },
                Some((byte_idx, '&')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    // Check for &&
                    if let Some((_, '&')) = self.now {
                        self.next_char();
                        let end_byte = if let Some((b, _)) = self.peeked {
                            b
                        } else {
                            self.input.len()
                        };
                        return Some(Token::AmpersandAmpersand(AmpersandAmpersandToken {
                            span: Span {
                                start_line,
                                start_column,
                                end_line: self.line,
                                end_column: self.column,
                                byte_start_idx: start_byte_flag.unwrap(),
                                byte_end_idx: end_byte,
                            }
                        }));
                    }

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };
                    return Some(Token::Ampersand(AmpersandToken {
                        span: Span {
                            start_line,
                            start_column,
                            end_line: self.line,
                            end_column: self.column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    }));
                },
                Some((byte_idx, '|')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    // Check for ||
                    if let Some((_, '|')) = self.now {
                        self.next_char();
                        let end_byte = if let Some((b, _)) = self.peeked {
                            b
                        } else {
                            self.input.len()
                        };
                        return Some(Token::PipePipe(PipePipeToken {
                            span: Span {
                                start_line,
                                start_column,
                                end_line: self.line,
                                end_column: self.column,
                                byte_start_idx: start_byte_flag.unwrap(),
                                byte_end_idx: end_byte,
                            }
                        }));
                    }

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };
                    return Some(Token::Pipe(PipeToken {
                        span: Span {
                            start_line,
                            start_column,
                            end_line: self.line,
                            end_column: self.column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    }));
                },
                Some((byte_idx, '^')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };
                    return Some(Token::Caret(CaretToken {
                        span: Span {
                            start_line,
                            start_column,
                            end_line: self.line,
                            end_column: self.column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    }));
                },
                Some((byte_idx, '~')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };
                    return Some(Token::Tilde(TildeToken {
                        span: Span {
                            start_line,
                            start_column,
                            end_line: self.line,
                            end_column: self.column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    }));
                },
                Some((byte_idx, '?')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };
                    return Some(Token::Question(QuestionToken {
                        span: Span {
                            start_line,
                            start_column,
                            end_line: self.line,
                            end_column: self.column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    }));
                },
                Some((byte_idx, ':')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };
                    return Some(Token::Colon(ColonToken {
                        span: Span {
                            start_line,
                            start_column,
                            end_line: self.line,
                            end_column: self.column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    }));
                },
                Some((byte_idx, ',')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };
                    return Some(Token::Comma(CommaToken {
                        span: Span {
                            start_line,
                            start_column,
                            end_line: self.line,
                            end_column: self.column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    }));
                },
                Some((byte_idx, '.')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };
                    return Some(Token::Dot(DotToken {
                        span: Span {
                            start_line,
                            start_column,
                            end_line: self.line,
                            end_column: self.column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    }));
                },
                Some((byte_idx, '[')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };
                    return Some(Token::LeftBracket(LeftBracketToken {
                        span: Span {
                            start_line,
                            start_column,
                            end_line: self.line,
                            end_column: self.column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    }));
                },
                Some((byte_idx, ']')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };
                    return Some(Token::RightBracket(RightBracketToken {
                        span: Span {
                            start_line,
                            start_column,
                            end_line: self.line,
                            end_column: self.column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    }));
                },
                Some((byte_idx, '{')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };

                    return Some(Token::LeftBrace(LeftBraceToken {
                        span: Span {
                            start_line,
                            start_column,
                            end_line: self.line,
                            end_column: self.column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    }));
                },
                Some((byte_idx, '}')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };

                    return Some(Token::RightBrace(RightBraceToken {
                        span: Span {
                            start_line,
                            start_column,
                            end_line: self.line,
                            end_column: self.column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    }));
                },
                Some((byte_idx, '(')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };

                    return Some(Token::LeftParen(LeftParenToken {
                        span: Span {
                            start_line,
                            start_column,
                            end_line: self.line,
                            end_column: self.column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    }));
                },
                Some((byte_idx, ')')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    let end_byte = if let Some((b, _)) = self.peeked {
                        b
                    } else {
                        self.input.len()
                    };

                    return Some(Token::RightParen(RightParenToken {
                        span: Span {
                            start_line,
                            start_column,
                            end_line: self.line,
                            end_column: self.column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte,
                        }
                    }));
                },
                Some((byte_idx, '/')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();
                    
                    if let Some((_, '*')) = self.now {
                        // ブロックコメントの開始
                        // '*' を消費
                        self.next_char();
    
                        // コメントの終わりを探す
                        loop {
                            match self.now {
                                Some((_, '*')) => {
                                    self.next_char();
                                    if let Some((byte_idx, '/')) = self.now {
                                        // '/' を読んだ時点で column は '/' の位置
                                        // 終端は '/' の次の位置なので + 1
                                        let end_line = self.line;
                                        let end_column = self.column + 1;
                                        
                                        // '/' の終端バイト位置を計算
                                        let end_byte = byte_idx + '/'.len_utf8();
                                        
                                        // '/' を消費
                                        self.next_char();

                                        return Some(Token::BlockComment(BlockCommentToken {
                                            span: Span {
                                                start_line,
                                                start_column,
                                                end_line,
                                                end_column,
                                                byte_start_idx: start_byte_flag.unwrap(),
                                                byte_end_idx: end_byte,
                                            }
                                        }));
                                    }
                                }
                                Some(_) => {
                                    self.next_char();
                                }
                                None => return None,
                            }
                        }
                    } else if let Some((_, '/')) = self.now {
                        // 行コメントの開始 //
                        // 2つ目の '/' を消費
                        self.next_char();
                        
                        // 行末まで読む
                        loop {
                            match self.now {
                                Some((byte_idx, '\n')) | Some((byte_idx, '\r')) => {
                                    // 改行文字の位置を記録
                                    let end_line = self.line;
                                    let end_column = self.column;
                                    let end_byte = byte_idx;
                                    
                                    return Some(Token::LineComment(LineCommentToken {
                                        span: Span {
                                            start_line,
                                            start_column,
                                            end_line,
                                            end_column,
                                            byte_start_idx: start_byte_flag.unwrap(),
                                            byte_end_idx: end_byte,
                                        }
                                    }));
                                }
                                None => {
                                    // EOF
                                    let end_line = self.line;
                                    let end_column = self.column;
                                    let end_byte = self.cur;
                                    
                                    return Some(Token::LineComment(LineCommentToken {
                                        span: Span {
                                            start_line,
                                            start_column,
                                            end_line,
                                            end_column,
                                            byte_start_idx: start_byte_flag.unwrap(),
                                            byte_end_idx: end_byte,
                                        }
                                    }));
                                }
                                Some(_) => {
                                    self.next_char();
                                }
                            }
                        }
                    } else {
                        // '/' 単独のトークン
                        let end_byte = if let Some((b, _)) = self.peeked {
                            b
                        } else {
                            self.input.len()
                        };
                        return Some(Token::Slash(SlashToken {
                            span: Span {
                                start_line,
                                start_column,
                                end_line: self.line,
                                end_column: self.column,
                                byte_start_idx: start_byte_flag.unwrap(),
                                byte_end_idx: end_byte,
                            }
                        }));
                    }
                },
                Some((byte_idx, '#')) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }
                    self.next_char();

                    // ディレクティブを行末まで読み取る（先頭の空白は start_char_idx でカバーされる）
                    let mut directive_text = String::new();
                    directive_text.push('#');

                    loop {
                        match self.now {
                            Some((_, ch)) => {
                                directive_text.push(ch);
                                self.next_char();
                                if ch == '\n' {
                                    break;
                                }
                            }
                            None => break,
                        }
                    }

                    // directive の中身（# を取り除いた後）を解析用に取得（先頭空白は trim_start する）
                    let content = directive_text.trim_start_matches('#').trim_start().to_string();

                    // バイトオフセットを計算：現在の文字のバイト位置か入力終端
                    let end_byte_idx = if let Some((b, _)) = self.now {
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

                        return Some(Token::Include(IncludeToken {
                            span: Span {
                                start_line,
                                start_column,
                                end_line,
                                end_column,
                                byte_start_idx: start_byte_flag.unwrap(),
                                byte_end_idx: end_byte_idx,
                            },
                            filename: filename.to_string(),
                        }));
                    }

                    // #define の処理：先頭の空白は token の offset/length に含まれる（start_byte がそれを指す）
                    if let Some(rest) = content.strip_prefix("define") {
                        let rest = rest.trim();
                        let mut parts = rest.splitn(2, ' ');
                        if let Some(name) = parts.next() {
                            let macro_name = name.to_string();
                            let macro_value = parts.next().unwrap_or("").to_string();

                            return Some(Token::Define(DefineToken {
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
                            }));
                        }
                    }

                    // #ifdef の処理
                    if let Some(_rest) = content.strip_prefix("ifdef") {
                        return Some(Token::Ifdef(IfdefToken {
                            span: Span {
                                start_line,
                                start_column,
                                end_line,
                                end_column,
                                byte_start_idx: start_byte_flag.unwrap(),
                                byte_end_idx: end_byte_idx,
                            },
                        }));
                    }

                    // #ifndef の処理
                    if let Some(_rest) = content.strip_prefix("ifndef") {
                        return Some(Token::Ifndef(IfndefToken {
                            span: Span {
                                start_line,
                                start_column,
                                end_line,
                                end_column,
                                byte_start_idx: start_byte_flag.unwrap(),
                                byte_end_idx: end_byte_idx,
                            },
                        }));
                    }

                    // #if の処理
                    if content.starts_with("if") && !content.starts_with("ifdef") && !content.starts_with("ifndef") {
                        return Some(Token::If(IfToken {
                            span: Span {
                                start_line,
                                start_column,
                                end_line,
                                end_column,
                                byte_start_idx: start_byte_flag.unwrap(),
                                byte_end_idx: end_byte_idx,
                            },
                        }));
                    }

                    // #elif の処理
                    if let Some(_rest) = content.strip_prefix("elif") {
                        return Some(Token::Elif(ElifToken {
                            span: Span {
                                start_line,
                                start_column,
                                end_line,
                                end_column,
                                byte_start_idx: start_byte_flag.unwrap(),
                                byte_end_idx: end_byte_idx,
                            },
                        }));
                    }

                    // #else の処理
                    if content.starts_with("else") {
                        return Some(Token::Else(ElseToken {
                            span: Span {
                                start_line,
                                start_column,
                                end_line,
                                end_column,
                                byte_start_idx: start_byte_flag.unwrap(),
                                byte_end_idx: end_byte_idx,
                            },
                        }));
                    }

                    // #endif の処理
                    if content.starts_with("endif") {
                        return Some(Token::Endif(EndifToken {
                            span: Span {
                                start_line,
                                start_column,
                                end_line,
                                end_column,
                                byte_start_idx: start_byte_flag.unwrap(),
                                byte_end_idx: end_byte_idx,
                            },
                        }));
                    }

                    // それ以外の # 系ディレクティブはとりあえず Include 風に生テキストを残す（既存互換）
                    return Some(Token::Include(IncludeToken {
                        span: Span {
                            start_line,
                            start_column,
                            end_line,
                            end_column,
                            byte_start_idx: start_byte_flag.unwrap(),
                            byte_end_idx: end_byte_idx,
                        },
                        filename: content.to_string(),
                    }));
                },
                Some((byte_idx, ch)) => {
                    if start_byte_flag.is_none() {
                        start_byte_flag = Some(byte_idx);
                    }

                    // 数字リテラルの判定
                    if ch.is_ascii_digit() {
                        let actual_byte_idx_start = byte_idx;
                        self.next_char();
                        let mut is_float = false;

                        // 16進数 (0x または 0X)
                        if actual_byte_idx_start + 1 < self.input.len() 
                            && &self.input[actual_byte_idx_start..actual_byte_idx_start+1] == "0"
                            && matches!(self.now, Some((_, 'x')) | Some((_, 'X'))) {
                            self.next_char(); // 'x' または 'X' を消費
                            
                            // 16進数の桁を読む
                            loop {
                                match self.now {
                                    Some((_, next_ch)) if next_ch.is_ascii_hexdigit() => {
                                        self.next_char();
                                    }
                                    _ => break,
                                }
                            }
                        } else {
                            // 8進数または10進数
                            loop {
                                match self.now {
                                    Some((_, next_ch)) if next_ch.is_ascii_digit() => {
                                        self.next_char();
                                    }
                                    _ => break,
                                }
                            }
                            
                            // 小数点をチェック
                            if matches!(self.now, Some((_, '.'))) {
                                is_float = true;
                                self.next_char(); // '.' を消費
                                
                                // 小数部を読む
                                loop {
                                    match self.now {
                                        Some((_, next_ch)) if next_ch.is_ascii_digit() => {
                                            self.next_char();
                                        }
                                        _ => break,
                                    }
                                }
                            }
                            
                            // 指数部をチェック (e または E)
                            if matches!(self.now, Some((_, 'e')) | Some((_, 'E'))) {
                                is_float = true;
                                self.next_char(); // 'e' または 'E' を消費
                                
                                // 符号をチェック (+ または -)
                                if matches!(self.now, Some((_, '+')) | Some((_, '-'))) {
                                    self.next_char();
                                }
                                
                                // 指数部の数値を読む
                                loop {
                                    match self.now {
                                        Some((_, next_ch)) if next_ch.is_ascii_digit() => {
                                            self.next_char();
                                        }
                                        _ => break,
                                    }
                                }
                            }
                        }

                        // suffix を読む
                        if is_float {
                            // float suffix: f, F, l, L
                            loop {
                                match self.now {
                                    Some((_, 'f')) | Some((_, 'F')) | Some((_, 'l')) | Some((_, 'L')) => {
                                        self.next_char();
                                    }
                                    _ => break,
                                }
                            }
                        } else {
                            // integer suffix: u, U, l, L
                            loop {
                                match self.now {
                                    Some((_, 'u')) | Some((_, 'U')) | Some((_, 'l')) | Some((_, 'L')) => {
                                        self.next_char();
                                    }
                                    _ => break,
                                }
                            }
                        }

                        // 数値リテラルの終端
                        let actual_byte_idx_end = if let Some((b, _)) = self.now {
                            b
                        } else {
                            self.input.len()
                        };

                        let value = self.input[actual_byte_idx_start..actual_byte_idx_end].to_string();
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
                        
                        if is_float {
                            return Some(Token::FloatLiteral(FloatLiteralToken {
                                span,
                                value,
                            }));
                        } else {
                            return Some(Token::NumberLiteral(NumberLiteralToken {
                                span,
                                value,
                            }));
                        }
                    }

                    // キーワードか識別子の判定
                    if self.is_identifier_start(ch) {
                        let actual_byte_idx_start = byte_idx;
                        self.next_char();

                        // 識別子の終わりまで読み進める
                        loop {
                            match self.now {
                                Some((_, next_ch)) if self.is_identifier_start(next_ch) || next_ch.is_ascii_digit() => {
                                    self.next_char();
                                }
                                _ => break,
                            }
                        }
                        
                        // 最後に読んだ文字の次のバイト位置を終端とする
                        let actual_byte_idx_end = if let Some((b, _)) = self.now {
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
                    } else {
                        // 識別子でない文字は無視して次へ
                        self.next_char();
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
