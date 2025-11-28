use std::{arch::x86_64::_MM_FROUND_CEIL, fs};


enum Token {
    BlockComment{start_line: usize, start_column: usize, end_line: usize, end_column: usize, offset: usize, length: usize},
    
}

#[derive(Debug)]
struct Lexer<'a> {
    input: &'a str,
    char_offsets: Vec<usize>, // 各文字のバイト開始位置
    cur: usize,               // 次に読む文字のインデックス (0..=len)
    column: usize,
    line: usize,
}

// Lexer の実装
// 文字列をトークンに分ける
impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        let char_offsets: Vec<usize> = input.char_indices().map(|(i, _)| i).collect();
        let mut lx = Lexer {
            input,
            char_offsets,
            cur: 0,
            column: 0,
            line: 0,
        };
        lx.update_pos();
        lx
    }

    // cur に基づいて line と column を更新する
    fn update_pos(&mut self) {
        let byte_off = if self.cur < self.char_offsets.len() {
            self.char_offsets[self.cur]
        } else {
            self.input.len()
        };
        let prefix = &self.input[..byte_off];
        self.line = prefix.chars().filter(|&c| c == '\n').count();
        self.column = prefix.rsplit('\n').next().map(|s| s.chars().count()).unwrap_or(0);
    }

    // 先に進めて文字を返す（存在しなければ None）
    fn next_char(&mut self) -> Option<char> {
        if self.cur >= self.char_offsets.len() {
            return None;
        }
        let b = self.char_offsets[self.cur];
        let ch = self.input[b..].chars().next().unwrap();
        self.cur += 1;
        self.update_pos();
        Some(ch)
    }

    // 一つ戻ってその文字を返す（存在しなければ None）
    fn prev_char(&mut self) -> Option<char> {
        if self.cur == 0 {
            return None;
        }
        self.cur -= 1;
        let b = self.char_offsets[self.cur];
        let ch = self.input[b..].chars().next().unwrap();
        self.update_pos();
        Some(ch)
    }

    // 次に読む文字を参照する（位置を変えない）
    fn peek(&self) -> Option<char> {
        if self.cur >= self.char_offsets.len() {
            None
        } else {
            let b = self.char_offsets[self.cur];
            Some(self.input[b..].chars().next().unwrap())
        }
    }

    fn len_chars(&self) -> usize {
        self.char_offsets.len()
    }

    fn pos_index(&self) -> usize {
        self.cur
    }

    // トークンを一つ読み取る
    fn next_token(&mut self) -> Option<Token> {

        let mut ret = None;

        // 現在の文字インデックス（次に読む文字のインデックス）を開始位置として記録
        let start_char_idx = self.pos_index();
        let start_line = self.line;
        let start_column = self.column;

        loop {
            match self.next_char() {
                Some(' ') | Some('\t') | Some('\n') | Some('\r') => {
                    // 空白文字はスキップ
                    continue;
                },
                Some('/') => {
                    if let Some('*') = self.peek() {
                        // ブロックコメントの開始
                        // '*' を消費
                        self.next_char();

                        let mut found = false;
    
                        // コメントの終わりを探す
                        while let Some(ch) = self.next_char() {
                            if ch == '*' {
                                if let Some('/') = self.peek() {
                                    // '/' を消費してコメントを終える
                                    self.next_char();
    
                                    // start_byte を char_offsets から取り出す
                                    let start_byte = if start_char_idx < self.char_offsets.len() {
                                        self.char_offsets[start_char_idx]
                                    } else {
                                        self.input.len()
                                    };
    
                                    // end_byte は現在の self.cur のバイトオフセット（self.cur は次に読む文字のインデックス）
                                    let end_byte = if self.cur < self.char_offsets.len() {
                                        self.char_offsets[self.cur]
                                    } else {
                                        self.input.len()
                                    };
    
                                    let length = end_byte.saturating_sub(start_byte);
    
                                    let end_line = self.line;
                                    let end_column = self.column;
                                    ret = Some(Token::BlockComment {
                                        start_line,
                                        start_column,
                                        end_line,
                                        end_column,
                                        offset: start_byte,
                                        length,
                                    });
                                    found = true;
                                    break;
                                }
                            }
                        }
                        
                        // コメントが閉じられなかった場合は None を返す
                        if !found {
                            ret = None;
                        }

                    } else {
                        // 他のトークン処理へ（ここでは省略）
                        return None
                    }
                },
                None => break,
                _ => break, // 他のトークン処理へ（ここでは省略）
            }
        }

        ret
    }
}


#[derive(Debug)]
struct TranslationUnit {
    items: Vec<Item>,
}

#[derive(Debug)]
enum Item {
    BlockComment { span: Span, text: String },
}

