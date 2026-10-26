# coolTCsh — Professional tcsh Configuration

A modern, feature-rich `.cshrc` configuration that transforms tcsh into a powerful, user-friendly shell experience with colors, smart completions, modern tool integrations, and productivity-boosting aliases.

---

## ✨ Features Overview

| Category | Enhancements |
|----------|--------------|
| **Visual** | Colored prompt with git branch, colored `ls`/`grep`/`diff`, Unicode box-drawing |
| **Navigation** | Smart `cd` (zoxide), directory stack, `..`/`...` aliases, fzf fuzzy finding |
| **History** | 10,000 entries, merge across sessions, deduplication, atuin sync |
| **Completion** | Case-insensitive, menu-driven, enhanced, no beep |
| **Modern Tools** | Auto-detects `eza`, `bat`, `fd`, `rg`, `btop`, `dust`, `duf`, `procs` |
| **Git** | 20+ aliases, branch in prompt, diff/status shortcuts |
| **Development** | Docker, Kubernetes, Terraform, Go, Rust, Python, Node env support |
| **Safety** | Interactive `rm`/`cp`/`mv`, `--preserve-root` |
| **Productivity** | Archive extractor, weather, cheat sheets, quick backup, tmux auto-start |

---

## 🚀 Quick Install

```tcsh
# Backup your existing config (if any)
mv ~/.cshrc ~/.cshrc.backup.$(date +%Y%m%d)

# Clone and link
git clone https://github.com/yourusername/coolTCsh.git ~/coolTCsh
ln -sf ~/coolTCsh/.cshrc ~/.cshrc

# Reload
source ~/.cshrc
```

Or copy directly:
```tcsh
cp /path/to/coolTCsh/.cshrc ~/.cshrc
source ~/.cshrc
```

---

## 📸 Visual Preview

### Prompt
```
┌─user@hostname:~/projects/coolTCsh (main)
└─>                                    [14:32:45]
```
- **Green**: username
- **Cyan**: hostname
- **Yellow**: current directory
- **Magenta**: git branch (when in repo)
- **Gray**: time in right prompt
- **Unicode box-drawing** for clean visual separation

### Modern `ls` (with eza)
```
❯ ll
⬢ .git          📁 src          📁 tests          📄 README.md          📄 .cshrc
```
- Icons for file types
---

## 🎯 How This Improves Your Experience

### 1. **Instant Visual Context**
The prompt shows *exactly* what you need at a glance:
- Who you are (`user`)
- Where you are (`hostname`, `directory`)
- **Git branch** — no more `git branch` to check
- Current time — useful for logs/timing

### 2. **Modern CLI Tools (Zero Config)**
The config **auto-detects** and aliases modern replacements:
| Legacy | Modern | Benefit |
|--------|--------|---------|
| `ls` | `eza` | Icons, colors, git status, tree view |
| `cat` | `bat` | Syntax highlighting, line numbers, paging |
| `find` | `fd` | Faster, respects `.gitignore`, colored |
| `grep` | `rg` | 5-10x faster, smart case, colors |
| `top` | `btop`/`htop` | Graphs, mouse support, better UI |
| `du` | `dust` | Visual tree, human-readable |
| `df` | `duf` | Pretty tables, colors |
| `ps` | `procs` | Keywords, colors, human-readable |

**No install required** — if the tool isn't present, falls back gracefully to standard commands.

### 3. **Fuzzy Finding Everything (fzf)**
- `Ctrl+T` — fuzzy find files in current directory
- `Ctrl+R` — fuzzy search command history
- `Alt+C` — fuzzy `cd` into subdirectories
- Preview pane shows file content (with `bat`) or directory tree

### 4. **Smart Directory Jumping (zoxide)**
```tcsh
cd proj<TAB>      # → /home/user/projects/coolTCsh
cd doc<TAB>       # → /home/user/documents
```
Learns your habits — the more you use it, the smarter it gets.

### 5. **Unified History Across Sessions (atuin)**
- History syncs across terminals instantly
- Search with `Ctrl+R` shows **all** history from all sessions
- Encrypted, can sync across machines

### 6. **Git Workflow at Your Fingertips**
```tcsh
gst          # git status
gl           # pretty log graph (last 20)
gla          # all branches graph (last 30)
gco feature  # checkout branch
gcb new-feat # create & checkout
gp           # push
gpl          # pull
gd           # diff working tree
gdc          # diff staged
```
Branch name **always visible in prompt** — never commit to wrong branch again.

### 7. **Productivity Aliases**
```tcsh
..           # cd ..
...          # cd ../..
....         # cd ../../..
mkcd dir     # mkdir -p && cd
extract x.zip# auto-detects format, extracts
ff pattern   # find files by name
fd pattern   # find dirs by name
sg "todo"    # search text in files
bak file     # timestamped backup
wttr london  # weather in terminal
cheat tar    # cheat sheet for command
```

