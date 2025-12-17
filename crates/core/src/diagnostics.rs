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
    pub check_global_var_naming: bool,  // グローバル変数の命名規則チェック（大文字）
    pub check_global_var_type_prefix: bool,  // グローバル変数の型名プレフィックスチェック
    pub check_preprocessor_indent: bool,  // プリプロセッサディレクティブのインデントチェック
    pub check_indent_style: bool,  // インデントスタイル（タブ/スペース）のチェック
    pub indent_style: crate::config::IndentStyle,  // 期待されるインデントスタイル
    pub indent_width: usize,  // スペース使用時のインデント幅
}

impl Default for DiagnosticConfig {
    fn default() -> Self {
        DiagnosticConfig {
            check_file_header: true,
            check_storage_class_order: true,
            check_function_format: true,
            check_type_safety: true,
            check_macro_parentheses: true,
            check_global_var_naming: true,
            check_global_var_type_prefix: true,
            check_preprocessor_indent: true,
            check_indent_style: true,
            indent_style: crate::config::IndentStyle::Spaces,
            indent_width: 4,
        }
    }
}

/// TranslationUnitに対して診断を実行
pub fn diagnose(tu: &TranslationUnit, config: &DiagnosticConfig) -> Vec<Diagnostic> {
    diagnose_with_source(tu, config, "")
}

