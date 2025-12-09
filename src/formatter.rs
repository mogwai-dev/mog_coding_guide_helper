use crate::ast::{TranslationUnit, Item};

const FILE_HEADER_TEMPLATE: &str = "/*****************************/
/* Author:                   */
/* Date:                     */
/* Purpose:                  */
/*****************************/\n\n";

#[derive(Debug)]
pub struct Formatter {
    pub add_header: bool,  // ヘッダーコメントを追加するかどうか
}

impl Formatter {
    pub fn new() -> Self {
        Formatter {
            add_header: true,  // デフォルトはtrue
        }
    }
    
    pub fn new_no_header() -> Self {
        Formatter {
            add_header: false,
        }
    }

    pub fn format_tu(&self, tu: &TranslationUnit) -> String {
        let mut s = String::new();
        
        // ヘッダーコメントがあるかチェック
        let has_header = self.has_file_header(tu);
        
        // add_headerがtrueかつヘッダーコメントがなければ追加
        if self.add_header && !has_header {
            s.push_str(FILE_HEADER_TEMPLATE);
        }
        
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
                // Stage 1: 条件コンパイルブロック
                Item::ConditionalBlock { directive_type, condition, items, .. } => {
                    // #ifdef CONDITION
                    s.push('#');
                    s.push_str(directive_type);
                    if !condition.is_empty() {
                        s.push(' ');
                        s.push_str(condition);
                    }
                    s.push('\n');
                    
                    // ブロック内のアイテムを再帰的にフォーマット
                    for inner_item in items {
                        s.push_str(&self.format_item(inner_item));
                    }
                    
                    // #endif
                    s.push_str("#endif\n");
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
                },
                Item::FunctionDecl { text, .. } => {
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
                Item::EnumDecl { text, .. } => {
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
                Item::UnionDecl { text, .. } => {
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

    // 個別のアイテムをフォーマット（再帰用）
    fn format_item(&self, item: &Item) -> String {
        let mut s = String::new();
        match item {
            Item::BlockComment { text, .. } => {
                let first_non_ws = text
                    .char_indices()
                    .find(|&(_, ch)| !ch.is_whitespace())
                    .map(|(i, _)| i)
                    .unwrap_or(text.len());
                let leading = &text[..first_non_ws];
                let kept_newlines: String = leading.chars().filter(|&c| c == '\n').collect();
                s.push_str(&kept_newlines);
                s.push_str(&text[first_non_ws..]);
            },
            Item::Include { text, ..} => {
                let first_non_ws = text
                    .char_indices()
                    .find(|&(_, ch)| !ch.is_whitespace())
                    .map(|(i, _)| i)
                    .unwrap_or(text.len());
                let leading = &text[..first_non_ws];
                let kept_newlines: String = leading.chars().filter(|&c| c == '\n').collect();
                s.push_str(&kept_newlines);
                s.push_str(&text[first_non_ws..]);       
            },
            Item::Define { text, .. } => {
                let first_non_ws = text
                    .char_indices()
                    .find(|&(_, ch)| !ch.is_whitespace())
                    .map(|(i, _)| i)
                    .unwrap_or(text.len());
                let leading = &text[..first_non_ws];
                let kept_newlines: String = leading.chars().filter(|&c| c == '\n').collect();
                s.push_str(&kept_newlines);
                s.push_str(&text[first_non_ws..]);
            },
            Item::ConditionalBlock { directive_type, condition, items, .. } => {
                s.push('#');
                s.push_str(directive_type);
                if !condition.is_empty() {
                    s.push(' ');
                    s.push_str(condition);
                }
                s.push('\n');
                for inner_item in items {
                    s.push_str(&self.format_item(inner_item));
                }
                s.push_str("#endif\n");
            },
            Item::TypedefDecl { text, .. } | Item::VarDecl { text, .. } | 
            Item::StructDecl { text, .. } | Item::FunctionDecl { text, .. } | 
            Item::EnumDecl { text, .. } | Item::UnionDecl { text, .. } => {
                let first_non_ws = text
                    .char_indices()
                    .find(|&(_, ch)| !ch.is_whitespace())
                    .map(|(i, _)| i)
                    .unwrap_or(text.len());
                let leading = &text[..first_non_ws];
                let kept_newlines: String = leading.chars().filter(|&c| c == '\n').collect();
                s.push_str(&kept_newlines);
                s.push_str(&text[first_non_ws..]);
            },
        }
        s
    }

    // AST から元のコードを再構築
    pub fn original_tu(&self, tu: &TranslationUnit) -> String {
        let mut s = String::new();
        for item in &tu.items {
            s.push_str(&self.original_item(item));
        }
        s
    }

    // 個別のアイテムから元のコードを再構築
    fn original_item(&self, item: &Item) -> String {
        match item {
            Item::ConditionalBlock { items, start_span: _, end_span: _, .. } => {
                let mut s = String::new();
                // start_spanから#ifdefディレクティブを取得
                // TODO: lexer.inputへのアクセスが必要（現在は不可能）
                // 暫定: 再構築は諦めて、内部のitemsだけ返す
                for inner_item in items {
                    s.push_str(&self.original_item(inner_item));
                }
                s
            },
            Item::BlockComment { text, .. } | Item::Include { text, .. } | 
            Item::Define { text, .. } | Item::TypedefDecl { text, .. } |
            Item::VarDecl { text, .. } | Item::StructDecl { text, .. } |
            Item::FunctionDecl { text, .. } | Item::EnumDecl { text, .. } |
            Item::UnionDecl { text, .. } => {
                text.clone()
            }
        }
    }

    /// ファイルヘッダーコメントがあるかチェック
    fn has_file_header(&self, tu: &TranslationUnit) -> bool {
        // 最初の連続するブロックコメントを結合してチェック
        let mut combined_text = String::new();
        let mut found_comments = false;
        
        for item in &tu.items {
            match item {
                Item::BlockComment { text, .. } => {
                    combined_text.push_str(text);
                    found_comments = true;
                },
                _ => {
                    // ブロックコメント以外が出てきたら終了
                    break;
                }
            }
        }
        
        if !found_comments {
            return false;
        }
        
        // Author/Auther, Date, Purpose のいずれかが含まれていればヘッダーコメントとみなす
        let lower_text = combined_text.to_lowercase();
        (lower_text.contains("author") || lower_text.contains("auther")) 
            && lower_text.contains("date") 
            && lower_text.contains("purpose")
    }
}

