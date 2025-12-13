use crate::span::Span;
use crate::trivia::Trivia;
use crate::type_system::Type;
use crate::expression::Expression;

/// ステートメント（文）
#[derive(Debug, Clone)]
pub enum Statement {
    /// 変数宣言文
    VarDecl {
        var_type: Option<Type>,
        var_name: String,
        initializer: Option<Expression>,
        span: Span,
    },
    /// 式文（式の後にセミコロン）
    Expression {
        expr: Expression,
        span: Span,
    },
    /// return文
    Return {
        value: Option<Expression>,
        span: Span,
    },
    /// if文
    If {
        condition: Expression,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
        span: Span,
    },
    /// while文
    While {
        condition: Expression,
        body: Vec<Statement>,
        span: Span,
    },
    /// for文
    For {
        init: Option<Box<Statement>>,
        condition: Option<Expression>,
        update: Option<Expression>,
        body: Vec<Statement>,
        span: Span,
    },
    /// ブロック文 { ... }
    Block {
        statements: Vec<Statement>,
        span: Span,
    },
    /// 空文（セミコロンのみ）
    Empty {
        span: Span,
    },
}

/// struct のメンバー情報
#[derive(Debug, Clone)]
pub struct StructMember {
    pub name: String,
    pub member_type: Option<Type>,
    pub bitfield_width: Option<u32>,  // ビットフィールド幅（例: unsigned int flag : 1;）
    pub span: Span,
}

/// union のメンバー情報
#[derive(Debug, Clone)]
pub struct UnionMember {
    pub name: String,
    pub member_type: Option<Type>,
    pub span: Span,
}

/// enum の列挙子情報
#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<i64>,  // 明示的な値指定（例: RED = 0）
    pub span: Span,
}

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
        condition_result: bool,  // 条件評価結果
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
        members: Vec<StructMember>,  // メンバー情報
        trivia: Trivia,
    },
    EnumDecl {
        span: Span,
        text: String,
        enum_name: Option<String>,
        has_typedef: bool,
        variable_names: Vec<String>,
        variants: Vec<EnumVariant>,  // 列挙子情報
        trivia: Trivia,
    },
    UnionDecl {
        span: Span,
        text: String,
        union_name: Option<String>,
        has_typedef: bool,
        variable_names: Vec<String>,
        members: Vec<UnionMember>,  // メンバー情報
        trivia: Trivia,
    },
    FunctionDecl {
        span: Span,
        text: String,
        return_type: String,
        function_name: String,
        parameters: String,
        storage_class: Option<String>,
        body: Option<Vec<Statement>>,  // 関数本体（定義の場合のみ）
        trivia: Trivia,
    },
}