/// ソースコード付きで診断を実行（インデントスタイルチェック用）
pub fn diagnose_with_source(tu: &TranslationUnit, config: &DiagnosticConfig, source: &str) -> Vec<Diagnostic> {
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
    
    if config.check_global_var_naming {
        diagnostics.extend(check_global_var_naming(tu));
    }
    
    if config.check_global_var_type_prefix {
        diagnostics.extend(check_global_var_type_prefix(tu));
    }
    
    if config.check_preprocessor_indent {
        diagnostics.extend(check_preprocessor_indent(tu));
    }
    
    if config.check_indent_style && !source.is_empty() {
        diagnostics.extend(check_indent_style(source, &config.indent_style, config.indent_width));
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

/// グローバル変数の命名規則チェック（大文字であるべき）
fn check_global_var_naming(tu: &TranslationUnit) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    
    // トップレベルのアイテムのみチェック（関数内部は除外）
    for item in &tu.items {
        match item {
            Item::VarDecl { span, var_name, var_type, .. } => {
                // extern宣言やtypedefは除外（var_typeがあるものだけチェック）
                if var_type.is_some() && !is_uppercase_with_underscores(var_name) {
                    diagnostics.push(Diagnostic {
                        span: span.clone(),
                        severity: DiagnosticSeverity::Warning,
                        message: format!(
                            "グローバル変数 '{}' は大文字とアンダースコアで命名することを推奨します。例: '{}'",
                            var_name,
                            to_uppercase_with_underscores(var_name)
                        ),
                        code: "CGH006".to_string(),
                    });
                }
            },
            Item::StructDecl { span, text, variable_names, members, has_typedef, .. } => {
                // typedefでない場合のみ変数名をチェック
                if !has_typedef {
                    // variable_namesが空の場合、textから抽出（変数宣言のみの場合）
                    if variable_names.is_empty() && members.is_empty() {
                        if let Some(var_name) = extract_var_name_from_struct_decl(text) {
                            if !is_uppercase_with_underscores(&var_name) {
                                diagnostics.push(Diagnostic {
                                    span: span.clone(),
                                    severity: DiagnosticSeverity::Warning,
                                    message: format!(
                                        "グローバル変数 '{}' は大文字とアンダースコアで命名することを推奨します。例: '{}'",
                                        var_name,
                                        to_uppercase_with_underscores(&var_name)
                                    ),
                                    code: "CGH006".to_string(),
                                });
                            }
                        }
                    } else {
                        // variable_namesがある場合はそれをチェック
                        for var_name in variable_names {
                            if !is_uppercase_with_underscores(var_name) {
                                diagnostics.push(Diagnostic {
                                    span: span.clone(),
                                    severity: DiagnosticSeverity::Warning,
                                    message: format!(
                                        "グローバル変数 '{}' は大文字とアンダースコアで命名することを推奨します。例: '{}'",
                                        var_name,
                                        to_uppercase_with_underscores(var_name)
                                    ),
                                    code: "CGH006".to_string(),
                                });
                            }
                        }
                    }
                }
            },
            Item::EnumDecl { span, text, variable_names, variants, has_typedef, .. } => {
                // typedefでない場合のみ変数名をチェック
                if !has_typedef {
                    // variable_namesが空の場合、textから抽出（変数宣言のみの場合）
                    if variable_names.is_empty() && variants.is_empty() {
                        if let Some(var_name) = extract_var_name_from_struct_decl(text) {
                            if !is_uppercase_with_underscores(&var_name) {
                                diagnostics.push(Diagnostic {
                                    span: span.clone(),
                                    severity: DiagnosticSeverity::Warning,
                                    message: format!(
                                        "グローバル変数 '{}' は大文字とアンダースコアで命名することを推奨します。例: '{}'",
                                        var_name,
                                        to_uppercase_with_underscores(&var_name)
                                    ),
                                    code: "CGH006".to_string(),
                                });
                            }
                        }
                    } else {
                        // variable_namesがある場合はそれをチェック
                        for var_name in variable_names {
                            if !is_uppercase_with_underscores(var_name) {
                                diagnostics.push(Diagnostic {
                                    span: span.clone(),
                                    severity: DiagnosticSeverity::Warning,
                                    message: format!(
                                        "グローバル変数 '{}' は大文字とアンダースコアで命名することを推奨します。例: '{}'",
                                        var_name,
                                        to_uppercase_with_underscores(var_name)
                                    ),
                                    code: "CGH006".to_string(),
                                });
                            }
                        }
                    }
                }
            },
            Item::UnionDecl { span, text, variable_names, members, has_typedef, .. } => {
                // typedefでない場合のみ変数名をチェック
                if !has_typedef {
                    // variable_namesが空の場合、textから抽出（変数宣言のみの場合）
                    if variable_names.is_empty() && members.is_empty() {
                        if let Some(var_name) = extract_var_name_from_struct_decl(text) {
                            if !is_uppercase_with_underscores(&var_name) {
                                diagnostics.push(Diagnostic {
                                    span: span.clone(),
                                    severity: DiagnosticSeverity::Warning,
                                    message: format!(
                                        "グローバル変数 '{}' は大文字とアンダースコアで命名することを推奨します。例: '{}'",
                                        var_name,
                                        to_uppercase_with_underscores(&var_name)
                                    ),
                                    code: "CGH006".to_string(),
                                });
                            }
                        }
                    } else {
                        // variable_namesがある場合はそれをチェック
                        for var_name in variable_names {
                            if !is_uppercase_with_underscores(var_name) {
                                diagnostics.push(Diagnostic {
                                    span: span.clone(),
                                    severity: DiagnosticSeverity::Warning,
                                    message: format!(
                                        "グローバル変数 '{}' は大文字とアンダースコアで命名することを推奨します。例: '{}'",
                                        var_name,
                                        to_uppercase_with_underscores(var_name)
                                    ),
                                    code: "CGH006".to_string(),
                                });
                            }
                        }
                    }
                }
            },
            _ => {}
        }
    }
    
    diagnostics
}

/// struct宣言のtextフィールドから変数名を抽出
/// 例: "struct Point myPoint;" -> Some("myPoint")
fn extract_var_name_from_struct_decl(text: &str) -> Option<String> {
    // "struct TypeName varName;" のパターンをパース
    let trimmed = text.trim();
    
    // "struct" で始まっているか確認
    if !trimmed.starts_with("struct") && !trimmed.starts_with("enum") && !trimmed.starts_with("union") {
        return None;
    }
    
    // セミコロンの前の部分を取得
    let before_semicolon = trimmed.trim_end_matches(';').trim();
    
    // 最後のトークン（変数名）を取得
    let tokens: Vec<&str> = before_semicolon.split_whitespace().collect();
    if tokens.len() >= 3 {
        // "struct" "TypeName" "varName" の形式
        return Some(tokens.last().unwrap().to_string());
    }
    
    None
}

/// 文字列が大文字とアンダースコアのみで構成されているか判定
fn is_uppercase_with_underscores(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    
    for ch in s.chars() {
        if !ch.is_uppercase() && ch != '_' && !ch.is_numeric() {
            return false;
        }
    }
    
    true
}

/// 文字列を大文字とアンダースコアの形式に変換
fn to_uppercase_with_underscores(s: &str) -> String {
    let mut result = String::new();
    let mut prev_was_lower = false;
    
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            // キャメルケースの境界を検出（前の文字が小文字で現在が大文字）
            if i > 0 && prev_was_lower {
                result.push('_');
            }
            result.push(ch);
            prev_was_lower = false;
        } else if ch.is_lowercase() {
            result.push(ch.to_uppercase().next().unwrap());
            prev_was_lower = true;
        } else {
            // アンダースコアや数字はそのまま
            result.push(ch);
            prev_was_lower = false;
        }
    }
    
    result
}