// ルートとノードを定義。所有する Span を持たせる（ライフタイム回避のため String/span を所有）
#[derive(Debug, Clone)]
struct Span {
    start_line: usize,
    start_column: usize,
    end_line: usize,
    end_column: usize,
    offset: usize,
    length: usize,
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
                Token::BlockComment { start_line, start_column, end_line, end_column , offset, length} => {
                    let span = Span {
                        start_line,
                        start_column,
                        end_line,
                        end_column,
                        offset,
                        length,
                    };
                    let text = self.lexer.input[offset..offset+length].to_string();
                    items.push(Item::BlockComment { span, text });
                }
            }
        }

        TranslationUnit { items }
    
    }
}

#[derive(Debug)]
struct Formatter {

}

impl Formatter {
    fn new() -> Self {
        Formatter {

        }
    }

    fn format_tu(&self, tu: &TranslationUnit) -> String {
        // フォーマットロジックをここに実装
        String::new()
    }

    // AST から元のコードを再構築
    fn original_tu(&self, tu: &TranslationUnit) -> String {
        // 元のコードを再構築するロジックをここに実装
        String::new()
    }
}

fn main() {
    lexer_sample();
    parser_sample();
}

fn lexer_sample() {

    println!("Lexer Sample:");
    let contents = fs::read_to_string("example.txt").unwrap();
    let mut lx = Lexer::new(&contents);
    

    while let Some(token) = lx.next_token() {
        match token {
            Token::BlockComment { start_line, start_column, end_line, end_column , offset, length} => {
                println!("Block comment from ({}, {}) to ({}, {}): {}", start_line, start_column, end_line, end_column, &contents[offset..offset+length]);
            }
        }
    }
}

fn parser_sample() {
    println!("\nParser Sample:");
    let contents = fs::read_to_string("example.txt").unwrap();
    let lx = Lexer::new(&contents);
    let mut parser = Parser::new(lx);
    let tu = parser.parse();

    for item in tu.items {
        match item {
            Item::BlockComment { span, text  } => {
                println!("Block comment from ({}, {}) to ({}, {}): {} ", span.start_line, span.start_column, span.end_line, span.end_column, text);
            }
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

        assert_eq!(lx.len_chars(), 4);
        assert_eq!(lx.peek(), Some('a'));

        // read 'a'
        assert_eq!(lx.next_char(), Some('a'));
        assert_eq!(lx.pos_index(), 1);
        assert_eq!(lx.line, 0);
        assert_eq!(lx.column, 1);

        // read 'b'
        assert_eq!(lx.next_char(), Some('b'));
        assert_eq!(lx.pos_index(), 2);
        assert_eq!(lx.line, 0);
        assert_eq!(lx.column, 2);

        // read '\n'
        assert_eq!(lx.next_char(), Some('\n'));
        assert_eq!(lx.pos_index(), 3);
        assert_eq!(lx.line, 1);
        assert_eq!(lx.column, 0);

        // go back one (to '\n')
        assert_eq!(lx.prev_char(), Some('\n'));
        assert_eq!(lx.pos_index(), 2);
        assert_eq!(lx.line, 0);
        assert_eq!(lx.column, 2);

    }

    #[test]
    fn test_multibyte_chars() {
        // 'é' is multibyte in UTF-8
        let s = "aéb";
        let mut lx = Lexer::new(s);
        assert_eq!(lx.len_chars(), 3);

        let mut got = Vec::new();
        while let Some(ch) = lx.next_char() {
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
                Token::BlockComment { start_line, start_column, end_line, end_column , offset, length} => {
                    assert_eq!(start_line, 0);
                    assert_eq!(start_column, 0);
                    assert_eq!(end_line, 0);
                    assert_eq!(end_column, 13);
                    assert_eq!(offset, 0);
                    assert_eq!(length, 13);
                    assert_eq!(&s[offset..offset+length], "/* comment */");
                    return;
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
                Token::BlockComment { start_line, start_column, end_line, end_column , offset, length} => {
                    assert_eq!(start_line, 0);
                    assert_eq!(start_column, 0);
                    assert_eq!(end_line, 0);
                    assert_eq!(end_column, 10);
                    assert_eq!(offset, 0);
                    assert_eq!(length, 18);
                    assert_eq!(&s[offset..offset+length], "/* コメント */");
                    return;
                }
            }
        }
        panic!("Block comment token not found");
    }

        #[test]
    fn test_lexer_block_comment_japanese_with_spaces() {
        let s = "\t\r\n /* コメント */";
        let mut lx = Lexer::new(s);

        // Skip to the block comment
        while let Some(token) = lx.next_token() {
            match token {
                Token::BlockComment { start_line, start_column, end_line, end_column , offset, length} => {
                    assert_eq!(start_line, 0);
                    assert_eq!(start_column, 0);
                    assert_eq!(end_line, 1);
                    assert_eq!(end_column, 11);
                    assert_eq!(offset, 0);
                    assert_eq!(length, 22);
                    assert_eq!(&s[offset..offset+length], "\t\r\n /* コメント */");
                    return;
                }
            }
        }
        panic!("Block comment token not found");
    }

}
