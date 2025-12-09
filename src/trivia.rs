use crate::span::Span;

/// コメントの種類
#[derive(Debug, Clone, PartialEq)]
pub enum Comment {
    Line { text: String, span: Span },
    Block { text: String, span: Span },
}

/// Trivia: 宣言に付随するコメント
#[derive(Debug, Clone, PartialEq)]
pub struct Trivia {
    pub leading: Vec<Comment>,   // 前置コメント（宣言の前）
    pub trailing: Vec<Comment>,  // 後置コメント（宣言の後、同じ行）
}

impl Trivia {
    pub fn new() -> Self {
        Trivia {
            leading: Vec::new(),
            trailing: Vec::new(),
        }
    }

    pub fn empty() -> Self {
        Self::new()
    }
}

impl Default for Trivia {
    fn default() -> Self {
        Self::new()
    }
}
