" Coding Guide Helper - Vim設定例

" =============================================================================
" vim-lspを使用する場合の設定例
" =============================================================================

" LSPサーバーのパス（自動検出されない場合のみ指定）
" let g:coding_guide_helper_lsp_path = expand('~/coding_guide_helper/target/release/coding-guide-helper-lsp')

" 保存時に自動フォーマット
let g:coding_guide_helper_auto_format = 1

" vim-lspの設定
let g:lsp_diagnostics_enabled = 1
let g:lsp_signs_enabled = 1
let g:lsp_diagnostics_echo_cursor = 1
let g:lsp_highlights_enabled = 1

" 診断記号のカスタマイズ
let g:lsp_signs_error = {'text': '✗'}
let g:lsp_signs_warning = {'text': '⚠'}
let g:lsp_signs_information = {'text': 'ℹ'}
let g:lsp_signs_hint = {'text': '💡'}

" =============================================================================
" カスタムキーマッピング
" =============================================================================

augroup coding_guide_helper_custom
  autocmd!
  
  " Cファイルを開いたときのキーマッピング
  autocmd FileType c nnoremap <buffer> <silent> <F3> :LspDocumentFormat<CR>
  autocmd FileType c nnoremap <buffer> <silent> <F4> :LspDocumentDiagnostics<CR>
  autocmd FileType c nnoremap <buffer> <silent> gd :LspDefinition<CR>
  autocmd FileType c nnoremap <buffer> <silent> gr :LspReferences<CR>
  autocmd FileType c nnoremap <buffer> <silent> K :LspHover<CR>
  
  " 保存時に自動フォーマット（個別設定）
  " autocmd BufWritePre *.c LspDocumentFormatSync
augroup END

" =============================================================================
" ALEを使用する場合の設定例（vim-lspの代わり）
" =============================================================================

" let g:ale_linters = {
" \   'c': ['coding-guide-helper', 'gcc'],
" \}
" 
" let g:ale_fixers = {
" \   'c': ['coding-guide-helper'],
" \}
" 
" " 保存時に自動修正
" let g:ale_fix_on_save = 1
" 
" " 診断の表示設定
" let g:ale_sign_error = '✗'
" let g:ale_sign_warning = '⚠'
" let g:ale_echo_msg_format = '[%linter%] %s [%severity%]'

" =============================================================================
" ステータスラインにLSP情報を表示
" =============================================================================

function! LspStatus() abort
  if exists('*lsp#get_server_status')
    let l:counts = lsp#get_buffer_diagnostics_counts()
    let l:errors = get(l:counts, 'error', 0)
    let l:warnings = get(l:counts, 'warning', 0)
    
    if l:errors > 0 || l:warnings > 0
      return printf(' E:%d W:%d', l:errors, l:warnings)
    endif
  endif
  return ''
endfunction

set statusline+=%{LspStatus()}
