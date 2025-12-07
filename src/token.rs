use crate::span::Span;

#[derive(Debug)]
pub enum Token<'a> {
    BlockComment{
        span: Span,
    },
    Include{
        span: Span,
        filename: String,        // ファイル名 あとで &str にしたほうがメモリ効率がいいんじゃない？
    },
    Define{
        span: Span,
        macro_name: String,      // マクロ名 あとで &str にしたほうがメモリ効率がいいんじゃない？
        macro_value: String      // マクロ値 あとで &str にしたほうがメモリ効率がいいんじゃない？
    },
    Semicolon{
        span: Span,
    },
    Equal{
        span: Span,
    },
    Ident{
        span: Span,
        name: &'a str,
    },
    // 記憶域クラス指定子
    Auto{               // C 言語ではスタックに保存するという意味の記憶域クラス指定子がある。実際に使われることはないそう。
        span: Span,
    },
    Register{           // C 言語では汎用レジスタに保存するという意味の記憶域クラス指定子がある。コンパイラは無視することがあるそう。
        span: Span,
    },
    Static{
        span: Span,     // データセグメントに配置。プログラム開始から終了まで存在。
    },
    Extern{
        span: Span,     // 他のファイルに定義されていることを示す。
    },
    Typedef{
        span: Span,
    },
    // 型修飾子
    Const{
        span: Span,
    },
    Volatile{
        span: Span,
    },
    Restrict{           // todo: C99 以降のキーワードであることを警告して使わせないようにする
        span: Span,
    },
    _Atomic{            // todo: C11 以降のキーワードであることを明示する
        span: Span,
    },
    // 型指定子
    Int { span: Span },
    Char { span: Span },
    Float { span: Span },
    Double { span: Span },
    Void { span: Span },
    Long { span: Span },
    Short { span: Span },
    Signed { span: Span },
    Unsigned { span: Span },
    // 構造体関連
    Struct { span: Span },
    LeftBrace { span: Span },    // {
    RightBrace { span: Span },   // }
}