### 8. **Safety First**
```tcsh
rm file      # → "remove file? (y/n)"
cp a b       # → "overwrite b? (y/n)"
mv a b       # → "overwrite b? (y/n)"
chmod 777 /  # → blocked by --preserve-root
```

### 9. **Language Environments Ready**
Auto-configures PATH for:
- **Go** (`/usr/local/go/bin`, `~/go/bin`)
- **Rust** (`~/.cargo/bin`)
- **Python** (pyenv)
- **Node** (nvm)

Just install the tool — the config detects and sets it up.

### 10. **Development Stack Aliases**
```tcsh
# Docker
dkps         # pretty container list
dkl web      # follow logs
dke web bash # exec into container

# Kubernetes
kgp          # get pods
klog pod     # follow logs
kex pod bash # exec

# Terraform
tfi && tfp && tfa
```
- Directories grouped first
- Git status indicators
---

## ⌨️ Key Bindings

| Key | Action |
|-----|--------|
| `↑` / `↓` | History search (type prefix first) |
| `Ctrl+←` / `Ctrl+→` | Word jump |
| `Alt+D` | Delete word forward |
| `Alt+Backspace` | Delete word backward |
| `Home` / `End` | Line start/end |
| `Ctrl+U` | Clear entire line |
| `Tab` | Menu completion |
| `Ctrl+T` | fzf file finder |
| `Ctrl+R` | fzf/atuin history search |
| `Alt+C` | fzf directory jump |

---

## 🛠️ Optional Dependencies

Install for maximum experience (all optional — config works without them):

```bash
# Modern CLI tools
sudo apt install eza bat fd-find ripgrep btop dust duf procs  # Debian/Ubuntu
sudo dnf install eza bat fd-find ripgrep btop dust duf procs  # Fedora
sudo pacman -S eza bat fd ripgrep btop dust duf procs         # Arch

# Fuzzy finder
sudo apt install fzf

# Smart cd
sudo apt install zoxide  # or: curl -sSfL https://raw.githubusercontent.com/ajeetdsouza/zoxide/main/install.sh | sh

# Shell history sync
curl --proto '=https' --tlsv1.2 -LsSf https://setup.atuin.sh | sh

# Starship prompt (alternative to built-in)
curl -sS https://starship.rs/install.sh | sh
```

---

## 📁 Project Structure

```
coolTCsh/
├── .cshrc          # Main configuration (symlink to ~/.cshrc)
├── README.md       # This file
└── .cshrc.local    # Your private overrides (gitignored)
```

### Local Overrides
Create `~/.cshrc.local` for machine-specific settings (API keys, paths, etc.):
```tcsh
# ~/.cshrc.local - not tracked in git
setenv MY_API_KEY "secret"
alias work 'cd ~/work && tmux attach -t work || tmux new -s work'
```
Automatically sourced at the end of `.cshrc`.

---

## 🔧 Customization

### Disable tmux auto-start
Comment out the last block in `.cshrc`:
```tcsh
# if ( ! $?TMUX && $?prompt ) then
#     tmux has-session -t default >& /dev/null
#     ...
# endif
```

### Use Starship instead of built-in prompt
Uncomment in `.cshrc`:
```tcsh
if ( -x /usr/bin/starship ) then
    eval `starship init tcsh`
endif
```

### Add your own aliases
Edit `~/.cshrc.local` (created automatically if missing):
```tcsh
alias myproject 'cd ~/projects/myproject && source venv/bin/activate.csh'
```

---

## 🐚 Why tcsh?

| Feature | tcsh | bash | zsh | fish |
|---------|------|------|-----|------|
| C-like syntax | ✅ | ❌ | ❌ | ❌ |
| Built-in array/list ops | ✅ | ❌ | ✅ | ✅ |
| History expansion (`!!`, `!*`) | ✅ | ✅ | ✅ | ⚠️ |
| Spelling correction | ✅ | ❌ | ✅ | ✅ |
| Lightweight | ✅ | ✅ | ❌ | ❌ |
| Ubiquitous on BSD/macOS | ✅ | ✅ | ❌ | ❌ |

**tcsh shines** for:
- C/C++ developers (familiar syntax)
- BSD/macOS users (default on FreeBSD, available everywhere)
- Lightweight scripting with arrays
- Users who like `!!`, `!*`, `^old^new^` history tricks

---

## 📜 License

MIT License — use freely, modify, share.

---

## 🤝 Contributing

1. Fork the repo
2. Create your feature branch
3. Test thoroughly in tcsh
4. Submit a PR

---

## 🙏 Acknowledgments

- **tcsh** — the shell that keeps on giving
- **eza**, **bat**, **fd**, **rg**, **btop**, **dust**, **duf**, **procs** — modern CLI renaissance
- **fzf**, **zoxide**, **atuin** — fuzzy finding, smart navigation, history sync
- **starship** — cross-shell prompt inspiration

---

> **Pro tip**: Run `tcsh --version` to confirm you're on a recent version (6.24+ recommended for best Unicode support).

**Happy tcsh-ing!** 🐚✨
- Colors by file type