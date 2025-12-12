//! C言語の型システム（ポインタ、修飾子、基本型を含む）
//!
//! このモジュールは以下のC言語の型表現を定義する:
//! - 基本型 (void, char, int, long, など)
//! - 型修飾子 (const, volatile, restrict, atomic)
//! - ポインタ層 (各層に修飾子を付与可能)
//! - 完全な型表現 (基本型 + ポインタ層)

use crate::span::Span;

/// C言語の基本型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaseType {
    Void,
    Char,
    Short,
    Int,
    Long,
    LongLong,
    Float,
    Double,
    LongDouble,
    Signed,
    Unsigned,
    Bool,
    /// 構造体型（名前付きまたは匿名）
    /// Option<String>: Some(name) for named struct, None for anonymous
    Struct(Option<String>),
    /// 共用体型（名前付きまたは匿名）
    Union(Option<String>),
    /// 列挙型（名前付きまたは匿名）
    Enum(Option<String>),
}

impl BaseType {
    /// 基本型を文字列表現に変換
    pub fn to_string(&self) -> &str {
        match self {
            BaseType::Void => "void",
            BaseType::Char => "char",
            BaseType::Short => "short",
            BaseType::Int => "int",
            BaseType::Long => "long",
            BaseType::LongLong => "long long",
            BaseType::Float => "float",
            BaseType::Double => "double",
            BaseType::LongDouble => "long double",
            BaseType::Signed => "signed",
            BaseType::Unsigned => "unsigned",
            BaseType::Bool => "_Bool",
            BaseType::Struct(Some(name)) => return Box::leak(format!("struct {}", name).into_boxed_str()),
            BaseType::Struct(None) => "struct",
            BaseType::Union(Some(name)) => return Box::leak(format!("union {}", name).into_boxed_str()),
            BaseType::Union(None) => "union",
            BaseType::Enum(Some(name)) => return Box::leak(format!("enum {}", name).into_boxed_str()),
            BaseType::Enum(None) => "enum",
        }
    }
}

/// C言語の型修飾子
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeQualifier {
    Const,
    Volatile,
    Restrict,
    Atomic,
}

impl TypeQualifier {
    /// 修飾子を文字列表現に変換
    pub fn to_string(&self) -> &str {
        match self {
            TypeQualifier::Const => "const",
            TypeQualifier::Volatile => "volatile",
            TypeQualifier::Restrict => "restrict",
            TypeQualifier::Atomic => "_Atomic",
        }
    }
}

/// 単一のポインタ層（修飾子付き）
///
/// 例: `int *const *volatile ptr` の場合:
/// - Layer 0 (最内層): qualifiers = []
/// - Layer 1: qualifiers = [Const]
/// - Layer 2 (最外層): qualifiers = [Volatile]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointerLayer {
    /// このポインタ層に適用される修飾子
    pub qualifiers: Vec<TypeQualifier>,
    /// アスタリスクと修飾子をカバーするSpan
    pub span: Span,
}

impl PointerLayer {
    /// 修飾子なしのポインタ層を新規作成
    pub fn new(span: Span) -> Self {
        PointerLayer {
            qualifiers: Vec::new(),
            span,
        }
    }

    /// 修飾子付きのポインタ層を新規作成
    pub fn with_qualifiers(qualifiers: Vec<TypeQualifier>, span: Span) -> Self {
        PointerLayer { qualifiers, span }
    }

    /// このポインタ層が特定の修飾子を持つか確認
    pub fn has_qualifier(&self, qualifier: TypeQualifier) -> bool {
        self.qualifiers.contains(&qualifier)
    }

    /// ポインタ層を文字列表現に変換
    pub fn to_string(&self) -> String {
        if self.qualifiers.is_empty() {
            "*".to_string()
        } else {
            let quals = self
                .qualifiers
                .iter()
                .map(|q| q.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            format!("*{}", quals)
        }
    }
}

/// 完全なC言語の型表現
///
/// 例: `const int *const *volatile ptr`
/// - base_type: Int
/// - base_qualifiers: [Const]
/// - pointer_layers: [
///     PointerLayer { qualifiers: [] },           // 最内層の *
///     PointerLayer { qualifiers: [Const] },      // *const
///     PointerLayer { qualifiers: [Volatile] }    // *volatile (最外層)
///   ]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type {
    /// 基本型 (int, char, など)
    pub base_type: BaseType,
    /// 基本型に適用される修飾子
    pub base_qualifiers: Vec<TypeQualifier>,
    /// ポインタ層 (最内層から順)
    pub pointer_layers: Vec<PointerLayer>,
    /// 型全体をカバーするSpan
    pub span: Span,
}

