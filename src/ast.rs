use crate::span::Span;

#[derive(Debug)]
pub struct TranslationUnit {
    pub items: Vec<Item>,
}

#[derive(Debug)]
pub enum Item {
    BlockComment { span: Span, text: String },
    Include { span: Span, text: String, filename: String },
    Define { span: Span, text: String, macro_name: String, macro_value: String },
    // Stage 1: 条件コンパイルブロック（#ifdef から #endif まで）
    ConditionalBlock { 
        directive_type: String,  // "ifdef", "ifndef", "if"
        condition: String,       // 条件式（"DEBUG", "defined(FEATURE)" など）
        items: Vec<Item>,        // ブロック内のアイテム
        start_span: Span,        // #ifdef のspan
        end_span: Span,          // #endif のspan
    },
    TypedefDecl { span: Span, text: String },
    VarDecl { 
        span: Span, 
        text: String,
        var_name: String,
        has_initializer: bool,
    },
    StructDecl {
        span: Span,
        text: String,
        struct_name: Option<String>,  // 無名構造体の場合は None
        has_typedef: bool,            // typedef struct の場合 true
    },
    EnumDecl {
        span: Span,
        text: String,
        enum_name: Option<String>,    // 無名enumの場合は None
        has_typedef: bool,            // typedef enum の場合 true
        variable_names: Vec<String>,  // 同時に宣言された変数のリスト
    },
    UnionDecl {
        span: Span,
        text: String,
        union_name: Option<String>,   // 無名unionの場合は None
        has_typedef: bool,            // typedef union の場合 true
        variable_names: Vec<String>,  // 同時に宣言された変数のリスト
    },
    FunctionDecl {
        span: Span,
        text: String,
        return_type: String,           // 戻り値の型（"int", "void" など）
        function_name: String,         // 関数名
        parameters: String,            // 引数リスト（"(void)", "(int x, char *y)" など）
        storage_class: Option<String>, // 記憶域クラス（"static" など）
    },
}
