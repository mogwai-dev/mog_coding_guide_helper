use crate::ast::{TranslationUnit, Item};
use crate::span::Span;
use crate::type_system::BaseType;

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub span: Span,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub code: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Debug, Clone)]
pub struct DiagnosticConfig {
    pub check_file_header: bool,
    pub check_storage_class_order: bool,
    pub check_function_format: bool,
    pub check_type_safety: bool,
    pub check_macro_parentheses: bool,  // マクロ定義の値が括弧で囲まれているかチェック
}

impl Default for DiagnosticConfig {
    fn default() -> Self {
        DiagnosticConfig {
            check_file_header: true,
            check_storage_class_order: true,
            check_function_format: true,
            check_type_safety: true,
            check_macro_parentheses: true,
        }
    }
}

/// TranslationUnitに対して診断を実行
pub fn diagnose(tu: &TranslationUnit, config: &DiagnosticConfig) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    
    if config.check_file_header {
        if let Some(diag) = check_file_header(tu) {
            diagnostics.push(diag);
        }
    }
    
    if config.check_function_format {
        diagnostics.extend(check_function_format(tu));
    }
    
    if config.check_type_safety {
        diagnostics.extend(check_type_safety(tu));
    }
    
    if config.check_macro_parentheses {
        diagnostics.extend(check_macro_parentheses(tu));
    }
    
    // 今後、他のチェックもここに追加
    
    diagnostics
}

/// ファイルヘッダーコメントの存在をチェック
fn check_file_header(tu: &TranslationUnit) -> Option<Diagnostic> {
    use crate::trivia::Comment;
    
    // leading_triviaからブロックコメントを結合してチェック
    let mut combined_text = String::new();
    let mut first_span: Option<Span> = None;
    
    for comment in &tu.leading_trivia.leading {
        if let Comment::Block { text, span } = comment {
            if first_span.is_none() {
                first_span = Some(span.clone());
            }
            combined_text.push_str(text);
        }
    }
    
    // ヘッダーコメントがあるかチェック
    let has_header = if !combined_text.is_empty() {
        let lower_text = combined_text.to_lowercase();
        (lower_text.contains("author") || lower_text.contains("auther")) 
            && lower_text.contains("date") 
            && lower_text.contains("purpose")
    } else {
        false
    };
    
    if !has_header {
        // ヘッダーがない場合は警告を出す
        // spanは(0,0)から(0,0)とする（ファイル先頭）
        let span = first_span.unwrap_or(Span {
            start_line: 0,
            start_column: 0,
            end_line: 0,
            end_column: 0,
            byte_start_idx: 0,
            byte_end_idx: 0,
        });
        
        return Some(Diagnostic {
            span,
            severity: DiagnosticSeverity::Warning,
            message: "File header comment is missing. Expected comment block with Author, Date, and Purpose fields.".to_string(),
            code: "CGH001".to_string(),
        });
    }
    
    None
}

/// 関数定義のフォーマットをチェック
/// 戻り値・修飾子が1行、関数名と引数が1行、開き括弧が1行に分かれているか確認
fn check_function_format(tu: &TranslationUnit) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    
    for item in &tu.items {
        if let Item::FunctionDecl { span, text, function_name, return_type, storage_class, .. } = item {
            // textの内容を行ごとに分割して解析
            // まず、行コメントを除去してから解析
            let clean_text: String = text.lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    // 行コメントのみの行は除外
                    !trimmed.starts_with("//")
                })
                .collect::<Vec<_>>()
                .join("\n");
            
            let lines: Vec<&str> = clean_text.lines()
                .filter(|line| !line.trim().is_empty()) // 空行を除外
                .collect();
            
            if lines.is_empty() {
                continue;
            }
            
            // 関数名が含まれる行を探す
            let mut function_line_idx = None;
            for (i, line) in lines.iter().enumerate() {
                if line.contains(function_name) {
                    function_line_idx = Some(i);
                    break;
                }
            }
            
            if let Some(fn_idx) = function_line_idx {
                let fn_line = lines[fn_idx];
                
                // チェック1: 関数名の行に開き括弧 '{' が含まれているか
                if fn_line.contains('{') {
                    diagnostics.push(Diagnostic {
                        span: span.clone(),
                        severity: DiagnosticSeverity::Warning,
                        message: format!(
                            "Function '{}' opening brace should be on a separate line.",
                            function_name
                        ),
                        code: "CGH002".to_string(),
                    });
                }
                
                // チェック2: 戻り値の型と関数名が同じ行にあるか
                // 戻り値の型（return_type）が関数名と同じ行にあるかチェック
                let has_return_type_on_same_line = if let Some(storage) = storage_class {
                    // storage classがある場合、return_typeかstorageのいずれかが同じ行にあるかチェック
                    fn_line.contains(return_type) || fn_line.contains(storage)
                } else {
                    fn_line.contains(return_type)
                };
                
                if has_return_type_on_same_line {
                    diagnostics.push(Diagnostic {
                        span: span.clone(),
                        severity: DiagnosticSeverity::Warning,
                        message: format!(
                            "Function '{}' name should be on a separate line from the return type.",
                            function_name
                        ),
                        code: "CGH002".to_string(),
                    });
                }
            }
        }
    }
    
    diagnostics
}

