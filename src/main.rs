use std::{fs};


enum Token {
    BlockComment{start_line: usize, start_column: usize, end_line: usize, end_column: usize, offset: usize, length: usize},
    Include{start_line: usize, start_column: usize, end_line: usize, end_column: usize, offset: usize, length: usize, filename: String},
    Define{start_line: usize, start_column: usize, end_line: usize, end_column: usize, offset: usize, length: usize, macro_name: String, macro_value: String},
}

#[derive(Debug)]
struct Lexer<'a> {
    input: &'a str,
    char_offsets: Vec<usize>, // 各文字のバイト開始位置
    cur: usize,               // 次に読む文字のインデックス (0..=len)
    column: usize,
    line: usize,
    chars: core::str::Chars<'a>,
    now: Option<char>,
    peeked: Option<char>,
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
            chars: input.chars(),
            now: None,
            peeked: None,
        };
        lx.next_char(); // 初期化のために一文字進める
        lx
    }

    // 先に進めて文字を返す（存在しなければ None）
    fn next_char(&mut self) -> Option<char> {

        self.now = self.peeked;
        self.peeked = self.chars.next();
 
        if let Some(ch) = self.now {
            self.cur += 1;
            if ch == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }
            return Some(ch);
        }
        None
    }

    // 次に読む文字を参照する（位置を変えない）
    fn peek(&self) -> Option<char> {
        self.peeked
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

                                    // ここで即座に返す
                                    return Some(Token::BlockComment {
                                        start_line,
                                        start_column,
                                        end_line,
                                        end_column,
                                        offset: start_byte,
                                        length,
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
                Some('#') => {
                    // ディレクティブを行末まで読み取る（先頭の空白は start_char_idx でカバーされる）
                    let mut directive_text = String::new();
                    directive_text.push('#');

                    while let Some(ch) = self.next_char() {
                        directive_text.push(ch);
                        if ch == '\n' {
                            break;
                        }
                    }

                    // directive の中身（# を取り除いた後）を解析用に取得（先頭空白は trim_start する）
                    let content = directive_text.trim_start_matches('#').trim_start().to_string();

                    // バイトオフセットを計算
                    let start_byte = if start_char_idx < self.char_offsets.len() {
                        self.char_offsets[start_char_idx]
                    } else {
                        self.input.len()
                    };
                    let end_byte = if self.cur < self.char_offsets.len() {
                        self.char_offsets[self.cur]
                    } else {
                        self.input.len()
                    };
                    let length = end_byte.saturating_sub(start_byte);
                    let end_line = self.line;
                    let end_column = self.column;

                    // #include の処理（既存の挙動を保持）
                    if let Some(rest) = content.strip_prefix("include") {
                        let rest = rest.trim();
                        let mut filename = rest.to_string();
                        if rest.starts_with('<') {
                            if let Some(end) = rest.find('>') {
                                if end > 1 {
                                    filename = rest[1..end].to_string();
                                } else {
                                    filename = String::new();
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
                            start_line,
                            start_column,
                            end_line,
                            end_column,
                            offset: start_byte,
                            length,
                            filename,
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
                                start_line,
                                start_column,
                                end_line,
                                end_column,
                                offset: start_byte,
                                length,
                                macro_name,
                                macro_value,
                            });
                        }
                    }

                    // それ以外の # 系ディレクティブはとりあえず Include 風に生テキストを残す（既存互換）
                    return Some(Token::Include {
                        start_line,
                        start_column,
                        end_line,
                        end_column,
                        offset: start_byte,
                        length,
                        filename: content,
                    });
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
    Include { span: Span, text: String, filename: String },
    // 追加：Define ノード（span と生テキスト、それと分離したマクロ名/展開値を保持）
    Define { span: Span, text: String, macro_name: String, macro_value: String },
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
                },
                Token::Include { start_line, start_column, end_line, end_column, offset, length, filename } => {
                    let span = Span {
                        start_line,
                        start_column,
                        end_line,
                        end_column,
                        offset,
                        length,
                    };
                    
                    let text = self.lexer.input[offset..offset+length].to_string();

                    items.push(Item::Include { span, text, filename });

                },
                Token::Define { start_line, start_column, end_line, end_column, offset, length, macro_name, macro_value } => {
                    let span = Span {
                        start_line,
                        start_column,
                        end_line,
                        end_column,
                        offset,
                        length,
                    };
                    let text = self.lexer.input[offset..offset+length].to_string();
                    items.push(Item::Define { span, text, macro_name, macro_value });
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
                Item::Include { span, text, filename } => {
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
                Item::Define { span, text, macro_name, macro_value } => {
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
    fn original_tu(&self, tu: &TranslationUnit) -> String {
        // 元のコードを再構築するロジックをここに実装
        let mut s = String::new();
        for item in &tu.items {
            match item {
                Item::BlockComment { text, .. } | Item::Include { text, .. } | Item::Define { text, .. }=> {
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

fn lexer_sample() {

    println!("Lexer Sample:");
    let contents = fs::read_to_string("example.txt").unwrap();
    let mut lx = Lexer::new(&contents);
    

    while let Some(token) = lx.next_token() {
        match token {
            Token::BlockComment { start_line, start_column, end_line, end_column , offset, length} => {
                println!("Block comment from ({}, {}) to ({}, {}): {}", start_line, start_column, end_line, end_column, &contents[offset..offset+length]);
            },
            Token::Include { start_line, start_column, end_line, end_column, offset, length, filename } => {
                println!("Include from ({}, {}) to ({}, {}): {} (filename: {})", start_line, start_column, end_line, end_column, &contents[offset..offset+length], filename);
            },
            Token::Define { start_line, start_column, end_line, end_column, offset, length, macro_name, macro_value } => {
                println!("Define from ({}, {}) to ({}, {}): {} (macro: {}, value: {})", start_line, start_column, end_line, end_column, &contents[offset..offset+length], macro_name, macro_value);
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
            },
            Item::Include { span, text, filename } => {
                println!("Include from ({}, {}) to ({}, {}): {} (filename: {})", span.start_line, span.start_column, span.end_line, span.end_column, text, filename);
            },
            Item::Define { span, text, macro_name, macro_value } => {
                println!("Define from ({}, {}) to ({}, {}): {} (macro: {}, value: {})", span.start_line, span.start_column, span.end_line, span.end_column, text, macro_name, macro_value);
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

    }

    #[test]
    fn test_multibyte_chars() {
        // 'é' is multibyte in UTF-8
        let s = "aéb";
        let mut lx = Lexer::new(s);

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
                _ => {
                    panic!("Unexpected token");
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
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, offset: 0, length: 0 };
        let item = Item::BlockComment { span, text: String::from("   /* hello */") };
        let tu = TranslationUnit { items: vec![item] };
        let fmt = Formatter::new();
        let out = fmt.format_tu(&tu);
        assert_eq!(out, "/* hello */");
    }

    #[test]
    fn test_formatter_original_tu_preserves_texts() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, offset: 0, length: 0 };
        let item1 = Item::BlockComment { span: span.clone(), text: String::from("/* one */") };
        let item2 = Item::BlockComment { span, text: String::from("/* two */") };
        let tu = TranslationUnit { items: vec![item1, item2] };
        let fmt = Formatter::new();
        let out = fmt.original_tu(&tu);
        assert_eq!(out, "/* one *//* two */");
    }

    #[test]
    fn test_formatter_keeps_newline_in_leading_whitespace() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, offset: 0, length: 0 };
        let item = Item::BlockComment { span, text: String::from("\t\r\n /* hello */") };
        let tu = TranslationUnit { items: vec![item] };
        let fmt = Formatter::new();
        let out = fmt.format_tu(&tu);
        assert_eq!(out, "\n/* hello */");
    }

    #[test]
    fn test_formatter_keeps_multiple_newlines() {
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, offset: 0, length: 0 };
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
                Token::Include { start_line, start_column, end_line, end_column, offset, length, filename } => {
                    assert_eq!(start_line, 0);
                    assert_eq!(start_column, 0);
                    assert_eq!(filename, "stdio.h");
                    assert_eq!(offset, 0);
                    assert_eq!(length, s.len());
                    assert_eq!(&s[offset..offset+length], "#include <stdio.h>\n");
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
                Token::Include { filename, offset, length, .. } => {
                    assert_eq!(filename, "file.h");
                    assert_eq!(&s[offset..offset+length], "#include \"file.h\"\n");
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
                Token::Include { filename, offset, length, .. } => {
                    assert_eq!(filename, "path/to/file");
                    assert_eq!(&s1[offset..offset+length], "#include <path/to/file\n");
                    break;
                }
                _ => {}
            }
        }

        let s2 = "#include \"another/path\n";
        let mut lx2 = Lexer::new(s2);
        while let Some(token) = lx2.next_token() {
            match token {
                Token::Include { filename, offset, length, .. } => {
                    assert_eq!(filename, "another/path");
                    assert_eq!(&s2[offset..offset+length], "#include \"another/path\n");
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
                Token::Define { macro_name, macro_value, offset, length, .. } => {
                    assert_eq!(macro_name, "MAX");
                    assert_eq!(macro_value, "10");
                    assert_eq!(&s[offset..offset+length], "#define MAX 10\n");
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
                Token::Define { macro_name, macro_value, offset, length, .. } => {
                    // offset should include leading whitespace (lexer records start before skipping)
                    assert_eq!(macro_name, "X");
                    assert_eq!(macro_value, "1");
                    assert_eq!(&s[offset..offset+length], "\t \r #define X 1\n");
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
        let span = Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, offset: 0, length: 0 };
        let text = String::from("\t\r\n  #define Z 42\n");
        let item = Item::Define { span, text: text.clone(), macro_name: "Z".into(), macro_value: "42".into() };
        let tu = TranslationUnit { items: vec![item] };
        let fmt = Formatter::new();
        let out = fmt.format_tu(&tu);
        // leading \t\r and spaces removed, newline kept, then the rest starts with '#'
        assert_eq!(out, "\n#define Z 42\n");
    }
}
