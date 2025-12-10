# Coding Guide Helper

C言語のコーディングガイドチェッカー＆フォーマッター

## 機能

- **診断**: コーディング規約違反を検出
  - CGH001: ファイルヘッダーコメントの確認
  - CGH002: 関数フォーマットのチェック

- **フォーマット**: コードの自動整形
  - ファイルヘッダーの自動追加
  - 空白の整理

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

## テスト

```bash
cargo test
```

## ライセンス

MIT