/// 型の安全性をチェック
fn check_type_safety(tu: &TranslationUnit) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    
    for item in &tu.items {
        if let Item::VarDecl { span, var_name, var_type, text, .. } = item {
            if let Some(ty) = var_type {
                // チェック1: void型の変数宣言（void*は除く）
                if ty.base_type == BaseType::Void && !ty.is_pointer() {
                    diagnostics.push(Diagnostic {
                        span: span.clone(),
                        severity: DiagnosticSeverity::Error,
                        message: format!(
                            "変数 '{}' はvoid型にできません。voidポインタは 'void *' を使用してください。",
                            var_name
                        ),
                        code: "CGH101".to_string(),
                    });
                }
                
                // チェック2: ポインタのポインタのポインタ（***以上）の使用を警告
                if ty.pointer_level() >= 3 {
                    diagnostics.push(Diagnostic {
                        span: span.clone(),
                        severity: DiagnosticSeverity::Warning,
                        message: format!(
                            "変数 '{}' は{}段階のポインタです。設計を簡素化することを検討してください。",
                            var_name,
                            ty.pointer_level()
                        ),
                        code: "CGH102".to_string(),
                    });
                }
                
                // チェック3: 型情報がテキストと一致しているか簡易確認
                // テキストにアスタリスクがあるのにポインタでない場合を検出
                let asterisk_count = text.chars().filter(|&c| c == '*').count();
                if asterisk_count > 0 && !ty.is_pointer() {
                    diagnostics.push(Diagnostic {
                        span: span.clone(),
                        severity: DiagnosticSeverity::Warning,
                        message: format!(
                            "変数 '{}' の宣言に '*' が含まれていますが、型情報は非ポインタ型を示しています。型解析が失敗した可能性があります。",
                            var_name
                        ),
                        code: "CGH103".to_string(),
                    });
                }
                
                // チェック4: テキストのアスタリスク数とポインタレベルの不一致
                if ty.is_pointer() && asterisk_count > 0 && asterisk_count != ty.pointer_level() {
                    diagnostics.push(Diagnostic {
                        span: span.clone(),
                        severity: DiagnosticSeverity::Information,
                        message: format!(
                            "変数 '{}' には{}個のアスタリスクがありますが、{}段階のポインタとして解析されました。複雑なポインタ構文の可能性があります。",
                            var_name,
                            asterisk_count,
                            ty.pointer_level()
                        ),
                        code: "CGH104".to_string(),
                    });
                }
            }
        }
    }
    
    diagnostics
}

/// #defineマクロの置換値が括弧で囲まれているかチェック
fn check_macro_parentheses(tu: &TranslationUnit) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    
    fn check_items(items: &[Item], diagnostics: &mut Vec<Diagnostic>) {
        for item in items {
            match item {
                Item::Define { text, span, .. } => {
                    // #define MACRO value の形式から value 部分を抽出
                    // テキストから "#define" を除去し、マクロ名と値を取得
                    let content = text.trim();
                    
                    // #define を除去
                    if let Some(after_define) = content.strip_prefix("#define") {
                        let parts: Vec<&str> = after_define.trim().splitn(2, char::is_whitespace).collect();
                        
                        if parts.len() == 2 {
                            let macro_name = parts[0];
                            let macro_value = parts[1].trim();
                            
                            // 関数マクロ（括弧を含む名前）はスキップ
                            if macro_name.contains('(') {
                                continue;
                            }
                            
                            // 空の値や数値リテラルのみはスキップ
                            if macro_value.is_empty() || is_simple_literal(macro_value) {
                                continue;
                            }
                            
                            // 演算子を含むかチェック
                            if contains_operator(macro_value) {
                                // 括弧で囲まれているかチェック
                                if !is_wrapped_in_parentheses(macro_value) {
                                    diagnostics.push(Diagnostic {
                                        span: span.clone(),
                                        severity: DiagnosticSeverity::Warning,
                                        message: format!(
                                            "マクロ '{}' の置換値 '{}' は演算子を含んでいますが、括弧で囲まれていません。意図しない演算子の優先順位問題を避けるため、括弧で囲むことを推奨します。",
                                            macro_name,
                                            macro_value
                                        ),
                                        code: "CGH005".to_string(),
                                    });
                                }
                            }
                        }
                    }
                },
                Item::ConditionalBlock { items, .. } => {
                    // 再帰的にチェック
                    check_items(items, diagnostics);
                },
                _ => {}
            }
        }
    }
    
    check_items(&tu.items, &mut diagnostics);
    diagnostics
}

/// 単純なリテラル（数値、文字列）かどうか判定
fn is_simple_literal(value: &str) -> bool {
    let trimmed = value.trim();
    
    // 空白を含む場合は複雑な式とみなす
    if trimmed.contains(char::is_whitespace) {
        return false;
    }
    
    // 数値リテラル（10進数、16進数、8進数、浮動小数点）
    if trimmed.parse::<i64>().is_ok() || trimmed.parse::<f64>().is_ok() {
        return true;
    }
    
    // 16進数
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        return true;
    }
    
    // 文字列リテラル
    if (trimmed.starts_with('"') && trimmed.ends_with('"')) ||
       (trimmed.starts_with('\'') && trimmed.ends_with('\'')) {
        return true;
    }
    
    false
}

/// 演算子を含むかチェック
fn contains_operator(value: &str) -> bool {
    let operators = ["+", "-", "*", "/", "%", "<<", ">>", "&", "|", "^", "~"];
    
    for op in &operators {
        if value.contains(op) {
            return true;
        }
    }
    
    false
}

/// 値全体が括弧で囲まれているかチェック
fn is_wrapped_in_parentheses(value: &str) -> bool {
    let trimmed = value.trim();
    
    if !trimmed.starts_with('(') || !trimmed.ends_with(')') {
        return false;
    }
    
    // 対応する括弧かチェック
    let mut depth = 0;
    for (i, ch) in trimmed.chars().enumerate() {
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            // 最初の開き括弧に対応する閉じ括弧が最後でない場合はfalse
            if depth == 0 && i < trimmed.len() - 1 {
                return false;
            }
        }
    }
    
    depth == 0
}
