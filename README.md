# Coding Guide Helper

C言語のコーディングガイドチェッカー＆フォーマッター

## 機能

- **診断**: コーディング規約違反を検出
  - CGH001: ファイルヘッダーコメントの確認
  - CGH002: 関数フォーマットのチェック

- **フォーマット**: コードの自動整形
  - ファイルヘッダーの自動追加
  - 空白の整理

**注意**: このツールはコーディングガイドチェックに特化しています。以下の機能は**既存のC言語LSP（clangd、C/C++ Extension等）との併用を推奨**します：
- シンボル定義へのジャンプ（Go to Definition）
- コード補完（IntelliSense）
- #include の解決
- リファクタリング

複数のLSPを同時に使用することで、コーディング規約チェックと開発支援機能の両方を活用できます。

## プロジェクト構成

```
coding_guide_helper/
├── crates/
│   ├── core/       # コアライブラリ（パーサー、フォーマッター等）
│   ├── cli/        # コマンドラインツール
│   └── lsp/        # LSPサーバー
└── editors/
    └── vscode/     # VSCode拡張
```

## ビルド

```bash
# 全体をビルド
cargo build --release

# LSPサーバーのみビルド
cargo build --release --package coding-guide-helper-lsp

# CLIツールのみビルド
cargo build --release --package coding-guide-helper
```

## 使用方法

### コマンドライン

```bash
# サンプル実行
cargo run --package coding-guide-helper

# ファイルを指定
cargo run --package coding-guide-helper -- example.c
```

### VSCode拡張

1. LSPサーバーをビルド
2. `editors/vscode`で`npm install`
3. `npm run compile`
4. F5で拡張をデバッグ実行

**推奨設定（settings.json）:**
```json
{
  // 既存のC言語LSP（シンボル解析・補完用）
  "C_Cpp.intelliSenseEngine": "default",
  // または clangd を使用
  // "clangd.path": "/path/to/clangd",
  
  // Coding Guide Helper（コーディング規約チェック用）
  "codingGuideHelper.enable": true
}
```

複数のLSPが同時に動作し、それぞれの機能を提供します。

## テスト

```bash
cargo test
```

## パフォーマンス測定

```bash
# ベンチマーク実行
cargo bench --package coding-guide-helper-core

# 結果はtarget/criterion/report/index.htmlで確認可能
```

## C言語標準からの仕様差分

このパーサーは一般的なC言語の構文を解析しますが、以下の点でC言語標準とは異なる実装となっています：

### 行継続（バックスラッシュ + 改行）

**C言語標準**: 
- プリプロセス段階で全ての `\` + 改行が削除される
- コード内のどこでも行継続が可能（文字列リテラル、識別子の途中など）

**本実装**:
- 行継続はプリプロセッサディレクティブ内（`#define`, `#include`, `#ifdef` など）でのみサポート
- **通常のC言語コード内でバックスラッシュを使用するとエラーになります**

**例**:
```c
// ✅ サポート（プリプロセッサディレクティブ内）
#define LONG_MACRO \
    VALUE_ON_NEXT_LINE

#ifdef CONDITION_A \
    && CONDITION_B

// ❌ エラー（通常のコード内）
int very_long_\
variable_name;  // エラー: Line continuation is not supported outside of preprocessor directives

int x = \
    10;  // エラー: バックスラッシュは使用できません
```

**エラーが発生した場合の出力例**:
```
[LEXER ERROR] Line 0, Column 14: Line continuation (backslash) is not supported outside of preprocessor directives. See README.md for details.
  Code: " \\\n"
```

パーサーはエラーを検出すると、行番号・列番号・該当コードを表示して処理を停止します。

**理由**: 
トークナイズ前に行継続を処理する必要があり、アーキテクチャの大幅な変更が必要になるため、実用上重要なプリプロセッサディレクティブ内のみに限定しています。

### 解析エラーについて

パーサーは構文エラーを検出した場合、エラーメッセージと共に処理を停止します。以下のような情報が表示されます：

- **エラー箇所**: 行番号、列番号
- **エラー内容**: 何が問題かの説明
- **該当コード**: エラーが発生した具体的なコード

複雑な構文エラーの場合、エラー箇所が実際の問題の直接の原因ではないことがあります。エラーメッセージから前後のコードも確認してください。

## ライセンス

MIT
