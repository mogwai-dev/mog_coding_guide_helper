use std::{fs, str::CharIndices};


enum Token {
    
    BlockComment{
        start_line: usize,      // 開始行 0 始まり
        start_column: usize,    // 開始列 0 始まり
        end_line: usize,        // 終了行 0 始まり
        end_column: usize,      // column は半開区間 (左閉右開)
        byte_offset: usize,     // バイトオフセット
        byte_length: usize      // バイト長
    },
    Include{
        start_line: usize,   // 開始行 0 始まり
        start_column: usize, // 開始列 0 始まり
        end_line: usize,     // 終了行 0 始まり
        end_column: usize,   // column は半開区間 (左閉右開)
        byte_offset: usize,  // バイトオフセット
        byte_end_idx: usize, // バイト終了インデックス
        filename: String     // ファイル名
    },
    Define{
        start_line: usize,      // 開始行 0 始まり
        start_column: usize,    // 開始列 0 始まり
        end_line: usize,        // 終了行 0 始まり
        end_column: usize,      // column は半開区間 (左閉右開)
        byte_offset: usize,     // バイトオフセット
        byte_end_idx: usize,    // バイト終了インデックス
        macro_name: String,      // マクロ名
        macro_value: String      // マクロ値
    },
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

        // 現在の文字インデックス（次に読む文字のインデックス）を開始位置として記録
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
                                        start_line,
                                        start_column,
                                        end_line,
                                        end_column,
                                        byte_offset: start_byte_flag.unwrap(),
                                        byte_length: end_byte,
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

                    // バイトオフセットを計算
                    let end_char_idx = self.pos_index();
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
                            byte_offset: start_byte_flag.unwrap(),
                            byte_end_idx: end_char_idx,
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
                                byte_offset: start_byte_flag.unwrap(),
                                byte_end_idx: end_char_idx,
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
                        byte_offset: start_byte_flag.unwrap(),
                        byte_end_idx: end_char_idx,
                        filename: content,
                    });
                },
                None => return None, // 入力の終わり
                _ => return None, // 他のトークン処理へ（ここでは省略）
            }
        }
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
    // 追加：Define ノード（span と生テキスト、それと分離したマクロ名/展開値を保持）
    Define { span: Span, text: String, macro_name: String, macro_value: String },
}