/// グローバル変数の型名プレフィックスチェック
fn check_global_var_type_prefix(tu: &TranslationUnit) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    
    // 型名とそのプレフィックスのマッピング
    let type_prefixes = [
        ("VU8", "VU8_"),
        ("VU16", "VU16_"),
        ("VU32", "VU32_"),
        ("VU64", "VU64_"),
        ("VS8", "VS8_"),
        ("VS16", "VS16_"),
        ("VS32", "VS32_"),
        ("VS64", "VS64_"),
        ("CU8", "CU8_"),
        ("CU16", "CU16_"),
        ("CU32", "CU32_"),
        ("CU64", "CU64_"),
        ("CS8", "CS8_"),
        ("CS16", "CS16_"),
        ("CS32", "CS32_"),
        ("CS64", "CS64_"),
    ];
    
    for item in &tu.items {
        if let Item::VarDecl { span, var_name, var_type, text, .. } = item {
            if var_type.is_some() {
                // textから型名を抽出（最初の空白までが型名）
                let type_name = text.trim().split_whitespace().next().unwrap_or("");
                
                // 型名が該当するかチェック
                for (type_str, prefix) in &type_prefixes {
                    if type_name == *type_str && !var_name.starts_with(prefix) {
                        let suggested_name = if var_name.starts_with(prefix.trim_end_matches('_')) {
                            // 既にプレフィックス部分はあるが_がない場合
                            format!("{}{}", prefix, &var_name[prefix.trim_end_matches('_').len()..])
                        } else {
                            // プレフィックスがない場合
                            format!("{}{}", prefix, var_name)
                        };
                        
                        diagnostics.push(Diagnostic {
                            span: span.clone(),
                            severity: DiagnosticSeverity::Warning,
                            message: format!(
                                "型 '{}' のグローバル変数 '{}' は '{}' で始まることを推奨します。例: '{}'",
                                type_str,
                                var_name,
                                prefix,
                                suggested_name
                            ),
                            code: "CGH007".to_string(),
                        });
                        break;
                    }
                }
            }
        }
    }
    
    diagnostics
}

