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
pub struct IdentToken<'a> {
    pub span: Span,
    pub name: &'a str,
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

// Conditional compilation directive tokens
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

// トークンの enum
#[derive(Debug)]
pub enum Token<'a> {
    BlockComment(BlockCommentToken),
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
    Ident(IdentToken<'a>),
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
}

