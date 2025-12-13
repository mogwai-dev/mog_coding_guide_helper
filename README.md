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

### 想定プロジェクト構成

このツールは以下のようなC言語プロジェクト構成を想定しています：

```
Project/
├── coding-guide.toml        # プロジェクト設定ファイル
├── main.c                   # メインファイル（オプション）
├── Project.ino              # Arduinoプロジェクトの場合（オプション）
├── include/                 # ヘッダーファイル（推奨）
│   ├── layer1/
│   │   ├── component1.h
│   │   └── component2.h
│   └── layer2/
│       ├── component3.h
│       └── component4.h
└── src/                     # ソースファイル
    ├── main.c               # メインファイル（オプション）
    ├── layer1/
    │   ├── component1.c
    │   └── component2.c
    └── layer2/
        ├── component3.c
        └── component4.c
```

**ヘッダーファイルについて:**
- `include/` ディレクトリに配置することを推奨しますが、必須ではありません
- ヘッダーファイルも `.h` 拡張子で自動認識され、診断対象になります
- `src/` と同じ階層構造で管理すると、対応関係が明確になります

### プロジェクト設定ファイル

プロジェクトルートに `coding-guide.toml` を配置することで、診断ルールやフォーマット設定をカスタマイズできます。

**coding-guide.toml の例:**
```toml
[diagnostics]
check_file_header = true
check_function_format = true
check_type_safety = true
check_storage_class_order = true

[file_header]
required_fields = ["Author", "Date", "Purpose"]

[formatting]
add_file_header = true
use_tabs = true  # 4スペースをタブに変換
```

**設定の説明:**

- `[diagnostics]` - 診断ルールの有効/無効
  - `check_file_header`: ファイルヘッダーコメントの確認 (CGH001)
  - `check_function_format`: 関数フォーマットのチェック (CGH002)
  - `check_type_safety`: 型安全性の警告 (CGH003)
  - `check_storage_class_order`: 記憶域クラス指定子の順序チェック (CGH004)

- `[file_header]` - ファイルヘッダーの要件
  - `required_fields`: 必須フィールドのリスト
  - `template`: ヘッダーテンプレート（オプション）

- `[formatting]` - フォーマット動作
  - `add_file_header`: ファイルヘッダーの自動追加
  - `use_tabs`: 4スペースをタブに変換（デフォルト: `false`）
    - `true`: 行頭の4スペースごとに1タブに変換、余りはスペースのまま
    - ブロック内のネストしたインデントも正しく処理されます

設定ファイルが見つからない場合、すべての診断が有効なデフォルト設定が使用されます。

### コマンドライン

```bash
# サンプル実行（カレントディレクトリから設定を検索）
cargo run --package coding-guide-helper

# ファイルを指定（ファイルのディレクトリから設定を検索）
cargo run --package coding-guide-helper -- src/main.c

# サブディレクトリ内のファイルも自動的にプロジェクトルートを検索
cargo run --package coding-guide-helper -- src/layer1/component1.c

# プロジェクトルートを明示的に指定
cargo run --package coding-guide-helper -- --project-root /path/to/Project src/layer2/component3.c

# 複数ファイルを順次チェック（シェルスクリプト例）
find src -name "*.c" -exec cargo run --package coding-guide-helper -- {} \;
```

CLI は指定されたファイルのディレクトリから親ディレクトリへ遡って `coding-guide.toml` を検索します。`--project-root` オプションで検索開始位置を明示できます。

**使用例:**
```bash
# Project/src/layer1/component1.c をチェック
cd Project
cargo run --package coding-guide-helper -- src/layer1/component1.c
# → Project/coding-guide.toml が自動検出されます
```

### VSCode拡張

1. LSPサーバーをビルド
2. `editors/vscode`で`npm install`
3. `npm run compile`
4. F5で拡張をデバッグ実行

LSP サーバーはワークスペースルートから `coding-guide.toml` を自動的に読み込みます。設定ファイルがない場合はデフォルト設定が使用されます。

**VSCodeでのワークスペース例:**
```
Project/  ← ワークスペースルート
├── coding-guide.toml  ← 自動検出される
├── src/
│   ├── main.c         ← 開いたファイルすべてに設定が適用
│   └── layer1/
│       └── component1.c
└── include/
    └── layer1/
        └── component1.h
```

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

### 他のエディタ（vim/nvim/Eclipse等）

LSP標準プロトコルに準拠しているため、LSP対応エディタであれば使用可能です。各エディタがワークスペースルートを自動的に検出し、`coding-guide.toml` が適用されます。

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

### サポートしていない機能

**ダイグラフ (Digraphs) とトライグラフ (Trigraphs)**:
- C言語標準では特定の文字の組み合わせを別の文字に置き換える機能がありますが、本実装では**サポートしていません**
- 例: `<:` → `[`, `??=` → `#` など
- 理由: 現代のC言語開発では使用されることがまれであり、実用上の優先度が低いため

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