/// プリプロセッサディレクティブのインデントをチェック（CGH008）
/// プリプロセッサディレクティブは行頭（0列目）から始まるべき
fn check_preprocessor_indent(tu: &TranslationUnit) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    
    fn check_items(items: &[Item], diagnostics: &mut Vec<Diagnostic>) {
        for item in items {
            match item {
                Item::Include { span, .. } |
                Item::Define { span, .. } => {
                    // プリプロセッサディレクティブの前にスペースがあるかチェック
                    if span.start_column > 0 {
                        diagnostics.push(Diagnostic {
                            span: span.clone(),
                            severity: DiagnosticSeverity::Warning,
                            message: format!(
                                "プリプロセッサディレクティブの前にスペースがあります（{}文字）。行頭から始めてください。",
                                span.start_column
                            ),
                            code: "CGH008".to_string(),
                        });
                    }
                },
                Item::ConditionalBlock { start_span, items: nested_items, .. } => {
                    // ifdef/ifndef/if/elif/else/endifもチェック
                    if start_span.start_column > 0 {
                        diagnostics.push(Diagnostic {
                            span: start_span.clone(),
                            severity: DiagnosticSeverity::Warning,
                            message: format!(
                                "プリプロセッサディレクティブの前にスペースがあります（{}文字）。行頭から始めてください。",
                                start_span.start_column
                            ),
                            code: "CGH008".to_string(),
                        });
                    }
                    // ネストされたアイテムも再帰的にチェック
                    check_items(nested_items, diagnostics);
                },
                _ => {}
            }
        }
    }
    
    check_items(&tu.items, &mut diagnostics);
    diagnostics
}

/// CGH009: インデントスタイル（タブ/スペース）のチェック
fn check_indent_style(
    source: &str,
    expected_style: &crate::config::IndentStyle,
    indent_width: usize,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    
    // ソースコードの各行をチェック
    let lines: Vec<&str> = source.lines().collect();
    
    for (line_idx, line) in lines.iter().enumerate() {
        // 空行や非空白文字で始まる行はスキップ
        if line.is_empty() || !line.starts_with(|c: char| c == ' ' || c == '\t') {
            continue;
        }
        
        // 行頭の空白を解析
        let leading_whitespace: String = line.chars()
            .take_while(|c| *c == ' ' || *c == '\t')
            .collect();
        
        let has_tabs = leading_whitespace.contains('\t');
        let has_spaces = leading_whitespace.contains(' ');
        
        // タブとスペースの混在をチェック
        if has_tabs && has_spaces {
            let span = Span {
                start_line: line_idx + 1,
                start_column: 0,
                end_line: line_idx + 1,
                end_column: leading_whitespace.len(),
                byte_start_idx: 0,
                byte_end_idx: leading_whitespace.len(),
            };
            
            diagnostics.push(Diagnostic {
                span,
                severity: DiagnosticSeverity::Warning,
                message: format!(
                    "インデントにタブとスペースが混在しています。{}のみを使用してください。",
                    match expected_style {
                        crate::config::IndentStyle::Tabs => "タブ",
                        crate::config::IndentStyle::Spaces => "スペース",
                    }
                ),
                code: "CGH009".to_string(),
            });
            continue;
        }
        
        // 期待されるスタイルと実際のスタイルが一致しているかチェック
        match expected_style {
            crate::config::IndentStyle::Tabs => {
                if has_spaces && !has_tabs {
                    let span = Span {
                        start_line: line_idx + 1,
                        start_column: 0,
                        end_line: line_idx + 1,
                        end_column: leading_whitespace.len(),
                        byte_start_idx: 0,
                        byte_end_idx: leading_whitespace.len(),
                    };
                    
                    diagnostics.push(Diagnostic {
                        span,
                        severity: DiagnosticSeverity::Warning,
                        message: "インデントにタブを使用すべきところでスペースが使われています。".to_string(),
                        code: "CGH009".to_string(),
                    });
                }
            }
            crate::config::IndentStyle::Spaces => {
                if has_tabs && !has_spaces {
                    let span = Span {
                        start_line: line_idx + 1,
                        start_column: 0,
                        end_line: line_idx + 1,
                        end_column: leading_whitespace.len(),
                        byte_start_idx: 0,
                        byte_end_idx: leading_whitespace.len(),
                    };
                    
                    diagnostics.push(Diagnostic {
                        span,
                        severity: DiagnosticSeverity::Warning,
                        message: format!(
                            "インデントにスペース（{}文字単位）を使用すべきところでタブが使われています。",
                            indent_width
                        ),
                        code: "CGH009".to_string(),
                    });
                }
            }
        }
    }
    
    diagnostics
}
