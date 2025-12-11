use crate::span::Span;
use crate::trivia::Trivia;
use crate::type_system::Type;

#[derive(Debug)]
pub struct TranslationUnit {
    pub items: Vec<Item>,
    pub leading_trivia: Trivia,  // ファイル先頭のコメント（ヘッダーコメントなど）
}

#[derive(Debug)]
pub enum Item {
    // BlockComment と LineComment は削除（triviaに移行）
    Include { 
        span: Span, 
        text: String, 
        filename: String,
        trivia: Trivia,
    },
    Define { 
        span: Span, 
        text: String, 
        macro_name: String, 
        macro_value: String,
        trivia: Trivia,
    },
    ConditionalBlock { 
        directive_type: String,
        condition: String,
        items: Vec<Item>,
        start_span: Span,
        end_span: Span,
        trivia: Trivia,
    },
    TypedefDecl { 
        span: Span, 
        text: String,
        trivia: Trivia,
    },
    VarDecl { 
        span: Span, 
        text: String,
        var_name: String,
        has_initializer: bool,
        var_type: Option<Type>,
        trivia: Trivia,
    },
    StructDecl {
        span: Span,
        text: String,
        struct_name: Option<String>,
        has_typedef: bool,
        trivia: Trivia,
    },
    EnumDecl {
        span: Span,
        text: String,
        enum_name: Option<String>,
        has_typedef: bool,
        variable_names: Vec<String>,
        trivia: Trivia,
    },
    UnionDecl {
        span: Span,
        text: String,
        union_name: Option<String>,
        has_typedef: bool,
        variable_names: Vec<String>,
        trivia: Trivia,
    },
    FunctionDecl {
        span: Span,
        text: String,
        return_type: String,
        function_name: String,
        parameters: String,
        storage_class: Option<String>,
        trivia: Trivia,
    },
}
