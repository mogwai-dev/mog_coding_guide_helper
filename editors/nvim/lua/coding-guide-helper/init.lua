-- Coding Guide Helper LSP plugin for Neovim
local M = {}

-- デフォルト設定
M.config = {
  cmd = nil,  -- LSPサーバーのパス（自動検出）
  filetypes = { "c" },
  root_dir = nil,  -- プロジェクトルート（自動検出）
  settings = {},
}

-- LSPサーバーのパスを自動検出
local function find_server_path()
  -- Cargoワークスペースのルートを探す
  local root = vim.fn.finddir('.git', '.;')
  if root ~= '' then
    root = vim.fn.fnamemodify(root, ':h')
    
    -- Windowsの場合
    if vim.fn.has('win32') == 1 then
      local debug_path = root .. '/target/debug/coding-guide-helper-lsp.exe'
      local release_path = root .. '/target/release/coding-guide-helper-lsp.exe'
      
      if vim.fn.filereadable(release_path) == 1 then
        return release_path
      elseif vim.fn.filereadable(debug_path) == 1 then
        return debug_path
      end
    else
      -- Linux/macOSの場合
      local debug_path = root .. '/target/debug/coding-guide-helper-lsp'
      local release_path = root .. '/target/release/coding-guide-helper-lsp'
      
      if vim.fn.filereadable(release_path) == 1 then
        return release_path
      elseif vim.fn.filereadable(debug_path) == 1 then
        return debug_path
      end
    end
  end
  
  -- グローバルインストールをチェック
  return vim.fn.exepath('coding-guide-helper-lsp')
end

-- プロジェクトルートを検出
local function find_root_dir(fname)
  local util = require('lspconfig.util')
  return util.root_pattern('Cargo.toml', '.git')(fname)
end

-- セットアップ関数
function M.setup(opts)
  opts = opts or {}
  M.config = vim.tbl_deep_extend('force', M.config, opts)
  
  -- LSPサーバーのパスが指定されていない場合は自動検出
  if not M.config.cmd then
    local server_path = find_server_path()
    if server_path == '' then
      vim.notify(
        'coding-guide-helper-lsp not found. Please build it first: cargo build --package coding-guide-helper-lsp',
        vim.log.levels.ERROR
      )
      return
    end
    M.config.cmd = { server_path }
  end
  
  -- LSP設定を登録
  local configs = require('lspconfig.configs')
  if not configs.coding_guide_helper then
    configs.coding_guide_helper = {
      default_config = {
        cmd = M.config.cmd,
        filetypes = M.config.filetypes,
        root_dir = M.config.root_dir or find_root_dir,
        settings = M.config.settings,
      },
    }
  end
  
  -- LSPを起動
  require('lspconfig').coding_guide_helper.setup({
    capabilities = vim.lsp.protocol.make_client_capabilities(),
    on_attach = function(client, bufnr)
      -- キーマッピング
      local opts = { noremap = true, silent = true, buffer = bufnr }
      
      -- フォーマット
      if client.server_capabilities.documentFormattingProvider then
        vim.keymap.set('n', '<leader>f', function()
          vim.lsp.buf.format({ async = true })
        end, opts)
      end
      
      -- 診断を表示
      vim.keymap.set('n', '<leader>e', vim.diagnostic.open_float, opts)
      vim.keymap.set('n', '[d', vim.diagnostic.goto_prev, opts)
      vim.keymap.set('n', ']d', vim.diagnostic.goto_next, opts)
      
      -- カスタムon_attachがあれば実行
      if M.config.on_attach then
        M.config.on_attach(client, bufnr)
      end
    end,
  })
  
  vim.notify('Coding Guide Helper LSP started', vim.log.levels.INFO)
end

return M
