# ポインタ型サポート実装計画

## 概要

C言語のポインタ型（`int *ptr`, `char **argv`, `const int * const ptr`など）を完全にサポートするための段階的実装計画。

現在の最大の課題は、Rustのライフタイム制約により`Token<'a>`が文字列を借用しているため、トークンバッファリングができないこと。

---

## Phase 1: Token設計の変更（Foundation）

### 目標
ライフタイム問題を解決するため、`Token`を所有型に変更する

### Step 1.1: IdentTokenをString所有に変更

**現在**:
```rust
pub struct IdentToken<'a> {
    pub span: Span,
    pub name: &'a str,  // 借用
}
pub enum Token<'a> { /* ... */ }
```

**変更後**:
```rust
pub struct IdentToken {
    pub span: Span,
    pub name: String,  // 所有
}
pub enum Token { /* ... */ }  // ライフタイムパラメータ削除
```

**影響範囲**:
- `crates/core/src/token.rs` - Token定義
- `crates/core/src/lexer.rs` - トークン生成時に`.to_string()`
- `crates/core/src/parser.rs` - `Parser<'a>` → `Parser`

### Step 1.2: LexerとParserのライフタイム削除

**変更**:
```rust
// Before
pub struct Lexer<'a> { input: &'a str, /* ... */ }
pub struct Parser<'a> { lexer: Lexer<'a>, /* ... */ }

// After
pub struct Lexer { input: String, /* ... */ }
pub struct Parser { lexer: Lexer, /* ... */ }
```

**利点**:
- トークンバッファリングが可能（`Vec<Token>`を内部保持できる）
- 先読み（lookahead）が実装できる
- ライフタイムエラーが発生しない

**欠点**:
- 文字列コピーのオーバーヘッド（小規模ファイルなら問題なし）

### 実装チェックリスト
- [ ] `IdentToken`の`name`を`String`に変更
- [ ] `Token<'a>`から`<'a>`を削除
- [ ] Lexerで識別子生成時に`.to_string()`を追加
- [ ] `Lexer<'a>`を`Lexer`に変更、`input: String`に
- [ ] `Lexer::new()`を`Lexer::new(input: &str)`に変更、内部で`.to_string()`
- [ ] `Parser<'a>`を`Parser`に変更
- [ ] 全テストが通ることを確認
- [ ] `cargo build`でwarningが出ないことを確認

---

## Phase 2: Asteriskトークンの追加

### 目標
`*`記号を認識できるようにする

### Step 2.1: AsteriskTokenの定義

**追加コード** (`crates/core/src/token.rs`):
```rust
#[derive(Debug, Clone)]
pub struct AsteriskToken {
    pub span: Span,
}

pub enum Token {
    // ... 既存のトークン
    Asterisk(AsteriskToken),
    // ...
}
```

### Step 2.2: Lexerに`*`の処理を追加

**追加場所**: `crates/core/src/lexer.rs`の`next_token()`メソッド内

```rust
Some((byte_idx, '*')) => {
    if start_byte_flag.is_none() {
        start_byte_flag = Some(byte_idx);
    }
    self.next_char();

    let end_byte = if let Some((b, _)) = self.peeked {
        b
    } else {
        self.input.len()
    };

    return Some(Token::Asterisk(AsteriskToken {
        span: Span {
            start_line,
            start_column,
            end_line: self.line,
            end_column: self.column,
            byte_start_idx: start_byte_flag.unwrap(),
            byte_end_idx: end_byte,
        }
    }));
},
```

**挿入位置**: `'='`の処理の後、`'{'`の処理の前

### Step 2.3: CLIでの表示対応

**追加コード** (`crates/cli/src/main.rs`の`lexer_sample()`):
```rust
Token::Asterisk(AsteriskToken { span }) => {
    println!("Asterisk from ({}, {}) to ({}, {}): {:?}", 
        span.start_line, span.start_column, span.end_line, span.end_column, 
        &contents[span.byte_start_idx..span.byte_end_idx]);
},
```

### テスト

**テストファイル作成** (`test_pointer.c`):
```c
int *ptr;
char *str;
void *voidptr;

int* func(int *arg) {
    return arg;
}
```

**実行**:
```bash
cargo run test_pointer.c
```

