use std::fs;

enum Token {
    BLOCK_COMMENT{start_line: usize, start_column: usize, end_line: usize, end_column: usize },
}

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

    fn next_token(&mut self) -> Option<Token> {
        // トークン化のロジックをここに実装
        
        let start_column = self.column;

        match self.next_char() {
            Some('/') => {

                if let Some('*') = self.peek() {
                    // ブロックコメントの開始
                    let start_line = self.line;
                    self.next_char(); // '*' を消費
                    // コメントの終わりを探す
                    while let Some(ch) = self.next_char() {
                        if ch == '*' {
                            if let Some('/') = self.peek() {
                                self.next_char(); // '/' を消費
                                let end_line = self.line;
                                let end_column = self.column;
                                return Some(Token::BLOCK_COMMENT {
                                    start_line,
                                    start_column,
                                    end_line,
                                    end_column,
                                });
                            }
                        }
                    }
                    // コメントが閉じられなかった場合は None を返す
                    None
                } else {
                    // 他のトークン処理へ（ここでは省略）
                    None
                }
            },
            None => None,
            _ => None, // 他のトークン処理へ（ここでは省略）
        }
    }
}

fn main() {
    lexer_sample();
}

fn lexer_sample() {
    let contents = fs::read_to_string("example.txt").unwrap();
    let mut lx = Lexer::new(&contents);

    while let Some(token) = lx.next_token() {
        match token {
            Token::BLOCK_COMMENT { start_line, start_column, end_line, end_column } => {
                println!("Block comment from ({}, {}) to ({}, {})", start_line, start_column, end_line, end_column);
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
                Token::BLOCK_COMMENT { start_line, start_column, end_line, end_column } => {
                    assert_eq!(start_line, 0);
                    assert_eq!(start_column, 0);
                    assert_eq!(end_line, 0);
                    assert_eq!(end_column, 13);
                    return;
                }
            }
        }
        panic!("Block comment token not found");
    }

}
