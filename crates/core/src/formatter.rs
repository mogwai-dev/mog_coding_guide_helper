use crate::ast::{TranslationUnit, Item};

const FILE_HEADER_TEMPLATE: &str = "/*****************************/
/* Author:                   */
/* Date:                     */
/* Purpose:                  */
/*****************************/\n\n";

#[derive(Debug)]
pub struct Formatter {
    pub add_header: bool,  // ヘッダーコメントを追加するかどうか
    pub use_type_info: bool,  // 型情報を使用して変数宣言をフォーマットするかどうか
    pub use_tabs: bool,  // 4スペースをタブに変換するかどうか
}

impl Formatter {
    pub fn new() -> Self {
        Formatter {
            add_header: true,  // デフォルトはtrue
            use_type_info: false,  // デフォルトは既存のテキストを使用
            use_tabs: false,  // デフォルトはスペースのまま
        }
    }
    
    pub fn new_no_header() -> Self {
        Formatter {
            add_header: false,
            use_type_info: false,
            use_tabs: false,
        }
    }
    
    pub fn new_with_type_info() -> Self {
        Formatter {
            add_header: true,
            use_type_info: true,
            use_tabs: false,
        }
    }
    
    pub fn new_with_options(add_header: bool, use_type_info: bool) -> Self {
        Formatter {
            add_header,
            use_type_info,
            use_tabs: false,
        }
    }
    
