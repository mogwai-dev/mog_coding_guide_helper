" Coding Guide Helper - Vim plugin
" VimのLanguage Server Protocol対応

if exists('g:loaded_coding_guide_helper')
  finish
endif
let g:loaded_coding_guide_helper = 1

" デフォルト設定
if !exists('g:coding_guide_helper_lsp_path')
  let g:coding_guide_helper_lsp_path = ''
endif

if !exists('g:coding_guide_helper_auto_format')
  let g:coding_guide_helper_auto_format = 0
endif

" LSPサーバーのパスを自動検出
function! s:FindLSPServer()
  " プロジェクトルートを探す
  let l:root = finddir('.git', '.;')
  if l:root != ''
    let l:root = fnamemodify(l:root, ':h')
    
    " Windowsの場合
    if has('win32') || has('win64')
      let l:release_path = l:root . '/target/release/coding-guide-helper-lsp.exe'
      let l:debug_path = l:root . '/target/debug/coding-guide-helper-lsp.exe'
      
      if filereadable(l:release_path)
        return l:release_path
      elseif filereadable(l:debug_path)
        return l:debug_path
      endif
    else
      " Linux/macOSの場合
      let l:release_path = l:root . '/target/release/coding-guide-helper-lsp'
      let l:debug_path = l:root . '/target/debug/coding-guide-helper-lsp'
      
      if filereadable(l:release_path)
        return l:release_path
      elseif filereadable(l:debug_path)
        return l:debug_path
      endif
    endif
  endif
  
  " グローバルインストールをチェック
  return exepath('coding-guide-helper-lsp')
endfunction

" vim-lspの設定（vim-lspがインストールされている場合）
function! s:SetupVimLsp()
  if !exists('*lsp#register_server')
    echohl WarningMsg
    echomsg 'vim-lsp is not installed. Please install prabirshrestha/vim-lsp'
    echohl None
    return
  endif

  let l:server_path = g:coding_guide_helper_lsp_path
  if l:server_path == ''
    let l:server_path = s:FindLSPServer()
  endif

  if l:server_path == ''
    echohl ErrorMsg
    echomsg 'coding-guide-helper-lsp not found. Please build it first: cargo build --package coding-guide-helper-lsp'
    echohl None
    return
  endif

  call lsp#register_server({
    \ 'name': 'coding-guide-helper',
    \ 'cmd': {server_info->[l:server_path]},
    \ 'allowlist': ['c'],
    \ 'workspace_config': {},
    \ })

  augroup coding_guide_helper_lsp
    autocmd!
    autocmd FileType c call s:SetupLspMappings()
    
    " 保存時に自動フォーマット
    if g:coding_guide_helper_auto_format
      autocmd BufWritePre *.c call execute('LspDocumentFormat')
    endif
  augroup END

  echomsg 'Coding Guide Helper LSP started'
endfunction

" キーマッピングの設定
function! s:SetupLspMappings()
  if !exists('*lsp#register_server')
    return
  endif

  " フォーマット
  nnoremap <buffer> <silent> <Leader>f :LspDocumentFormat<CR>
  
  " 診断
  nnoremap <buffer> <silent> <Leader>e :LspDocumentDiagnostics<CR>
  nnoremap <buffer> <silent> [d :LspPreviousDiagnostic<CR>
  nnoremap <buffer> <silent> ]d :LspNextDiagnostic<CR>
  
  " ホバー情報
  nnoremap <buffer> <silent> K :LspHover<CR>
endfunction

" ALEの設定（ALEがインストールされている場合）
function! s:SetupALE()
  if !exists('g:ale_linters')
    return
  endif

  let l:server_path = g:coding_guide_helper_lsp_path
  if l:server_path == ''
    let l:server_path = s:FindLSPServer()
  endif

  if l:server_path == ''
    echohl ErrorMsg
    echomsg 'coding-guide-helper-lsp not found'
    echohl None
    return
  endif

  " ALEのLSP設定
  if !exists('g:ale_linters')
    let g:ale_linters = {}
  endif
  let g:ale_linters['c'] = get(g:ale_linters, 'c', []) + ['coding-guide-helper']

  if !exists('g:ale_linter_aliases')
    let g:ale_linter_aliases = {}
  endif

  call ale#linter#Define('c', {
    \ 'name': 'coding-guide-helper',
    \ 'lsp': 'stdio',
    \ 'executable': l:server_path,
    \ 'command': '%e',
    \ 'project_root': function('ale#c#FindProjectRoot'),
    \ })

  echomsg 'Coding Guide Helper ALE linter registered'
endfunction

" プラグインの初期化
function! coding_guide_helper#setup()
  " vim-lspが利用可能か確認
  if exists('*lsp#register_server')
    call s:SetupVimLsp()
  " ALEが利用可能か確認
  elseif exists('g:ale_enabled')
    call s:SetupALE()
  else
    echohl WarningMsg
    echomsg 'No LSP client found. Please install vim-lsp or ALE'
    echohl None
  endif
endfunction

" コマンド定義
command! CodingGuideHelperSetup call coding_guide_helper#setup()
command! CodingGuideHelperFormat LspDocumentFormat

" 自動起動
augroup coding_guide_helper_init
  autocmd!
  autocmd FileType c call coding_guide_helper#setup()
augroup END
