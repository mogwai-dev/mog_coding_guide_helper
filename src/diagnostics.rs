use crate::ast::{TranslationUnit, Item};
use crate::span::Span;

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
}

impl Default for DiagnosticConfig {
    fn default() -> Self {
        DiagnosticConfig {
            check_file_header: true,
            check_storage_class_order: true,
            check_function_format: true,
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