**期待される出力**:
```
[Lexer Sample]
Type specifier from (0, 0) to (0, 3): "int"
Asterisk from (0, 4) to (0, 5): "*"
Ident from (0, 5) to (0, 8): "ptr" (name: ptr)
Semicolon from (0, 8) to (0, 9): ";"
```

### 実装チェックリスト
- [ ] `AsteriskToken`構造体を定義
- [ ] `Token::Asterisk`バリアントを追加
- [ ] Lexerに`*`マッチング処理を追加
- [ ] CLIに`Asterisk`表示処理を追加
- [ ] `test_pointer.c`で動作確認
- [ ] 既存テストが通ることを確認

---

## Phase 3: 型システムモジュールの作成

### 目標
C言語の型を表現する構造体を定義する

### Step 3.1: type_system.rsモジュール作成

**新規ファイル**: `crates/core/src/type_system.rs`

```rust
use crate::span::Span;

/// 基本型（型指定子）
#[derive(Debug, Clone, PartialEq)]
pub enum BaseType {
    Void,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Signed,
    Unsigned,
    // 複合型
    Struct(Option<String>),  // struct name
    Union(Option<String>),   // union name
    Enum(Option<String>),    // enum name
    // typedef名
    TypedefName(String),
}

/// 型修飾子
#[derive(Debug, Clone, PartialEq)]
pub enum TypeQualifier {
    Const,
    Volatile,
    Restrict,
    Atomic,
}

/// ポインタの層（各層ごとの修飾子を持つ）
#[derive(Debug, Clone)]
pub struct PointerLayer {
    pub qualifiers: Vec<TypeQualifier>,  // const, volatile など
    pub span: Option<Span>,
}

/// C言語の型表現
#[derive(Debug, Clone)]
pub struct Type {
    /// 基本型
    pub base_type: BaseType,
    
    /// 基本型への修飾子（const int など）
    pub base_qualifiers: Vec<TypeQualifier>,
    
    /// ポインタの層（外側から順に）
    /// 例: int *const *volatile の場合
    /// [PointerLayer { qualifiers: [Volatile] }, PointerLayer { qualifiers: [Const] }]
    pub pointer_layers: Vec<PointerLayer>,
    
    /// 配列次元（将来対応）
    pub array_dimensions: Vec<Option<usize>>,
    
    pub span: Span,
}

impl Type {
    /// ポインタかどうか
    pub fn is_pointer(&self) -> bool {
        !self.pointer_layers.is_empty()
    }
    
    /// ポインタのレベル（*, **, *** など）
    pub fn pointer_level(&self) -> usize {
        self.pointer_layers.len()
    }
    
    /// 基本型が const かどうか
    pub fn is_base_const(&self) -> bool {
        self.base_qualifiers.contains(&TypeQualifier::Const)
    }
    
    /// 最外のポインタが const かどうか
    pub fn is_outer_pointer_const(&self) -> bool {
        self.pointer_layers.first()
            .map(|layer| layer.qualifiers.contains(&TypeQualifier::Const))
            .unwrap_or(false)
    }
    
    /// 型を文字列として再構築（デバッグ用）
    pub fn to_string(&self) -> String {
        let mut result = String::new();
        
        // 基本型の修飾子
        for q in &self.base_qualifiers {
            result.push_str(&format!("{:?} ", q).to_lowercase());
        }
        
        // 基本型
        result.push_str(&match &self.base_type {
            BaseType::Void => "void".to_string(),
            BaseType::Char => "char".to_string(),
            BaseType::Short => "short".to_string(),
            BaseType::Int => "int".to_string(),
            BaseType::Long => "long".to_string(),
            BaseType::Float => "float".to_string(),
            BaseType::Double => "double".to_string(),
            BaseType::Signed => "signed".to_string(),
            BaseType::Unsigned => "unsigned".to_string(),
            BaseType::Struct(name) => format!("struct {}", name.as_ref().unwrap_or(&"".to_string())),
            BaseType::Union(name) => format!("union {}", name.as_ref().unwrap_or(&"".to_string())),
            BaseType::Enum(name) => format!("enum {}", name.as_ref().unwrap_or(&"".to_string())),
            BaseType::TypedefName(name) => name.clone(),
        });
        
        // ポインタ層（逆順で出力）
        for layer in self.pointer_layers.iter().rev() {
            result.push_str(" *");
            for q in &layer.qualifiers {
                result.push_str(&format!(" {:?}", q).to_lowercase());
            }
        }
        
        result
    }
}
```

