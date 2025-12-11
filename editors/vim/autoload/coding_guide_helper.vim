" Coding Guide Helper - autoload functions

" LSPサーバーのパスを取得
function! coding_guide_helper#get_lsp_path()
  if g:coding_guide_helper_lsp_path != ''
    return g:coding_guide_helper_lsp_path
  endif
  
  " プロジェクトルートから自動検出
  let l:root = finddir('.git', '.;')
  if l:root != ''
    let l:root = fnamemodify(l:root, ':h')
    
    if has('win32') || has('win64')
      let l:release = l:root . '/target/release/coding-guide-helper-lsp.exe'
      let l:debug = l:root . '/target/debug/coding-guide-helper-lsp.exe'
    else
      let l:release = l:root . '/target/release/coding-guide-helper-lsp'
      let l:debug = l:root . '/target/debug/coding-guide-helper-lsp'
    endif
    
    if filereadable(l:release)
      return l:release
    elseif filereadable(l:debug)
      return l:debug
    endif
  endif
  
  return exepath('coding-guide-helper-lsp')
endfunction

" フォーマット実行
function! coding_guide_helper#format()
  if exists('*lsp#request')
    call lsp#request('coding-guide-helper', 'textDocument/formatting', {})
  else
    echohl WarningMsg
    echomsg 'LSP client not available'
    echohl None
  endif
endfunction

" 診断情報を表示
function! coding_guide_helper#show_diagnostics()
  if exists('*lsp#ui#vim#diagnostics#document_diagnostics')
    call lsp#ui#vim#diagnostics#document_diagnostics()
  else
    echohl WarningMsg
    echomsg 'LSP client not available'
    echohl None
  endif
endfunction
