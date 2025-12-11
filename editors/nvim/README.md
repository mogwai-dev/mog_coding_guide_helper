# Coding Guide Helper for Neovim

Neovim用のCoding Guide Helper LSPプラグイン。

## 必要要件

- Neovim >= 0.8.0
- [nvim-lspconfig](https://github.com/neovim/nvim-lspconfig)
- coding-guide-helper-lsp（ビルド済みのLSPサーバー）

## インストール

### lazy.nvim

```lua
{
  'mogwai-dev/mog_coding_guide_helper',
  ft = { 'c' },
  config = function()
    require('coding-guide-helper').setup({
      -- オプション設定（任意）
      -- cmd = { '/path/to/coding-guide-helper-lsp' },  -- LSPサーバーのパス（自動検出）
    })
  end,
  dependencies = {
    'neovim/nvim-lspconfig',
  },
}
```

### packer.nvim

```lua
use {
  'mogwai-dev/mog_coding_guide_helper',
  ft = { 'c' },
  requires = { 'neovim/nvim-lspconfig' },
  config = function()
    require('coding-guide-helper').setup()
  end
}
```

### vim-plug

```vim
Plug 'neovim/nvim-lspconfig'
Plug 'mogwai-dev/mog_coding_guide_helper', { 'for': 'c' }

lua << EOF
require('coding-guide-helper').setup()
EOF
```

### 手動インストール

このディレクトリを `~/.config/nvim/lua/coding-guide-helper/` にコピー:

```bash
# Linux/macOS
mkdir -p ~/.config/nvim/lua
cp -r editors/nvim/lua/coding-guide-helper ~/.config/nvim/lua/

# Windows
mkdir %LOCALAPPDATA%\nvim\lua
xcopy editors\nvim\lua\coding-guide-helper %LOCALAPPDATA%\nvim\lua\coding-guide-helper\ /E /I
```

`init.lua` または `init.vim` に追加:

```lua
-- init.lua
require('coding-guide-helper').setup()
```

```vim
" init.vim
lua require('coding-guide-helper').setup()
```

## 設定

```lua
require('coding-guide-helper').setup({
  -- LSPサーバーのパス（省略時は自動検出）
  cmd = { '/path/to/coding-guide-helper-lsp' },
  
  -- 対象ファイルタイプ
  filetypes = { 'c' },
  
  -- カスタムon_attach
  on_attach = function(client, bufnr)
    -- 独自のキーマッピングなど
  end,
})
```

## キーマッピング（デフォルト）

- `<leader>f` - コードフォーマット
- `<leader>e` - 診断メッセージを表示
- `[d` - 前の診断へ移動
- `]d` - 次の診断へ移動

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

エラーメッセージが表示された場合は、LSPサーバーのパスを明示的に指定してください:

```lua
require('coding-guide-helper').setup({
  cmd = { vim.fn.expand('~/path/to/coding-guide-helper-lsp') },
})
```

### ログの確認

LSPのログを確認するには:

```vim
:lua vim.cmd('e ' .. vim.lsp.get_log_path())
```

### LSPの状態確認

```vim
:LspInfo
```

## 機能

- ✅ リアルタイム診断（CGH001, CGH002）
- ✅ コードフォーマット
- 🚧 コードアクション（今後実装予定）
- 🚧 ホバー情報（今後実装予定）

## ライセンス

MIT