### Step 3.2: lib.rsへの登録

**追加** (`crates/core/src/lib.rs`):
```rust
pub mod type_system;

// re-export
pub use type_system::{Type, BaseType, TypeQualifier, PointerLayer};
```

### Step 3.3: テストケース追加

**追加** (`crates/core/src/type_system.rs`の末尾):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_int_pointer() {
        let ty = Type {
            base_type: BaseType::Int,
            base_qualifiers: vec![],
            pointer_layers: vec![PointerLayer { qualifiers: vec![], span: None }],
            array_dimensions: vec![],
            span: Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 },
        };
        
        assert_eq!(ty.is_pointer(), true);
        assert_eq!(ty.pointer_level(), 1);
        assert_eq!(ty.to_string(), "int *");
    }
    
    #[test]
    fn test_const_pointer_to_const_int() {
        // const int * const ptr;
        let ty = Type {
            base_type: BaseType::Int,
            base_qualifiers: vec![TypeQualifier::Const],
            pointer_layers: vec![PointerLayer { 
                qualifiers: vec![TypeQualifier::Const], 
                span: None 
            }],
            array_dimensions: vec![],
            span: Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 },
        };
        
        assert_eq!(ty.is_base_const(), true);
        assert_eq!(ty.is_outer_pointer_const(), true);
        assert_eq!(ty.to_string(), "const int * const");
    }
    
    #[test]
    fn test_complex_pointer() {
        // int *const *volatile ptr;
        let ty = Type {
            base_type: BaseType::Int,
            base_qualifiers: vec![],
            pointer_layers: vec![
                PointerLayer { qualifiers: vec![TypeQualifier::Volatile], span: None },
                PointerLayer { qualifiers: vec![TypeQualifier::Const], span: None },
            ],
            array_dimensions: vec![],
            span: Span { start_line: 0, start_column: 0, end_line: 0, end_column: 0, byte_start_idx: 0, byte_end_idx: 0 },
        };
        
        assert_eq!(ty.pointer_level(), 2);
        assert_eq!(ty.to_string(), "int * const * volatile");
    }
}
```

### 実装チェックリスト
- [ ] `type_system.rs`モジュール作成
- [ ] `BaseType`, `TypeQualifier`, `PointerLayer`, `Type`の定義
- [ ] `Type`のヘルパーメソッド実装
- [ ] `lib.rs`にモジュール登録
- [ ] テストケース追加
- [ ] `cargo test`で3つのテストが通ることを確認

---

## Phase 4: Parserでの型解析（基本型のみ）

### 目標
ポインタなしの基本型をパースする（`int`, `char`, `void`など）

### Step 4.1: parse_type()メソッド実装（第1版）

**追加** (`crates/core/src/parser.rs`):
```rust
use crate::type_system::{Type, BaseType, TypeQualifier, PointerLayer};

