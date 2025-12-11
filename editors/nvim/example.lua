-- Coding Guide Helper LSPの簡単なセットアップ例

-- 基本的な使い方
require('coding-guide-helper').setup()

-- カスタム設定の例
require('coding-guide-helper').setup({
  -- LSPサーバーのパスを明示的に指定
  cmd = { vim.fn.expand('~/coding_guide_helper/target/release/coding-guide-helper-lsp') },
  
  -- カスタムon_attach関数
  on_attach = function(client, bufnr)
    print('Coding Guide Helper LSP attached to buffer ' .. bufnr)
    
    -- 追加のキーマッピング
    local opts = { noremap = true, silent = true, buffer = bufnr }
    vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)
    vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)
  end,
})

-- 保存時に自動フォーマット
vim.api.nvim_create_autocmd('BufWritePre', {
  pattern = '*.c',
  callback = function()
    vim.lsp.buf.format({ async = false })
  end,
})
