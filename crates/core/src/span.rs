// ルートとノードを定義。所有する Span を持たせる（ライフタイム回避のため String/span を所有）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub byte_start_idx: usize, // オリジナルの文字列におけるバイトオフセット
    pub byte_end_idx: usize,   // オリジナルの文字列におけるバイトオフセットの終端
}

impl Span {
    /// Create a new Span with line/column positions
    pub fn new(
        start_line: usize,
        start_column: usize,
        end_line: usize,
        end_column: usize,
    ) -> Self {
        Span {
            start_line,
            start_column,
            end_line,
            end_column,
            byte_start_idx: 0,
            byte_end_idx: 0,
        }
    }
}