impl Parser {
    /// 型をパースする（第1版：基本型のみ、ポインタは未対応）
    /// 
    /// # 引数
    /// - first_token: 型の最初のトークン（型指定子や修飾子）
    /// 
    /// # 戻り値
    /// - Some((Type, next_token)): パース成功。next_tokenは型の次のトークン
    /// - None: 型として認識できなかった
    fn parse_type(&mut self, first_token: Token) -> Option<(Type, Token)> {
        let start_span = Self::get_token_span(&first_token)?;
        let mut base_type: Option<BaseType> = None;
        let mut base_qualifiers = Vec::new();
        let mut current_token = first_token;
        
        // 型指定子と修飾子を収集
        loop {
            match &current_token {
                Token::Const(_) => {
                    base_qualifiers.push(TypeQualifier::Const);
                },
                Token::Volatile(_) => {
                    base_qualifiers.push(TypeQualifier::Volatile);
                },
                Token::Restrict(_) => {
                    base_qualifiers.push(TypeQualifier::Restrict);
                },
                Token::Atomic(_) => {
                    base_qualifiers.push(TypeQualifier::Atomic);
                },
                Token::Void(_) if base_type.is_none() => {
                    base_type = Some(BaseType::Void);
                },
                Token::Char(_) if base_type.is_none() => {
                    base_type = Some(BaseType::Char);
                },
                Token::Short(_) if base_type.is_none() => {
                    base_type = Some(BaseType::Short);
                },
                Token::Int(_) if base_type.is_none() => {
                    base_type = Some(BaseType::Int);
                },
                Token::Long(_) if base_type.is_none() => {
                    base_type = Some(BaseType::Long);
                },
                Token::Float(_) if base_type.is_none() => {
                    base_type = Some(BaseType::Float);
                },
                Token::Double(_) if base_type.is_none() => {
                    base_type = Some(BaseType::Double);
                },
                Token::Signed(_) if base_type.is_none() => {
                    base_type = Some(BaseType::Signed);
                },
                Token::Unsigned(_) if base_type.is_none() => {
                    base_type = Some(BaseType::Unsigned);
                },
                _ => {
                    // 型の一部ではない → ここで終了
                    break;
                }
            }
            
            // 次のトークンを取得
            current_token = self.lexer.next_token()?;
        }
        
        // 基本型が見つからなかった場合はエラー
        let base_type = base_type?;
        
        Some((Type {
            base_type,
            base_qualifiers,
            pointer_layers: vec![],  // Phase 5で実装
            array_dimensions: vec![],
            span: start_span,
        }, current_token))
    }
    
    /// トークンからSpanを取得するヘルパー
    fn get_token_span(token: &Token) -> Option<Span> {
        match token {
            Token::Const(t) => Some(t.span.clone()),
            Token::Volatile(t) => Some(t.span.clone()),
            Token::Restrict(t) => Some(t.span.clone()),
            Token::Atomic(t) => Some(t.span.clone()),
            Token::Void(t) => Some(t.span.clone()),
            Token::Char(t) => Some(t.span.clone()),
            Token::Short(t) => Some(t.span.clone()),
            Token::Int(t) => Some(t.span.clone()),
            Token::Long(t) => Some(t.span.clone()),
            Token::Float(t) => Some(t.span.clone()),
            Token::Double(t) => Some(t.span.clone()),
            Token::Signed(t) => Some(t.span.clone()),
            Token::Unsigned(t) => Some(t.span.clone()),
            Token::Struct(t) => Some(t.span.clone()),
            Token::Union(t) => Some(t.span.clone()),
            Token::Enum(t) => Some(t.span.clone()),
            _ => None,
        }
    }
}
```

### Step 4.2: テスト追加

**追加** (`crates/core/src/parser.rs`):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_basic_int_type() {
        let code = "int";
        let lexer = Lexer::new(code);
        let mut parser = Parser::new(lexer);
        
        let first_token = parser.lexer.next_token().unwrap();
        let result = parser.parse_type(first_token);
        
        assert!(result.is_some());
        let (ty, _) = result.unwrap();
        assert_eq!(ty.base_type, BaseType::Int);
        assert_eq!(ty.is_pointer(), false);
    }
    
    #[test]
    fn test_parse_const_int_type() {
        let code = "const int";
        let lexer = Lexer::new(code);
        let mut parser = Parser::new(lexer);
        
        let first_token = parser.lexer.next_token().unwrap();
        let result = parser.parse_type(first_token);
        
        assert!(result.is_some());
        let (ty, _) = result.unwrap();
        assert_eq!(ty.base_type, BaseType::Int);
        assert_eq!(ty.is_base_const(), true);
    }
}
```

### 実装チェックリスト
- [ ] `parse_type()`メソッド実装（基本型のみ）
- [ ] `get_token_span()`ヘルパー実装
- [ ] テストケース追加
- [ ] `cargo test`でテストが通ることを確認
- [ ] 既存のパーサーテストが通ることを確認

---

## Phase 5: ポインタ層のパース

### 目標
`*`を含む型をパースする（`int *`, `char **`, `const int * const`など）

### Step 5.1: parse_type()の拡張

