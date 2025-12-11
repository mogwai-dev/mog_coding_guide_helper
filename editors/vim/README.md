# Coding Guide Helper for Vim

Vim用のCoding Guide Helper LSPプラグイン。

## 必要要件

- Vim 8.0以上
- LSPクライアント（以下のいずれか）:
  - [vim-lsp](https://github.com/prabirshrestha/vim-lsp) （推奨）
  - [ALE](https://github.com/dense-analysis/ale)
- coding-guide-helper-lsp（ビルド済みのLSPサーバー）

## インストール

### vim-plugを使用

```vim
" LSPクライアント（vim-lspを推奨）
Plug 'prabirshrestha/vim-lsp'
Plug 'prabirshrestha/async.vim'

" Coding Guide Helper
Plug 'mogwai-dev/mog_coding_guide_helper', { 'rtp': 'editors/vim' }
```

### Vundleを使用

```vim
Plugin 'prabirshrestha/vim-lsp'
Plugin 'prabirshrestha/async.vim'
Plugin 'mogwai-dev/mog_coding_guide_helper', { 'rtp': 'editors/vim' }
```

### 手動インストール

1. このディレクトリを `~/.vim/` にコピー:

```bash
# Linux/macOS
mkdir -p ~/.vim/plugin ~/.vim/autoload
cp editors/vim/plugin/coding-guide-helper.vim ~/.vim/plugin/
cp editors/vim/autoload/coding_guide_helper.vim ~/.vim/autoload/

# Windows
mkdir %USERPROFILE%\vimfiles\plugin %USERPROFILE%\vimfiles\autoload
copy editors\vim\plugin\coding-guide-helper.vim %USERPROFILE%\vimfiles\plugin\
copy editors\vim\autoload\coding_guide_helper.vim %USERPROFILE%\vimfiles\autoload\
```

2. vim-lspもインストールしてください

## 設定

### 基本設定（.vimrcまたはinit.vim）

```vim
" LSPサーバーのパス（省略時は自動検出）
" let g:coding_guide_helper_lsp_path = '/path/to/coding-guide-helper-lsp'

" 保存時に自動フォーマット（0: 無効, 1: 有効）
let g:coding_guide_helper_auto_format = 0

" vim-lspの設定
let g:lsp_diagnostics_enabled = 1
let g:lsp_diagnostics_echo_cursor = 1
```

### vim-lspを使用する場合

```vim
" vim-lspのインストール
Plug 'prabirshrestha/vim-lsp'
Plug 'prabirshrestha/async.vim'
Plug 'mogwai-dev/mog_coding_guide_helper', { 'rtp': 'editors/vim' }

" .vimrc
let g:lsp_diagnostics_enabled = 1
let g:lsp_signs_enabled = 1
let g:lsp_diagnostics_echo_cursor = 1

" オプション: 診断の表示設定
let g:lsp_signs_error = {'text': '✗'}
let g:lsp_signs_warning = {'text': '⚠'}
let g:lsp_signs_hint = {'text': '💡'}
```

### ALEを使用する場合

```vim
" ALEのインストール
Plug 'dense-analysis/ale'
Plug 'mogwai-dev/mog_coding_guide_helper', { 'rtp': 'editors/vim' }

" .vimrc
let g:ale_linters = {
\   'c': ['coding-guide-helper'],
\}

let g:ale_fixers = {
\   'c': ['coding-guide-helper'],
\}

" 保存時に自動修正
let g:ale_fix_on_save = 1
```

## キーマッピング（デフォルト）

Cファイルを開くと、以下のキーマッピングが有効になります:

- `<Leader>f` - コードフォーマット
- `<Leader>e` - 診断メッセージ一覧を表示
- `[d` - 前の診断へ移動
- `]d` - 次の診断へ移動
- `K` - ホバー情報を表示

### カスタムキーマッピング

```vim
" .vimrc
augroup coding_guide_helper_keys
  autocmd!
  autocmd FileType c nnoremap <buffer> <F3> :LspDocumentFormat<CR>
  autocmd FileType c nnoremap <buffer> <F4> :LspDocumentDiagnostics<CR>
augroup END
```

## コマンド

- `:CodingGuideHelperSetup` - LSPサーバーを手動でセットアップ
- `:CodingGuideHelperFormat` - コードフォーマットを実行

## LSPサーバーのビルド

プラグインを使用する前に、LSPサーバーをビルドしてください:

```bash
cd /path/to/coding_guide_helper
cargo build --release --package coding-guide-helper-lsp
```

ビルド後、以下の場所にバイナリが作成されます:
- `target/release/coding-guide-helper-lsp`（Linux/macOS）
- `target\release\coding-guide-helper-lsp.exe`（Windows）

## トラブルシューティング

### LSPサーバーが見つからない

エラーメッセージが表示された場合は、パスを明示的に指定してください:

```vim
let g:coding_guide_helper_lsp_path = expand('~/path/to/coding-guide-helper-lsp')
```

### vim-lspのログ確認

```vim
:LspStatus
:LspLog
```

### デバッグモードの有効化

```vim
let g:lsp_log_verbose = 1
let g:lsp_log_file = expand('~/vim-lsp.log')
```

## 機能

- ✅ リアルタイム診断（CGH001, CGH002）
- ✅ コードフォーマット
- ✅ vim-lsp対応
- ✅ ALE対応
- 🚧 コードアクション（今後実装予定）
- 🚧 ホバー情報（今後実装予定）

## 参考リンク

- [vim-lsp](https://github.com/prabirshrestha/vim-lsp)
- [ALE](https://github.com/dense-analysis/ale)

## ライセンス

MIT
