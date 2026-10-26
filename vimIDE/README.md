# vimIDE — A Professional IDE Configuration for Vim

This repository contains a curated `.vimrc` that transforms vanilla Vim into a lightweight, modern IDE — **without any external plugin managers or third-party plugins**. Everything runs on Vim's built-in capabilities (Netrw, Omni-completion, async job control, etc.), so it works out of the box on any system with Vim 8+ / Neovim.

---

## 🎯 What This Config Adds Over Vanilla Vim

| Area | Vanilla Vim (Default) | vimIDE (This Config) |
|------|----------------------|----------------------|
| **UI / Theming** | No syntax highlighting, no line numbers, no cursor line | `syntax on`, `desert` colorscheme, line numbers, cursor line, persistent sign column, 24-bit true color support |
| **Indentation** | Hard tabs, no auto-indent | 4-space soft tabs, `autoindent` + `smartindent` for C-like languages |
| **Search** | Case-sensitive, no live highlighting | `incsearch` (live), `hlsearch` (persistent), `smartcase` (smart case handling), bracket matching |
| **Status Line** | Minimal, hidden by default | Always-visible custom status line showing: file path, format, type, **git blame for current line**, position, percentage |
| **File Explorer** | `:Ex` opens a basic buffer list | **Tree-style Netrw** (`:Lexplore`), toggled with **`Ctrl+N`** — VS Code-like sidebar |
| **Keybindings** | None for common IDE actions | `Ctrl+N` → toggle file tree; `Ctrl+L` → clear search highlights + redraw |
| **Command Completion** | Basic tab completion | **Wildmenu** with longest-common-match → full list, ignores `*.o`, `*~`, `*.pyc` |
| **Text Completion** | Basic `<C-n>`/`<C-p>` keyword completion | **Omni-completion** enabled via `filetype plugin indent on`; IDE-style popup (`completeopt=menuone,noinsert,noselect`) |
| **Semantic Highlighting** | None | **Auto-highlight word under cursor** after 300 ms idle (`CursorHold`) |
| **Whitespace Visibility** | Invisible | Trailing spaces shown as `·`, tabs as `▸` (configurable via `listchars`) |
| **Git Integration** | None | **Async git blame** per line in status line — non-blocking, updates on `CursorHold` |
| **Git Blame Command** | Manual `:!git blame %` | `:Blame` opens a scratch buffer with full-file blame |

---

## 🚀 Quick Start

```bash
# Clone or copy the .vimrc to your home directory
cp .vimrc ~/.vimrc

# Or symlink it (recommended for this repo)
ln -sf $(pwd)/.vimrc ~/.vimrc
```

Open Vim — no plugin installation needed.

---

## 🧭 Keybindings Cheatsheet

| Key | Action |
|-----|--------|
| `Ctrl+N` | Toggle file explorer (Netrw tree view) |
| `Ctrl+L` | Clear search highlights + redraw screen |
| `<C-n>` / `<C-p>` (Insert mode) | Next / previous completion match |
| `:Blame` | Open git blame for current file in a scratch buffer |
| `:Lexplore` | Open file explorer manually |

---

## 🛠️ How It Works (Under the Hood)

### 1. **Netrw as a File Tree**
```vim
let g:netrw_banner = 0
let g:netrw_liststyle = 3      " tree view
let g:netrw_browse_split = 4   " open in previous window
let g:netrw_winsize = 20       " 20% width
let g:netrw_altv = 1           " split right
nnoremap <C-n> :Lexplore<CR>
```
No NERDTree, no nvim-tree — uses Vim's built-in Netrw.

### 2. **Async Git Blame in Status Line**
```vim
autocmd CursorHold * call FetchLineBlameAsync()
function! FetchLineBlameAsync()
  let l:cmd = ['git', 'blame', '-L', line('.').','.line('.'), '--', expand('%:t')]
  call job_start(l:cmd, { 'cwd': expand('%:p:h'), 'out_cb': 'BlameOutHandler' })
endfunction
```
Uses `job_start()` (Vim 8+) for non-blocking git blame. Result appears in status line as `[Author (Date)]`.

### 3. **Auto Word Highlighting**
```vim
set updatetime=300
autocmd CursorHold * let w = expand('<cword>') | exe (w != '' ? 'match Search /\V\<' . escape(w, '\') . '\>/' : 'match none')
```
Highlights all occurrences of the word under cursor after 300 ms idle.

### 4. **IDE-Style Completion**
```vim
filetype plugin indent on
set completeopt=menuone,noinsert,noselect
```
- `menuone` — show popup even with one match  
- `noinsert` — don't auto-insert first match  
- `noselect` — don't pre-select first item  

Works for any language with a Vim ftplugin (Python, Go, Rust, C, etc.).

---

## 📦 Requirements

- **Vim 8.0+** or **Neovim 0.5+** (for `job_start`, `termguicolors`, `signcolumn`)
- **Git** (for blame integration)
- A terminal with **true color support** (most modern terminals)

---

## 🎨 Customization

### Change Colorscheme
```vim
colorscheme elflord   " or ron, slate, evening, etc.
```

### Adjust Indentation
```vim
set tabstop=2
set shiftwidth=2
```

### Disable Whitespace Markers
```vim
set nolist
```

### Change File Explorer Width
```vim
let g:netrw_winsize = 30   " 30%
```

---

## 🧪 Testing the Config

```bash
# Syntax check (no errors expected)
vim -u .vimrc -c "qa!"

# Verify key features
vim -u .vimrc -c "echo 'syntax:' exists('g:syntax_on')" -c "echo 'netrw:' exists('g:loaded_netrwPlugin')" -c "qa!"
```

---

## 📄 License

MIT — Use freely, modify, share.

---

## 🤝 Contributing

PRs welcome for:
- Additional built-in Vim features (e.g., `signcolumn=auto`, `cursorcolumn`)
- More filetype-specific tweaks
- Documentation improvements

---

> **Philosophy**: *Vim is already an IDE. This config just turns on the features that ship with it.*