**修正** (`crates/core/src/parser.rs`の`parse_type()`):
```rust
fn parse_type(&mut self, first_token: Token) -> Option<(Type, Token)> {
    // ... 既存の基本型パース処理 ...
    
    // 基本型が見つからなかった場合はエラー
    let base_type = base_type?;
    
    // ★ ポインタ層のパース（ここから追加）
    let mut pointer_layers = Vec::new();
    
    loop {
        // `*`をチェック
        if let Token::Asterisk(asterisk_token) = &current_token {
            let pointer_span = asterisk_token.span.clone();
            let mut qualifiers = Vec::new();
            
            // ポインタ後の修飾子を収集
            loop {
                current_token = self.lexer.next_token()?;
                
                match &current_token {
                    Token::Const(_) => {
                        qualifiers.push(TypeQualifier::Const);
                    },
                    Token::Volatile(_) => {
                        qualifiers.push(TypeQualifier::Volatile);
                    },
                    Token::Restrict(_) => {
                        qualifiers.push(TypeQualifier::Restrict);
                    },
                    Token::Atomic(_) => {
                        qualifiers.push(TypeQualifier::Atomic);
                    },
                    _ => {
                        // 修飾子ではない → このポインタ層は終了
                        break;
                    }
                }
            }
            
            // ポインタ層を追加（外側から順に）
            pointer_layers.push(PointerLayer {
                qualifiers,
                span: Some(pointer_span),
            });
        } else {
            // `*`ではない → ポインタ解析終了
            break;
        }
    }
    
    Some((Type {
        base_type,
        base_qualifiers,
        pointer_layers,
        array_dimensions: vec![],
        span: start_span,
    }, current_token))
}
```

### Step 5.2: テスト追加

**追加** (`crates/core/src/parser.rs`):
```rust
#[test]
fn test_parse_int_pointer() {
    let code = "int *";
    let lexer = Lexer::new(code);
    let mut parser = Parser::new(lexer);
    
    let first_token = parser.lexer.next_token().unwrap();
    let result = parser.parse_type(first_token);
    
    assert!(result.is_some());
    let (ty, _) = result.unwrap();
    assert_eq!(ty.base_type, BaseType::Int);
    assert_eq!(ty.is_pointer(), true);
    assert_eq!(ty.pointer_level(), 1);
    assert_eq!(ty.to_string(), "int *");
}

#[test]
fn test_parse_double_pointer() {
    let code = "char **";
    let lexer = Lexer::new(code);
    let mut parser = Parser::new(lexer);
    
    let first_token = parser.lexer.next_token().unwrap();
    let result = parser.parse_type(first_token);
    
    assert!(result.is_some());
    let (ty, _) = result.unwrap();
    assert_eq!(ty.pointer_level(), 2);
}

#[test]
fn test_parse_const_pointer() {
    let code = "int * const";
    let lexer = Lexer::new(code);
    let mut parser = Parser::new(lexer);
    
    let first_token = parser.lexer.next_token().unwrap();
    let result = parser.parse_type(first_token);
    
    assert!(result.is_some());
    let (ty, _) = result.unwrap();
    assert_eq!(ty.is_outer_pointer_const(), true);
}
```

### 実装チェックリスト
- [ ] `parse_type()`にポインタ層解析を追加
- [ ] 修飾子付きポインタの処理を実装
- [ ] テストケース追加（単純ポインタ、二重ポインタ、const修飾）
- [ ] `cargo test`で新しいテストが通ることを確認
- [ ] 実際の宣言文でテスト（`int *ptr;`など）

---

## Phase 6: ASTの型情報統合

### 目標
AST ItemsにTypeフィールドを追加して、パース結果を保存する

### Step 6.1: VarDeclに型情報追加

**修正** (`crates/core/src/ast.rs`):
```rust
VarDecl { 
    span: Span, 
    text: String,
    var_type: Option<Type>,  // ★ 追加
    var_name: String,
    has_initializer: bool,
    trivia: Trivia,
},
```

### Step 6.2: ParserでVarDeclをパース時に型情報を取得

