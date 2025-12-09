use crate::ast::{TranslationUnit, Item};

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
}

