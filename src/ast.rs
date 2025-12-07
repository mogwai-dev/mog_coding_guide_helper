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
}