**修正** (`crates/core/src/parser.rs`の変数宣言パース部分):
```rust
Token::Int(token) | Token::Char(token) | /* ... */ => {
    // ★ 型解析を試みる
    let type_result = self.parse_type(token.clone());
    let (parsed_type, next_token) = if let Some(result) = type_result {
        (Some(result.0), result.1)
    } else {
        (None, token)  // フォールバック
    };
    
    // 変数名の取得（next_tokenまたは後続トークンから）
    let mut var_name = String::new();
    // ... 既存の変数名取得ロジック ...
    
    items.push(Item::VarDecl {
        span: final_span,
        text,
        var_type: parsed_type,  // ★ 追加
        var_name,
        has_initializer,
        trivia: self.take_trivia(),
    });
}
```

### Step 6.3: FunctionDeclの拡張（オプション）

将来的に関数の型情報も扱う場合：

```rust
FunctionDecl {
    span: Span,
    return_type: Option<Type>,  // ★ Stringから変更
    function_name: String,
    parameters: Vec<Parameter>,  // ★ 新しい構造体
    storage_class: Option<String>,
    trivia: Trivia,
},

pub struct Parameter {
    pub param_type: Type,
    pub name: Option<String>,
    pub span: Span,
}
```

### 実装チェックリスト
- [ ] `ast.rs`の`VarDecl`に`var_type`フィールド追加
- [ ] `parser.rs`の変数宣言パースで`parse_type()`を呼び出し
- [ ] 型情報がASTに正しく格納されることをテスト
- [ ] Formatterを更新（型情報を使う場合）
- [ ] （オプション）`FunctionDecl`の拡張

---

## Phase 7: Diagnosticsでの型チェック

### 目標
ポインタ型に関するルールをチェックする診断機能を追加

### Step 7.1: void*の適切な使用チェック

**追加** (`crates/core/src/diagnostics.rs`):
```rust
fn check_void_pointer_usage(item: &Item, diagnostics: &mut Vec<Diagnostic>) {
    if let Item::VarDecl { var_type: Some(ty), span, .. } = item {
        if ty.base_type == BaseType::Void && ty.is_pointer() {
            // void* の演算チェックなど
            diagnostics.push(Diagnostic {
                code: "W_VOID_PTR".to_string(),
                severity: DiagnosticSeverity::Information,
                message: "void pointer detected. Be careful with pointer arithmetic.".to_string(),
                span: span.clone(),
            });
        }
    }
}
```

### Step 7.2: const correctnessチェック

**追加** (`crates/core/src/diagnostics.rs`):
```rust
fn check_const_correctness(tu: &TranslationUnit, diagnostics: &mut Vec<Diagnostic>) {
    // const int* への代入チェック
    // int* const の再代入チェック
    // など
}
```

### 実装チェックリスト
- [ ] `void*`使用の情報診断
- [ ] const修飾子の整合性チェック
- [ ] ポインタレベルの不一致警告
- [ ] テストケース追加

---

## Phase 8: Formatterでの型表示

### 目標
型情報を使って綺麗にフォーマットする

### Step 8.1: VarDeclのフォーマット更新

**修正** (`crates/core/src/formatter.rs`):
```rust
Item::VarDecl { var_type, var_name, has_initializer, .. } => {
    if let Some(ty) = var_type {
        // 型情報を使って整形
        let type_str = ty.to_string();
        if *has_initializer {
            format!("{} {} = ...", type_str, var_name)
        } else {
            format!("{} {};", type_str, var_name)
        }
    } else {
        // フォールバック: text を使う
        text.clone()
    }
}
```

### 実装チェックリスト
- [ ] `Type::to_string()`を使ったフォーマット
- [ ] 変数宣言の整形
- [ ] 関数宣言の整形（オプション）
- [ ] フォーマット結果のテスト

---

## マイルストーン

### Milestone 1: "基本型のパースまで"
**目標**: Phase 1-4を完了
- トークンが所有型になる
- `*`トークンが認識される
- 型システムモジュールが完成
- 基本型（ポインタなし）がパースされる

**完了条件**:
- [ ] `int x;` が `VarDecl { var_type: Some(Type { base: Int, ... }) }`
- [ ] `const char y;` の型情報が正しくパースされる
- [ ] 全既存テストが通る
- [ ] 新しいテスト3件以上が通る

### Milestone 2: "ポインタ型のパース"
**目標**: Phase 5を完了
- ポインタを含む型がパースされる

