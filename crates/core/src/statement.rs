use crate::span::Span;
use crate::expression::Expression;
use crate::type_system::Type;

/// C言語の文を表すAST
#[derive(Debug, Clone)]
pub enum Statement {
    /// 式文（expression;）
    ExpressionStmt {
        expression: Option<Expression>, // None の場合は空文（;のみ）
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
        init: Option<Box<Statement>>,       // 初期化
        condition: Option<Expression>,       // 条件
        increment: Option<Expression>,       // 増分
        body: Vec<Statement>,
        span: Span,
    },
    
    /// do-while文
    DoWhile {
        body: Vec<Statement>,
        condition: Expression,
        span: Span,
    },
    
    /// break文
    Break {
        span: Span,
    },
    
    /// continue文
    Continue {
        span: Span,
    },
    
    /// switch文
    Switch {
        expression: Expression,
        cases: Vec<SwitchCase>,
        span: Span,
    },
    
    /// 複合文（ブロック）
    Compound {
        statements: Vec<Statement>,
        span: Span,
    },
    
    /// 変数宣言文
    VariableDecl {
        var_type: Option<Type>,
        name: String,
        initializer: Option<Expression>,
        span: Span,
    },
    
    /// ラベル文
    Label {
        label: String,
        statement: Box<Statement>,
        span: Span,
    },
    
    /// goto文
    Goto {
        label: String,
        span: Span,
    },
}

/// switch文のcase
#[derive(Debug, Clone)]
pub enum SwitchCase {
    Case {
        value: Expression,
        statements: Vec<Statement>,
        span: Span,
    },
    Default {
        statements: Vec<Statement>,
        span: Span,
    },
}