impl Type {
    /// 非ポインタ型を新規作成
    pub fn new(base_type: BaseType, span: Span) -> Self {
        Type {
            base_type,
            base_qualifiers: Vec::new(),
            pointer_layers: Vec::new(),
            span,
        }
    }

    /// 基本型修飾子付きの型を新規作成
    pub fn with_base_qualifiers(
        base_type: BaseType,
        base_qualifiers: Vec<TypeQualifier>,
        span: Span,
    ) -> Self {
        Type {
            base_type,
            base_qualifiers,
            pointer_layers: Vec::new(),
            span,
        }
    }

    /// ポインタ型を新規作成
    pub fn with_pointers(
        base_type: BaseType,
        base_qualifiers: Vec<TypeQualifier>,
        pointer_layers: Vec<PointerLayer>,
        span: Span,
    ) -> Self {
        Type {
            base_type,
            base_qualifiers,
            pointer_layers,
            span,
        }
    }

    /// ポインタ型かどうか確認
    pub fn is_pointer(&self) -> bool {
        !self.pointer_layers.is_empty()
    }

    /// ポインタ階層を取得 (非ポインタは0, *は1, **は2, など)
    pub fn pointer_level(&self) -> usize {
        self.pointer_layers.len()
    }

    /// 基本型が特定の修飾子を持つか確認
    pub fn has_base_qualifier(&self, qualifier: TypeQualifier) -> bool {
        self.base_qualifiers.contains(&qualifier)
    }

    /// 型を文字列表現に変換
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        // 基本型修飾子
        if !self.base_qualifiers.is_empty() {
            let quals = self
                .base_qualifiers
                .iter()
                .map(|q| q.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            result.push_str(&quals);
            result.push(' ');
        }

        // 基本型
        result.push_str(self.base_type.to_string());

        // ポインタ層
        for layer in &self.pointer_layers {
            result.push(' ');
            result.push_str(&layer.to_string());
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_pointer() {
        // int *
        let span = Span::new(0, 0, 0, 5);
        let pointer_layer = PointerLayer::new(span.clone());
        let ty = Type::with_pointers(
            BaseType::Int,
            Vec::new(),
            vec![pointer_layer],
            span,
        );

        assert!(ty.is_pointer());
        assert_eq!(ty.pointer_level(), 1);
        assert_eq!(ty.to_string(), "int *");
    }

    #[test]
    fn test_const_pointer() {
        // int *const
        let span = Span::new(0, 0, 0, 11);
        let pointer_layer = PointerLayer::with_qualifiers(
            vec![TypeQualifier::Const],
            span.clone(),
        );
        let ty = Type::with_pointers(
            BaseType::Int,
            Vec::new(),
            vec![pointer_layer],
            span,
        );

        assert!(ty.is_pointer());
        assert_eq!(ty.pointer_level(), 1);
        assert!(ty.pointer_layers[0].has_qualifier(TypeQualifier::Const));
        assert_eq!(ty.to_string(), "int *const");
    }

    #[test]
    fn test_complex_pointer() {
        // const int *const *volatile
        let span = Span::new(0, 0, 0, 27);
        let layer1 = PointerLayer::with_qualifiers(
            vec![TypeQualifier::Const],
            span.clone(),
        );
        let layer2 = PointerLayer::with_qualifiers(
            vec![TypeQualifier::Volatile],
            span.clone(),
        );
        let ty = Type::with_pointers(
            BaseType::Int,
            vec![TypeQualifier::Const],
            vec![layer1, layer2],
            span,
        );

        assert!(ty.is_pointer());
        assert_eq!(ty.pointer_level(), 2);
        assert!(ty.has_base_qualifier(TypeQualifier::Const));
        assert!(ty.pointer_layers[0].has_qualifier(TypeQualifier::Const));
        assert!(ty.pointer_layers[1].has_qualifier(TypeQualifier::Volatile));
        assert_eq!(ty.to_string(), "const int *const *volatile");
    }
}