**完了条件**:
- [ ] `int *ptr;` が正しくパースされる
- [ ] `char **argv;` が2層ポインタとして認識される
- [ ] `const int * const p;` の修飾子が正しく配置される
- [ ] ポインタ関連のテスト5件以上が通る

### Milestone 3: "AST統合"
**目標**: Phase 6を完了
- ASTに型情報が統合される

**完了条件**:
- [ ] `VarDecl`に型情報が保存される
- [ ] Formatterが型情報を使う
- [ ] エンドツーエンドで動作確認

### Milestone 4: "診断機能"
**目標**: Phase 7-8を完了
- 型に基づく診断機能が動作する

**完了条件**:
- [ ] `void*`の警告が出る
- [ ] const関連のチェックが動作
- [ ] フォーマットが完全に動作

---

## リスク分析

| Phase | リスク | 影響度 | 対策 |
|-------|--------|--------|------|
| Phase 1 | 文字列コピーで性能低下 | 低 | 小規模ファイル想定なら問題なし。実測して確認 |
| Phase 1 | 既存コード全体への影響 | 高 | 段階的に変更、各ステップでテスト |
| Phase 5 | パースロジックの複雑性 | 中 | 小さなテストケースを多数追加 |
| Phase 6 | AST構造の変更 | 中 | `Option<Type>`にして段階移行 |

---

## 推奨実装順序

1. **Phase 1 → Phase 2** : まず基盤を整える
2. **Phase 3** : 型システムを独立して実装
3. **Phase 4 → Phase 5** : パーサーを段階的に拡張
4. **Phase 6** : AST統合
5. **Phase 7, 8** : 必要に応じて実装

---

## 参考資料

### C言語のポインタ宣言の読み方

```c
int *p;              // "p is a pointer to int"
int **p;             // "p is a pointer to pointer to int"
int *const p;        // "p is a const pointer to int"
const int *p;        // "p is a pointer to const int"
const int *const p;  // "p is a const pointer to const int"
int *volatile p;     // "p is a volatile pointer to int"
```

### ポインタ層の順序

C言語の宣言は「右から左」に読むが、パース時は「左から右」に進む。
内部表現では「外側から内側」の順序で保存する。

例: `int *const *volatile ptr`
- 最も外側: `*volatile` （volatile pointer）
- その次: `*const` （const pointer）
- 基本型: `int`

```rust
Type {
    base_type: Int,
    base_qualifiers: [],
    pointer_layers: [
        PointerLayer { qualifiers: [Volatile] },  // 外側
        PointerLayer { qualifiers: [Const] },     // 内側
    ]
}
```

---

## 完了チェックリスト

### Phase 1
- [ ] IdentToken を String 所有に変更
- [ ] Token のライフタイムパラメータ削除
- [ ] Lexer のライフタイムパラメータ削除
- [ ] Parser のライフタイムパラメータ削除
- [ ] 全テスト通過

### Phase 2
- [ ] AsteriskToken 定義
- [ ] Lexer に `*` 処理追加
- [ ] CLI に表示処理追加
- [ ] test_pointer.c で動作確認

### Phase 3
- [ ] type_system.rs 作成
- [ ] BaseType, TypeQualifier, PointerLayer, Type 定義
- [ ] ヘルパーメソッド実装
- [ ] テスト3件追加

### Phase 4
- [ ] parse_type() 実装（基本型のみ）
- [ ] get_token_span() 実装
- [ ] テスト2件追加

### Phase 5
- [ ] parse_type() にポインタ層解析追加
- [ ] テスト3件追加

### Phase 6
- [ ] VarDecl に var_type 追加
- [ ] Parser で parse_type() 呼び出し
- [ ] Formatter 更新

### Phase 7 & 8
- [ ] void* チェック追加
- [ ] const correctness チェック
- [ ] Formatter で型情報使用

---

## まとめ

この実装計画は段階的アプローチを採用しており、各Phaseは独立してテスト可能です。

最も重要なのは**Phase 1**（Token設計の変更）で、これがライフタイム問題の根本解決になります。Phase 1さえ完了すれば、残りのPhaseは比較的スムーズに実装できます。

各Phaseの完了時に必ずテストを実行し、リグレッションがないことを確認してください。
