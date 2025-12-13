# プロジェクト設定ファイル実装完了

## 実装内容

### 1. 設定ファイル構造 (`coding-guide.toml`)

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
```

### 2. 実装したモジュール

#### `crates/core/src/config.rs` (168行)
- **ProjectConfig**: プロジェクト全体の設定
  - DiagnosticsConfig: 診断ルールの有効/無効
  - FileHeaderConfig: ファイルヘッダー要件
  - FormattingConfig: フォーマット動作
  
- **find_and_load()**: ディレクトリツリーを遡って設定ファイルを検索
- **load_from_file()**: TOML ファイルをパース
- **to_diagnostic_config()**: DiagnosticConfig への変換

- すべての設定にデフォルト値あり（serde の `#[serde(default)]`）
- 設定ファイルが見つからない場合はデフォルト設定を使用

### 3. CLI 統合 (`crates/cli/src/main.rs`)

**引数解析:**
```bash
# ファイルディレクトリから自動検索
cargo run --bin coding-guide-helper -- example.c

# プロジェクトルートを明示
cargo run --bin coding-guide-helper -- --project-root /path/to/project src/main.c
```

**動作:**
- `--project-root` 指定時: そのディレクトリから検索開始
- 未指定時: ファイルのディレクトリから親へ遡って検索
- 設定読み込み後、診断実行時に適用

### 4. LSP 統合 (`crates/lsp/src/main.rs`)

**Initialize 時の動作:**
- `InitializeParams` から `workspace_folders` を取得
- ワークスペースルートで `ProjectConfig::find_and_load()` を実行
- 設定を `Arc<RwLock<ProjectConfig>>` で管理（非同期アクセス対応）

**診断実行時:**
- `config.read().await` で設定取得
- `to_diagnostic_config()` で診断設定に変換
- `diagnose()` に渡して実行

### 5. テスト

**`crates/core/src/config.rs` の tests モジュール (3テスト):**
1. `test_default_config`: デフォルト値の確認
2. `test_parse_toml`: 完全な TOML のパース
3. `test_partial_config`: 部分指定時のデフォルト値適用

**実行結果:**
```
running 244 tests
test config::tests::test_default_config ... ok
test config::tests::test_partial_config ... ok
test config::tests::test_parse_toml ... ok
...
test result: ok. 244 passed; 0 failed
```

### 6. ドキュメント

**README.md の追加セクション:**
- 「プロジェクト設定ファイル」セクション追加
  - 設定ファイルの例
  - 全設定項目の説明
  - CLI での使用例
  - LSP での自動読み込み説明

## 動作確認

### テスト 1: プロジェクトルートに設定ファイル
```bash
# coding-guide.toml に check_file_header = false を設定
cargo run --bin coding-guide-helper -- examples/valid_code.c
```
**結果:**
```
Loaded config from: coding-guide.toml
Check file header: false
Check function format: true
```

### テスト 2: サブディレクトリから自動検索
```bash
cargo run --bin coding-guide-helper -- examples/subdir/test.c
```
**結果:**
```
Loaded config from: coding-guide.toml  # ← 親ディレクトリで発見
Check file header: false
```

### テスト 3: 明示的なプロジェクトルート指定
```bash
cargo run --bin coding-guide-helper -- --project-root . examples/valid_code.c
```
**結果:**
```
Loaded config from: .\coding-guide.toml
Check file header: false
```

## 依存関係

**Cargo.toml に追加:**
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
```

## 技術的特徴

1. **標準的なパターン**: rustfmt.toml, .eslintrc と同様の設計
2. **Git 管理可能**: チーム全体で設定を共有
3. **デフォルト値**: 設定ファイルなしでも動作
4. **エラーハンドリング**: パースエラー時はデフォルトにフォールバック
5. **非同期対応**: LSP での RwLock 使用
6. **ディレクトリ検索**: プロジェクトルートまで自動検索

## 今後の拡張可能性

- `[file_header].template`: カスタムヘッダーテンプレート対応
- 診断レベルの細かい制御 (error/warning/info)
- インクルードパスの設定
- コーディングスタイルの詳細設定

## 完了したタスク

- ✅ ProjectConfig 構造体定義
- ✅ 設定ファイル検索・読み込み機能
- ✅ DiagnosticConfig への変換
- ✅ CLI 引数処理更新
- ✅ LSP 初期化時の設定読み込み
- ✅ 設定ファイルテスト (3件)
- ✅ README ドキュメント更新

**総テスト数: 244 (すべて合格)**