    pub fn new_with_all_options(add_header: bool, use_type_info: bool, use_tabs: bool) -> Self {
        Formatter {
            add_header,
            use_type_info,
            use_tabs,
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
        
        // ファイル先頭のコメント（leading_trivia）を出力
        self.format_trivia(&tu.leading_trivia, &mut s);
        
        for item in &tu.items {
            // アイテム前のコメント（leading trivia）を出力
            if let Some(trivia) = self.get_item_trivia(item) {
                self.format_trivia(trivia, &mut s);
            }
            
            match item {
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
                Item::VarDecl { text, var_name, var_type, has_initializer, .. } => {
                    if self.use_type_info && var_type.is_some() {
                        // 型情報を使用してフォーマット
                        let ty = var_type.as_ref().unwrap();
                        let type_str = ty.to_string();
                        
                        // 先頭の空白部分から改行だけ取り出して保持する
                        let first_non_ws = text
                            .char_indices()
                            .find(|&(_, ch)| !ch.is_whitespace())
                            .map(|(i, _)| i)
                            .unwrap_or(text.len());
                        let leading = &text[..first_non_ws];
                        let kept_newlines: String = leading.chars().filter(|&c| c == '\n').collect();
                        
                        s.push_str(&kept_newlines);
                        s.push_str(&type_str);
                        s.push_str(" ");
                        s.push_str(var_name);
                        
                        // 初期化子がある場合は元のテキストから抽出
                        if *has_initializer {
                            // 元のテキストから'='を見つけて、それ以降を含める
                            if let Some(eq_pos) = text.find('=') {
                                s.push_str(" ");
                                s.push_str(&text[eq_pos..].trim_end());
                            } else {
                                s.push_str(";");
                            }
                        } else {
                            s.push_str(";");
                        }
                        
                        // 改行が元のテキストの最後になければ追加
                        if !text.ends_with('\n') && !text.ends_with("\r\n") {
                            // 何もしない（改行なし）
                        } else {
                            s.push('\n');
                        }
                    } else {
                        // 既存の実装: テキストをそのまま使用
                        let first_non_ws = text
                            .char_indices()
                            .find(|&(_, ch)| !ch.is_whitespace())
                            .map(|(i, _)| i)
                            .unwrap_or(text.len());

                        let leading = &text[..first_non_ws];
                        let kept_newlines: String = leading.chars().filter(|&c| c == '\n').collect();

                        s.push_str(&kept_newlines);
                        s.push_str(&text[first_non_ws..]);
                    }
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

                    // インデント変換を適用して出力
                    let content = self.format_item_text(&text[first_non_ws..]);
                    s.push_str(&kept_newlines);
                    s.push_str(&content);
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
        
        // アイテム前のコメント（leading trivia）を出力
        if let Some(trivia) = self.get_item_trivia(item) {
            self.format_trivia(trivia, &mut s);
        }
        
        match item {
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
        
        // インデント変換を適用
        if self.use_tabs {
            s = self.convert_indents_to_tabs(&s);
        }
        
        s
    }
    
    /// 4スペースをタブに変換し、ブロック内の字下げを処理
    fn convert_indents_to_tabs(&self, text: &str) -> String {
        let mut result = String::new();
        
        for line in text.lines() {
            // 行頭の連続するスペースをカウント
            let leading_spaces = line.chars().take_while(|&c| c == ' ').count();
            
            if leading_spaces > 0 {
                // 4スペース = 1タブに変換
                let tabs = leading_spaces / 4;
                let remaining_spaces = leading_spaces % 4;
                
                // タブを追加
                for _ in 0..tabs {
                    result.push('\t');
                }
                // 余りのスペースを追加
                for _ in 0..remaining_spaces {
                    result.push(' ');
                }
                
                // 残りの行を追加
                result.push_str(&line[leading_spaces..]);
            } else {
                // インデントがない行はそのまま
                result.push_str(line);
            }
            
            result.push('\n');
        }
        
        // 最後の余分な改行を削除（元のテキストに合わせる）
        if !text.ends_with('\n') && result.ends_with('\n') {
            result.pop();
        }
        
        result
    }
    
    /// アイテムのテキストをフォーマット（インデント変換を適用）
    fn format_item_text(&self, text: &str) -> String {
        if self.use_tabs {
            self.convert_indents_to_tabs(text)
        } else {
            text.to_string()
        }
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
            Item::Include { text, .. } | Item::Define { text, .. } | 
            Item::TypedefDecl { text, .. } | Item::VarDecl { text, .. } | 
            Item::StructDecl { text, .. } | Item::FunctionDecl { text, .. } | 
            Item::EnumDecl { text, .. } | Item::UnionDecl { text, .. } => {
                text.clone()
            }
        }
    }

    /// ファイルヘッダーコメントがあるかチェック
    fn has_file_header(&self, tu: &TranslationUnit) -> bool {
        use crate::trivia::Comment;
        
        // leading_triviaからブロックコメントを結合してチェック
        let mut combined_text = String::new();
        
        for comment in &tu.leading_trivia.leading {
            if let Comment::Block { text, .. } = comment {
                combined_text.push_str(text);
            }
        }
        
        if combined_text.is_empty() {
            return false;
        }
        
        // Author/Auther, Date, Purpose のいずれかが含まれていればヘッダーコメントとみなす
        let lower_text = combined_text.to_lowercase();
        (lower_text.contains("author") || lower_text.contains("auther")) 
            && lower_text.contains("date") 
            && lower_text.contains("purpose")
    }

    /// Triviaをフォーマットして文字列に追加
    fn format_trivia(&self, trivia: &crate::trivia::Trivia, out: &mut String) {
        use crate::trivia::Comment;
        
        for comment in &trivia.leading {
            match comment {
                Comment::Line { text, .. } => {
                    out.push_str(text);
                    if !text.ends_with('\n') {
                        out.push('\n');
                    }
                },
                Comment::Block { text, .. } => {
                    out.push_str(text);
                }
            }
        }
    }

    /// Itemからtriviaを取得
    fn get_item_trivia<'a>(&self, item: &'a Item) -> Option<&'a crate::trivia::Trivia> {
        match item {
            Item::Include { trivia, .. } |
            Item::Define { trivia, .. } |
            Item::ConditionalBlock { trivia, .. } |
            Item::TypedefDecl { trivia, .. } |
            Item::VarDecl { trivia, .. } |
            Item::StructDecl { trivia, .. } |
            Item::EnumDecl { trivia, .. } |
            Item::UnionDecl { trivia, .. } |
            Item::FunctionDecl { trivia, .. } => Some(trivia),
        }
    }
}


