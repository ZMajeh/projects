" =============================================================================
" PROFESSIONAL IDE CONFIGURATION
" =============================================================================

" 1. UI & Theming
syntax on
colorscheme desert           " Built-in dark theme (alternatives: elflord, ron)
set background=dark
set cursorline               " Highlight the current line
set number                   " Show line numbers
set signcolumn=yes           " Always show the left gutter (prevents text shifting)
set termguicolors            " Enable 24-bit RGB color support for modern terminals

" 2. Indentation & Formatting
set tabstop=4                " Number of spaces a tab counts for
set shiftwidth=4             " Number of spaces for auto-indent
set expandtab                " Convert tabs to spaces
set autoindent               " Copy indent from current line when starting a new line
set smartindent              " Smart auto-indenting for C-like languages

" 3. Search & Navigation
set hlsearch                 " Highlight all search results
set incsearch                " Highlight search matches as you type
set showmatch                " Highlight matching parentheses and brackets
set ignorecase               " Ignore case in search patterns
set smartcase                " Override ignorecase if search contains uppercase

" 4. Enhanced Status Line
set laststatus=2             " Always display the status line at the bottom
" Format: FilePath/Name [Format] [Type] [Line,Col] [Percentage]
"set statusline=%F%m%r%h%w\ [FORMAT=%{&ff}]\ [TYPE=%Y]\ [POS=%l,%v][%p%%]
set statusline=%F%m%r%h%w\ [FORMAT=%{&ff}]\ [TYPE=%Y]%=%{get(b:,'git_blame_status','')}\ [POS=%l,%v][%p%%]

" 5. IDE File Explorer (Using built-in Netrw)
let g:netrw_banner = 0       " Hide the unneeded help banner
let g:netrw_liststyle = 3    " Use tree-style view
let g:netrw_browse_split = 4 " Open files in the previous window
let g:netrw_winsize = 20     " Set explorer width to 20% of the screen
let g:netrw_altv = 1         " Open splits to the right

" Keybinding: Press Ctrl+N to toggle the file tree on the left (like VS Code)
nnoremap <C-n> :Lexplore<CR>

" Keybinding: Clear highlighted search results with Ctrl+L
nnoremap <C-L> :nohlsearch<CR><C-L>

" =============================================================================
" AUTO-COMPLETION & HIGHLIGHTING
" =============================================================================

" 6. Command-Line Auto-Completion
set wildmenu                 " Enable the enhanced command-line completion menu at the bottom
set wildmode=longest:full,full " Auto-complete to the longest common match, then list full options
set wildignore=*.o,*~,*.pyc  " Ignore compiled files in command auto-complete

" 7. Text Auto-Completion (Native)
filetype plugin indent on    " Enable language-specific plugins for Omni-completion
set completeopt=menuone,noinsert,noselect " IDE-like behavior: show menu, don't auto-insert first match
" In Insert mode, press Ctrl+N (Next match) or Ctrl+P (Previous match) to trigger the popup menu.

" 8. Highlight Current Word Automatically
set updatetime=300           " Milliseconds of idle time before highlighting triggers
augroup HighlightWord
    autocmd!
    " Automatically match and highlight the word under the cursor using the 'Search' color group
    autocmd CursorHold * let w = expand('<cword>') | exe (w != '' ? 'match Search /\V\<' . escape(w, '\') . '\>/' : 'match none')
augroup END


" =============================================================================
" VISIBLE WHITESPACE
" =============================================================================

" 9. Show Trailing Spaces
set list                     " Enable the rendering of invisible characters
set listchars=trail:·        " Render trailing spaces as a middle dot (·)

" Optional: Uncomment the line below if you also want to make tabs visible
set listchars=trail:·,tab:▸\

command! Blame vnew | setlocal buftype=nofile bufhidden=wipe noswapfile nowrap | execute "read !git blame " . shellescape(expand('#')) | 1delete _


" Set the wait time for CursorHold to 500 milliseconds
set updatetime=500

" Define your vanilla statusline
"set statusline=%f\ %m\ %r%=%{get(b:,'git_blame_status','')}

augroup VanillaGitBlame
  autocmd!
  autocmd CursorHold * call FetchLineBlameAsync()
augroup END

function! FetchLineBlameAsync()
  if &buftype != '' || !filereadable(expand('%'))
    let b:git_blame_status = ''
    return
  endif

  let l:line = line('.')
  let l:file = expand('%:t')
  let l:dir = expand('%:p:h')

  " Run git blame asynchronously so it doesn't freeze Vim
  if exists('*job_start')
    let l:cmd = ['git', 'blame', '-L', l:line . ',' . l:line, '--', l:file]
    call job_start(l:cmd, {\
          \ 'cwd': l:dir,\
          \ 'out_cb': 'BlameOutHandler',\
          \ 'err_cb': 'BlameErrHandler'\
          \ })
  endif
endfunction

" This function handles the output when Git finishes
function! BlameOutHandler(channel, msg)
  if a:msg =~ '00000000' || a:msg == ''
    let b:git_blame_status = '[Uncommitted]'
  else
    let l:parsed = matchstr(a:msg, '(\zs.\{-}\ze\s\+\d\+)')
    if l:parsed != ''
      let b:git_blame_status = '[' . l:parsed . ']'
    else
      let b:git_blame_status = '[Parse Error]'
    endif
  endif
  redrawstatus
endfunction

" Ignore errors silently
function! BlameErrHandler(channel, msg)
endfunction