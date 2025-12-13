use crate::span::Span;

// 各トークン種類の構造体
#[derive(Debug, Clone)]
pub struct BlockCommentToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IncludeToken {
    pub span: Span,
    pub filename: String,
}

#[derive(Debug, Clone)]
pub struct DefineToken {
    pub span: Span,
    pub macro_name: String,
    pub macro_value: String,
}

#[derive(Debug, Clone)]
pub struct SemicolonToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EqualToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AsteriskToken {
    pub span: Span,
}

// 演算子トークン
#[derive(Debug, Clone)]
pub struct PlusToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MinusToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SlashToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct PercentToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EqualEqualToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct NotEqualToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LessThanToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LessThanOrEqualToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct GreaterThanToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct GreaterThanOrEqualToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AmpersandToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AmpersandAmpersandToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct PipeToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct PipePipeToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CaretToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TildeToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ExclamationToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LeftShiftToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct RightShiftToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LeftBracketToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct RightBracketToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct QuestionToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ColonToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CommaToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct DotToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ArrowToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct PlusPlusToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MinusMinusToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IdentToken {
    pub span: Span,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct AutoToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct RegisterToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StaticToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ExternToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TypedefToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ConstToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct VolatileToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct RestrictToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AtomicToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IntToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CharToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FloatToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct DoubleToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct VoidToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LongToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ShortToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SignedToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct UnsignedToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct UnionToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LeftBraceToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct RightBraceToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LeftParenToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct RightParenToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ReturnToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IfKeywordToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ElseKeywordToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct WhileToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ForToken {
    pub span: Span,
}

// 条件コンパイルディレクティブトークン
#[derive(Debug, Clone)]
pub struct IfdefToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IfndefToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IfToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ElifToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ElseToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EndifToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LineCommentToken {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct NumberLiteralToken {
    pub span: Span,
    pub value: String,  // "123", "0x1A", "0755" など元の文字列表現
}

#[derive(Debug, Clone)]
pub struct FloatLiteralToken {
    pub span: Span,
    pub value: String,  // "1.5", "3.14f", "1e10", "2.5e-3L" など元の文字列表現
}

// トークンの enum
#[derive(Debug, Clone)]
pub enum Token {
    BlockComment(BlockCommentToken),
    LineComment(LineCommentToken),
    Include(IncludeToken),
    Define(DefineToken),
    Ifdef(IfdefToken),
    Ifndef(IfndefToken),
    If(IfToken),
    Elif(ElifToken),
    Else(ElseToken),
    Endif(EndifToken),
    Semicolon(SemicolonToken),
    Equal(EqualToken),
    Asterisk(AsteriskToken),
    NumberLiteral(NumberLiteralToken),
    FloatLiteral(FloatLiteralToken),
    
    // 演算子
    Plus(PlusToken),
    Minus(MinusToken),
    Slash(SlashToken),
    Percent(PercentToken),
    EqualEqual(EqualEqualToken),
    NotEqual(NotEqualToken),
    LessThan(LessThanToken),
    LessThanOrEqual(LessThanOrEqualToken),
    GreaterThan(GreaterThanToken),
    GreaterThanOrEqual(GreaterThanOrEqualToken),
    Ampersand(AmpersandToken),
    AmpersandAmpersand(AmpersandAmpersandToken),
    Pipe(PipeToken),
    PipePipe(PipePipeToken),
    Caret(CaretToken),
    Tilde(TildeToken),
    Exclamation(ExclamationToken),
    LeftShift(LeftShiftToken),
    RightShift(RightShiftToken),
    LeftBracket(LeftBracketToken),
    RightBracket(RightBracketToken),
    Question(QuestionToken),
    Colon(ColonToken),
    Comma(CommaToken),
    Dot(DotToken),
    Arrow(ArrowToken),
    PlusPlus(PlusPlusToken),
    MinusMinus(MinusMinusToken),
    
