# Coding Guide Helper for Eclipse

Eclipse用のCoding Guide Helper LSP統合設定。

## 必要要件

- Eclipse IDE 2021-03以降（CDT含む）
- LSP4E（Language Servers for Eclipse）プラグイン
- coding-guide-helper-lsp（ビルド済みのLSPサーバー）

## インストール

### 1. LSP4Eプラグインのインストール

Eclipse Marketplaceから「LSP4E」をインストール：

1. `Help` → `Eclipse Marketplace...`
2. 検索ボックスに「LSP4E」と入力
3. 「Language Servers (LSP4E)」をインストール
4. Eclipseを再起動

または、Update Siteから直接インストール：
- `Help` → `Install New Software...`
- Add: `https://download.eclipse.org/lsp4e/releases/latest/`

### 2. LSPサーバーのビルド

```bash
cd /path/to/coding_guide_helper
cargo build --release --package coding-guide-helper-lsp
```

### 3. LSPサーバーの設定

#### 方法A: Preferences経由（推奨）

1. `Window` → `Preferences` → `Language Servers`
2. `Add` ボタンをクリック
3. 以下を入力：
   - **Name**: `Coding Guide Helper`
   - **Launch Mode**: `Standard I/O`
   - **Program**: LSPサーバーのフルパスを指定
     - Windows: `C:\path\to\coding_guide_helper\target\release\coding-guide-helper-lsp.exe`
     - Linux/macOS: `/path/to/coding_guide_helper/target/release/coding-guide-helper-lsp`
   - **Content Type**: `C Source File` を選択
4. `Apply and Close`

#### 方法B: .settings経由（プロジェクト設定）

プロジェクトの `.settings/org.eclipse.lsp4e.prefs` に以下を追加：

```properties
eclipse.preferences.version=1
languageServers=coding-guide-helper

# Windows
coding-guide-helper.command=C\:\\path\\to\\coding-guide-helper-lsp.exe

# Linux/macOS
# coding-guide-helper.command=/path/to/coding-guide-helper-lsp

coding-guide-helper.contentType=org.eclipse.cdt.core.cSource
coding-guide-helper.contentType.1=org.eclipse.cdt.core.cHeader
```

### 4. 環境変数の設定（オプション）

LSPサーバーをPATHに追加しておくと、パスの指定が不要になります：

**Windows:**
```powershell
$env:Path += ";C:\path\to\coding_guide_helper\target\release"
```

**Linux/macOS:**
```bash
export PATH="$PATH:/path/to/coding_guide_helper/target/release"
```

## 使い方

### プロジェクトのセットアップ

1. EclipseでCプロジェクトを作成または既存プロジェクトを開く
2. `.c` または `.h` ファイルを開く
3. LSPサーバーが自動的に起動します

### 機能

#### 診断（Diagnostics）

- リアルタイムでコーディング規約違反を検出
- エディタ上に波線と問題ビューに表示される
- `Problems` ビュー（`Window` → `Show View` → `Problems`）で確認

**検出されるエラー:**
- CGH001: ファイルヘッダーコメント不足
- CGH002: 関数の不正なフォーマット

#### コードフォーマット

Cファイルを開いた状態で：
1. 右クリック → `Source` → `Format`
2. または `Ctrl+Shift+F`（Windows/Linux）/ `Cmd+Shift+F`（macOS）

#### キーボードショートカット

- `Ctrl+Shift+F` - フォーマット
- `Ctrl+1` - クイックフィックス（今後実装予定）
- `F2` - 診断の詳細を表示

## トラブルシューティング

### LSPサーバーが起動しない

#### 1. LSP4Eのログを確認

`Window` → `Show View` → `Other...` → `General` → `Error Log`

#### 2. コマンドラインで手動テスト

```bash
# LSPサーバーが正常に動作するか確認
coding-guide-helper-lsp

# 出力がない場合は正常（LSPはstdioで通信するため）
# エラーが出る場合はビルドを確認
```

#### 3. パスの確認

Preferences → Language Serversで設定したパスが正しいか確認：
- Windowsの場合: バックスラッシュ `\\` でエスケープが必要
- 絶対パスを使用（相対パスは避ける）

### 診断が表示されない

1. `Window` → `Preferences` → `General` → `Editors` → `Text Editors` → `Annotations`
2. 以下が有効になっているか確認：
   - `Errors`
   - `Warnings`
   - `Info`

### フォーマットが機能しない

1. `Window` → `Preferences` → `C/C++` → `Code Style` → `Formatter`
2. 「Use default formatter」が選択されていないことを確認
3. または、右クリック → `Source` → `Format` を直接使用

## 設定例

### プロジェクト設定ファイル

`.settings/org.eclipse.lsp4e.prefs`:
```properties
eclipse.preferences.version=1
languageServers=coding-guide-helper

# Windows example
coding-guide-helper.command=C\:\\workspace\\coding_guide_helper\\target\\release\\coding-guide-helper-lsp.exe

# Linux/macOS example  
# coding-guide-helper.command=/home/user/coding_guide_helper/target/release/coding-guide-helper-lsp

coding-guide-helper.contentType=org.eclipse.cdt.core.cSource
coding-guide-helper.contentType.1=org.eclipse.cdt.core.cHeader
coding-guide-helper.initializationOptions={}
```

### ワークスペース全体で有効化

`.metadata/.plugins/org.eclipse.core.runtime/.settings/org.eclipse.lsp4e.prefs`:
```properties
# 上記と同じ設定をワークスペースレベルで設定
```

## デバッグモード

LSPサーバーのデバッグログを有効にする：

1. LSPサーバーを直接実行してログを確認：
```bash
RUST_LOG=debug coding-guide-helper-lsp > lsp.log 2>&1
```

2. Eclipseの環境変数設定：
   - `Window` → `Preferences` → `Language Servers`
   - サーバー設定を編集
   - Environment variables: `RUST_LOG=debug`

## 既知の問題

- Eclipse CDTの組み込みフォーマッタとの競合がある場合があります
  - 回避策: CDTフォーマッタを無効化
- 非常に大きなファイル（10000行以上）では応答が遅くなる場合があります

## 参考リンク

- [LSP4E Documentation](https://github.com/eclipse/lsp4e)
- [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
- [Eclipse CDT](https://www.eclipse.org/cdt/)

## ライセンス

MIT