// ルートとノードを定義。所有する Span を持たせる（ライフタイム回避のため String/span を所有）
#[derive(Debug, Clone)]
pub struct Span {
    start_line: usize,
    start_column: usize,
    end_line: usize,
    end_column: usize,
    pub offset: usize, // あとあとで使うかも
    pub length: usize, // あとあとで使うかも
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
                Token::BlockComment { start_line, start_column, end_line, end_column, byte_offset, byte_length: byte_end_idx } => {
                    let span = Span {
                        start_line,
                        start_column,
                        end_line,
                        end_column,
                        offset: byte_offset,
                        length: byte_end_idx.saturating_sub(byte_offset),
                    };
                    // ここでは byte_offset をバイトオフセット、char_length をバイト長扱いでスライスしている既存のコードに合わせる
                    let text = self.lexer.input[byte_offset..byte_end_idx].to_string();
                    items.push(Item::BlockComment { span, text });
                },
                Token::Include { start_line, start_column, end_line, end_column, byte_offset, byte_end_idx, filename } => {
                    let span = Span {
                        start_line,
                        start_column,
                        end_line,
                        end_column,
                        offset: byte_offset,
                        length: byte_end_idx.saturating_sub(byte_offset),
                    };
                    let text = self.lexer.input[byte_offset..byte_end_idx].to_string();
                    items.push(Item::Include { span, text, filename });
                },
                Token::Define { start_line, start_column, end_line, end_column, byte_offset, byte_end_idx, macro_name, macro_value } => {
                    let span = Span {
                        start_line,
                        start_column,
                        end_line,
                        end_column,
                        offset: byte_offset,
                        length: byte_end_idx.saturating_sub(byte_offset),
                    };
                    let text = self.lexer.input[byte_offset..byte_end_idx].to_string();
                    items.push(Item::Define { span, text, macro_name, macro_value });
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
            Token::BlockComment { start_line, start_column, end_line, end_column , byte_offset, byte_length: byte_end_idx} => {
                println!("Block comment from ({}, {}) to ({}, {}): {}", start_line, start_column, end_line, end_column, &contents[byte_offset..byte_offset+byte_end_idx]);
            },
            Token::Include { start_line, start_column, end_line, end_column, byte_offset, byte_end_idx, filename } => {
                println!("Include from ({}, {}) to ({}, {}): {} (filename: {})", start_line, start_column, end_line, end_column, &contents[byte_offset..byte_offset+byte_end_idx], filename);
            },
            Token::Define { start_line, start_column, end_line, end_column, byte_offset, byte_end_idx, macro_name, macro_value } => {
                println!("Define from ({}, {}) to ({}, {}): {} (macro: {}, value: {})", start_line, start_column, end_line, end_column, &contents[byte_offset..byte_offset+byte_end_idx], macro_name, macro_value);
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
                Token::BlockComment { start_line, start_column, end_line, end_column , byte_offset, byte_length} => {
                    assert_eq!(start_line, 0);
                    assert_eq!(start_column, 0);
                    assert_eq!(end_line, 0);
                    assert_eq!(end_column, 13);
                    assert_eq!(byte_offset, 0);
                    assert_eq!(&s[byte_offset..byte_offset+byte_length], "/* comment */");
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
                Token::BlockComment { start_line, start_column, end_line, end_column , byte_offset, byte_length: byte_end_idx} => {
                    assert_eq!(start_line, 0);
                    assert_eq!(start_column, 0);
                    assert_eq!(end_line, 0);
                    assert_eq!(end_column, 10);
                    assert_eq!(byte_offset, 0);
                    assert_eq!(&s[byte_offset..byte_offset+byte_end_idx], "/* コメント */");
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
                Token::BlockComment { start_line, start_column, end_line, end_column , byte_offset, byte_length: byte_end_idx} => {
                    assert_eq!(start_line, 0);
                    assert_eq!(start_column, 0);
                    assert_eq!(end_line, 1);
                    assert_eq!(end_column, 10);
                    assert_eq!(byte_offset, 0);
                    assert_eq!(&s[byte_offset..byte_offset+byte_end_idx], "\t\r\n /* コメント*/");
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
                Token::Include { start_line, start_column, end_line, end_column, byte_offset, byte_end_idx, filename } => {
                    assert_eq!(start_line, 0);
                    assert_eq!(start_column, 0);
                    assert_eq!(end_line, 1);
                    assert_eq!(end_column, 0);
                    assert_eq!(filename, "stdio.h");
                    assert_eq!(byte_offset, 0);
                    assert_eq!(byte_end_idx, s.len());
                    assert_eq!(&s[byte_offset..byte_end_idx], "#include <stdio.h>\n");
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
                Token::Include { filename, byte_offset, byte_end_idx, .. } => {
                    assert_eq!(filename, "file.h");
                    assert_eq!(&s[byte_offset..byte_end_idx], "#include \"file.h\"\n");
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
                Token::Include { filename, byte_offset, byte_end_idx, .. } => {
                    assert_eq!(filename, "path/to/file");
                    assert_eq!(&s1[byte_offset..byte_offset+byte_end_idx], "#include <path/to/file\n");
                    break;
                }
                _ => {}
            }
        }

        let s2 = "#include \"another/path\n";
        let mut lx2 = Lexer::new(s2);
        while let Some(token) = lx2.next_token() {
            match token {
                Token::Include { filename, byte_offset, byte_end_idx, .. } => {
                    assert_eq!(filename, "another/path");
                    assert_eq!(&s2[byte_offset..byte_offset+byte_end_idx], "#include \"another/path\n");
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
                Token::Define { macro_name, macro_value, byte_offset, byte_end_idx, .. } => {
                    assert_eq!(macro_name, "MAX");
                    assert_eq!(macro_value, "10");
                    assert_eq!(&s[byte_offset..byte_offset+byte_end_idx], "#define MAX 10\n");
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
                Token::Define { macro_name, macro_value, byte_offset, byte_end_idx, .. } => {
                    assert_eq!(macro_name, "X");
                    assert_eq!(macro_value, "1");
                    assert_eq!(&s[byte_offset..byte_offset+byte_end_idx], "\t \r #define X 1\n");
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