    Ident(IdentToken),
    Auto(AutoToken),
    Register(RegisterToken),
    Static(StaticToken),
    Extern(ExternToken),
    Typedef(TypedefToken),
    Const(ConstToken),
    Volatile(VolatileToken),
    Restrict(RestrictToken),
    Atomic(AtomicToken),
    Int(IntToken),
    Char(CharToken),
    Float(FloatToken),
    Double(DoubleToken),
    Void(VoidToken),
    Long(LongToken),
    Short(ShortToken),
    Signed(SignedToken),
    Unsigned(UnsignedToken),
    Struct(StructToken),
    Enum(EnumToken),
    Union(UnionToken),
    LeftBrace(LeftBraceToken),
    RightBrace(RightBraceToken),
    LeftParen(LeftParenToken),
    RightParen(RightParenToken),
    Return(ReturnToken),
    IfKeyword(IfKeywordToken),
    ElseKeyword(ElseKeywordToken),
    While(WhileToken),
    For(ForToken),
}

impl Token {
    /// トークンのスパン情報を取得
    pub fn span(&self) -> Span {
        match self {
            Token::BlockComment(t) => t.span.clone(),
            Token::LineComment(t) => t.span.clone(),
            Token::Include(t) => t.span.clone(),
            Token::Define(t) => t.span.clone(),
            Token::Ifdef(t) => t.span.clone(),
            Token::Ifndef(t) => t.span.clone(),
            Token::If(t) => t.span.clone(),
            Token::Elif(t) => t.span.clone(),
            Token::Else(t) => t.span.clone(),
            Token::Endif(t) => t.span.clone(),
            Token::Semicolon(t) => t.span.clone(),
            Token::Equal(t) => t.span.clone(),
            Token::Asterisk(t) => t.span.clone(),
            Token::NumberLiteral(t) => t.span.clone(),
            Token::FloatLiteral(t) => t.span.clone(),
            Token::Plus(t) => t.span.clone(),
            Token::Minus(t) => t.span.clone(),
            Token::Slash(t) => t.span.clone(),
            Token::Percent(t) => t.span.clone(),
            Token::EqualEqual(t) => t.span.clone(),
            Token::NotEqual(t) => t.span.clone(),
            Token::LessThan(t) => t.span.clone(),
            Token::LessThanOrEqual(t) => t.span.clone(),
            Token::GreaterThan(t) => t.span.clone(),
            Token::GreaterThanOrEqual(t) => t.span.clone(),
            Token::Ampersand(t) => t.span.clone(),
            Token::AmpersandAmpersand(t) => t.span.clone(),
            Token::Pipe(t) => t.span.clone(),
            Token::PipePipe(t) => t.span.clone(),
            Token::Caret(t) => t.span.clone(),
            Token::Tilde(t) => t.span.clone(),
            Token::Exclamation(t) => t.span.clone(),
            Token::LeftShift(t) => t.span.clone(),
            Token::RightShift(t) => t.span.clone(),
            Token::LeftBracket(t) => t.span.clone(),
            Token::RightBracket(t) => t.span.clone(),
            Token::Question(t) => t.span.clone(),
            Token::Colon(t) => t.span.clone(),
            Token::Comma(t) => t.span.clone(),
            Token::Dot(t) => t.span.clone(),
            Token::Arrow(t) => t.span.clone(),
            Token::PlusPlus(t) => t.span.clone(),
            Token::MinusMinus(t) => t.span.clone(),
            Token::Ident(t) => t.span.clone(),
            Token::Auto(t) => t.span.clone(),
            Token::Register(t) => t.span.clone(),
            Token::Static(t) => t.span.clone(),
            Token::Extern(t) => t.span.clone(),
            Token::Typedef(t) => t.span.clone(),
            Token::Const(t) => t.span.clone(),
            Token::Volatile(t) => t.span.clone(),
            Token::Restrict(t) => t.span.clone(),
            Token::Atomic(t) => t.span.clone(),
            Token::Int(t) => t.span.clone(),
            Token::Char(t) => t.span.clone(),
            Token::Float(t) => t.span.clone(),
            Token::Double(t) => t.span.clone(),
            Token::Void(t) => t.span.clone(),
            Token::Long(t) => t.span.clone(),
            Token::Short(t) => t.span.clone(),
            Token::Signed(t) => t.span.clone(),
            Token::Unsigned(t) => t.span.clone(),
            Token::Struct(t) => t.span.clone(),
            Token::Enum(t) => t.span.clone(),
            Token::Union(t) => t.span.clone(),
            Token::LeftBrace(t) => t.span.clone(),
            Token::RightBrace(t) => t.span.clone(),
            Token::LeftParen(t) => t.span.clone(),
            Token::RightParen(t) => t.span.clone(),
            Token::Return(t) => t.span.clone(),
            Token::IfKeyword(t) => t.span.clone(),
            Token::ElseKeyword(t) => t.span.clone(),
            Token::While(t) => t.span.clone(),
            Token::For(t) => t.span.clone(),
        }
    }
}

