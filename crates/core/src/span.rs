// ルートとノードを定義。所有する Span を持たせる（ライフタイム回避のため String/span を所有）
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub byte_start_idx: usize, // オリジナルの文字列におけるバイトオフセット
    pub byte_end_idx: usize,   // オリジナルの文字列におけるバイトオフセットの終端
}
