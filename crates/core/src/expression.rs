use crate::span::Span;

/// C言語の式を表すAST
#[derive(Debug, Clone)]
pub enum Expression {
    /// 整数リテラル
    IntLiteral {
        value: i64,
        span: Span,
    },
    
    /// 浮動小数点リテラル
    FloatLiteral {
        value: f64,
        span: Span,
    },
    
    /// 識別子（変数名、関数名等）
    Identifier {
        name: String,
        span: Span,
    },
    
    /// 二項演算（a + b, a == b 等）
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
        span: Span,
    },
    
    /// 単項演算（-a, !a, *p 等）
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
        span: Span,
    },
    
    /// キャスト式（(type) expr）
    Cast {
        target_type: crate::type_system::Type,
        operand: Box<Expression>,
        span: Span,
    },
    
    /// 関数呼び出し
    FunctionCall {
        function: Box<Expression>,
        arguments: Vec<Expression>,
        span: Span,
    },
    
    /// 配列アクセス（a[i]）
    ArrayAccess {
        array: Box<Expression>,
        index: Box<Expression>,
        span: Span,
    },
    
    /// メンバーアクセス（a.b）
    MemberAccess {
        object: Box<Expression>,
        member: String,
        span: Span,
    },
    
    /// ポインタメンバーアクセス（a->b）
    PointerMemberAccess {
        object: Box<Expression>,
        member: String,
        span: Span,
    },
    
    /// 三項演算子（a ? b : c）
    Conditional {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
        span: Span,
    },
    
    /// 代入（a = b）
    Assignment {
        left: Box<Expression>,
        right: Box<Expression>,
        span: Span,
    },
}

/// 二項演算子
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    // 算術演算子
    Add,        // +
    Subtract,   // -
    Multiply,   // *
    Divide,     // /
    Modulo,     // %
    
    // ビット演算子
    BitwiseAnd, // &
    BitwiseOr,  // |
    BitwiseXor, // ^
    LeftShift,  // <<
    RightShift, // >>
    
    // 比較演算子
    Equal,          // ==
    NotEqual,       // !=
    LessThan,       // <
    LessThanOrEq,   // <=
    GreaterThan,    // >
    GreaterThanOrEq,// >=
    
    // 論理演算子
    LogicalAnd,     // &&
    LogicalOr,      // ||
}

/// 単項演算子
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Negate,         // - (単項マイナス)
    LogicalNot,     // !
    BitwiseNot,     // ~
    AddressOf,      // &
    Dereference,    // *
    PreIncrement,   // ++a
    PreDecrement,   // --a
    PostIncrement,  // a++
    PostDecrement,  // a--
}

impl Expression {
    /// 式のスパン情報を取得
    pub fn span(&self) -> &Span {
        match self {
            Expression::IntLiteral { span, .. } => span,
            Expression::FloatLiteral { span, .. } => span,
            Expression::Identifier { span, .. } => span,
            Expression::BinaryOp { span, .. } => span,
            Expression::UnaryOp { span, .. } => span,
            Expression::Cast { span, .. } => span,
            Expression::FunctionCall { span, .. } => span,
            Expression::ArrayAccess { span, .. } => span,
            Expression::MemberAccess { span, .. } => span,
            Expression::PointerMemberAccess { span, .. } => span,
            Expression::Conditional { span, .. } => span,
            Expression::Assignment { span, .. } => span,
        }
    }
}
