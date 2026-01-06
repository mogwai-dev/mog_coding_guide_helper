use crate::ast::{TranslationUnit, Item};
use crate::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub span: Span,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub code: DiagnosticCode,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

/// 診断コードの種類
#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticCode {
    /// プロジェクト固有の診断コード（既存）
    Custom(String),
    
    /// CERT-C 診断
    CertC(String),  // 例: "ARR32-C"
    
    /// CWE-C 診断
    CweC(u32),      // 例: 120 (CWE-120)
    
    /// MISRA-C 診断
    MisraC { directive: u8, rule: u8 }, // 例: Directive 8, Rule 1
}

impl std::fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagnosticCode::Custom(code) => write!(f, "{}", code),
            DiagnosticCode::CertC(code) => write!(f, "CERT-{}", code),
            DiagnosticCode::CweC(num) => write!(f, "CWE-{}", num),
            DiagnosticCode::MisraC { directive, rule } => {
                write!(f, "MISRA-C:2012 Dir {}.{}", directive, rule)
            }
        }
    }
}

impl Diagnostic {
    pub fn new(
        severity: DiagnosticSeverity,
        code: DiagnosticCode,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        Self {
            severity,
            code,
            message: message.into(),
            span,
            notes: Vec::new(),
        }
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct DiagnosticConfig {
    pub check_file_header: bool,
    pub check_storage_class_order: bool,
    pub check_function_format: bool,
    
    // CERT-C チェック
    pub check_cert_c: bool,
    
    // CWE-C チェック
    pub check_cwe_c: bool,
    
    // MISRA-C チェック
    pub check_misra_c: bool,
}

impl Default for DiagnosticConfig {
    fn default() -> Self {
        DiagnosticConfig {
            check_file_header: true,
            check_storage_class_order: true,
            check_function_format: true,
            check_cert_c: true,
            check_cwe_c: true,
            check_misra_c: true,
        }
    }
}

/// TranslationUnitに対して診断を実行
pub fn diagnose(tu: &TranslationUnit, config: &DiagnosticConfig, source: &str) -> Vec<Diagnostic> {
    diagnose_all(tu, source, config)
}

/// ファイルヘッダーコメントの存在をチェック
fn check_file_header(tu: &TranslationUnit, _source: &str) -> Option<Diagnostic> {
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

/// CERT-C 診断を実行
/// 
/// 実装予定のルール:
/// - ARR32-C: Ensure size arguments for variable length arrays are in a valid range
/// - MEM31-C: Free dynamically allocated memory when no longer needed
/// - STR31-C: Guarantee that storage for strings has sufficient space for character data and the null terminator
/// - INT32-C: Ensure that operations on signed integers do not result in overflow
fn check_cert_c(tu: &TranslationUnit, _source: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    
    // TODO: CERT-C ルールの実装
    // 現在は基本的なフレームワークのみ
    
    diagnostics
}

/// CWE-C 診断を実行
/// 
/// 実装予定のルール:
/// - CWE-120: Buffer Copy without Checking Size of Input ('Classic Buffer Overflow')
/// - CWE-457: Use of Uninitialized Variable
/// - CWE-476: NULL Pointer Dereference
/// - CWE-190: Integer Overflow or Wraparound
fn check_cwe_c(tu: &TranslationUnit, _source: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    
    // TODO: CWE-C ルールの実装
    // 現在は基本的なフレームワークのみ
    
    diagnostics
}

/// MISRA-C 診断を実行
/// 
/// 実装予定のルール:
/// - Rule 8.1: Types shall be explicitly specified
/// - Rule 9.1: The value of an object with automatic storage duration shall not be read before it has been set
/// - Rule 14.3: Controlling expressions shall not be invariant
/// - Rule 17.7: The value returned by a function having non-void return type shall be used
fn check_misra_c(tu: &TranslationUnit, _source: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    
    // TODO: MISRA-C ルールの実装
    // 現在は基本的なフレームワークのみ
    
    diagnostics
}

/// 診断エンジン: すべてのチェックを統合して実行
pub fn diagnose_all(tu: &TranslationUnit, source: &str, config: &DiagnosticConfig) -> Vec<Diagnostic> {
    let mut all_diagnostics = Vec::new();
    
    // CGH カスタムルールチェック
    if config.check_file_header {
        if let Some(diag) = check_file_header(tu, source) {
            all_diagnostics.push(diag);
        }
    }
    
    if config.check_function_format {
        all_diagnostics.extend(check_function_format(tu));
    }
    
    // CERT-C チェック
    if config.check_cert_c {
        all_diagnostics.extend(check_cert_c(tu, source));
    }
    
    // CWE-C チェック
    if config.check_cwe_c {
        all_diagnostics.extend(check_cwe_c(tu, source));
    }
    
    // MISRA-C チェック
    if config.check_misra_c {
        all_diagnostics.extend(check_misra_c(tu, source));
    }
    
    all_diagnostics
}
