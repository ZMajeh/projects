# ~/.cshrc - Professional tcsh configuration with colors and modern features
# Author: majeh
# Last updated: 2026-10-26


# =============================================================================
# BASIC SETTINGS
# =============================================================================

# Enable color support
setenv CLICOLOR 1
setenv LSCOLORS "ExGxBxDxCxEgEdxbxgxcxd"
setenv LS_COLORS "di=1;34:ln=1;36:so=1;35:pi=1;33:ex=1;32:bd=1;34;46:cd=1;34;43:su=1;30;41:sg=1;30;46:tw=1;30;42:ow=1;30;43"


# History settings
set history = 10000
set savehist = (10000 merge)
set histdup = erase

# Autocomplete settings
set autolist
set complete = enhance
set correct = cmd
set autoexpand
set autocorrect

# Editor and pager
setenv EDITOR vim
setenv VISUAL vim
setenv PAGER less
setenv LESS "-R -i -M -x4"

# Language/locale
setenv LANG en_US.UTF-8
setenv LC_ALL en_US.UTF-8

# =============================================================================
# PROMPT - Professional colored prompt with git info
# =============================================================================

# Define colors for prompt
set red="%{\033[1;31m%}"
set green="%{\033[1;32m%}"
set yellow="%{\033[1;33m%}"
set blue="%{\033[1;34m%}"
set magenta="%{\033[1;35m%}"
set cyan="%{\033[1;36m%}"
set white="%{\033[1;37m%}"
set gray="%{\033[0;90m%}"
set reset="%{\033[0m%}"
set bold="%{\033[1m%}"
set dim="%{\033[2m%}"


# Build the prompt - using Unicode box drawing characters
# Define prompt without git branch (git branch added via precmd)
set prompt = "${gray}┌─${reset}${green}%n${reset}${gray}@${reset}${cyan}%m${reset}${gray}:${reset}${yellow}%~${reset}\n${gray}└─${reset}${bold}\$${reset} "


# Right prompt (shows time)
set rprompt = "${gray}[%T]${reset}"


# Update prompt with git branch before each prompt
#alias precmd 'set gitbr = `git branch 2>/dev/null | grep "^\*" | sed "s/^\* //"`; if ("$gitbr" != "") set prompt = "${gray}┌─${reset}${green}%n${reset}${gray}@${reset}${cyan}%m${reset}${gray}:${reset}${yellow}%~${reset} ${gray}(${reset}${magenta}$gitbr${reset}${gray})${reset}\n${gray}└─${reset}${bold}>${reset} "; if ("$gitbr" == "") set prompt = "${gray}┌─${reset}${green}%n${reset}${gray}@${reset}${cyan}%m${reset}${gray}:${reset}${yellow}%~${reset}\n${gray}└─${reset}${bold}>${reset} "'

alias precmd 'set gitbr = `git branch |& grep "^\*" | sed "s/^\* //"`; if ("$gitbr" != "") set prompt = "${gray}┌─${reset}${green}%n${reset}${gray}@${reset}${cyan}%m${reset}${gray}:${reset}${yellow}%~${reset} ${gray}(${reset}${magenta}$gitbr${reset}${gray})${reset}\n${gray}└─${reset}${bold}>${reset} "; if ("$gitbr" == "") set prompt = "${gray}┌─${reset}${green}%n${reset}${gray}@${reset}${cyan}%m${reset}${gray}:${reset}${yellow}%~${reset}\n${gray}└─${reset}${bold}>${reset} "'

# =============================================================================
# ALIASES - Modern replacements and shortcuts
# =============================================================================


# Navigation
alias ..      'cd ..'
alias ...     'cd ../..'
alias ....    'cd ../../..'
alias .....   'cd ../../../..'
alias ~       'cd ~'
alias -- -    'cd -'


# Modern ls replacements
if ( -x /usr/bin/eza ) then
    alias ls  'eza --icons --group-directories-first --color=always'
    alias ll  'eza -l --icons --group-directories-first --color=always --git'
    alias la  'eza -la --icons --group-directories-first --color=always --git'
    alias lt  'eza --tree --icons --color=always --level=2'
    alias ltl 'eza --tree --icons --color=always --level=3 --long'
else if ( -x /usr/bin/lsd ) then
    alias ls  'lsd --group-dirs first --color always'
    alias ll  'lsd -l --group-dirs first --color always'
    alias la  'lsd -la --group-dirs first --color always'
    alias lt  'lsd --tree --color always --depth 2'
else
    alias ls  'ls --color=auto -h --group-directories-first'
    alias ll  'ls -l --color=auto -h --group-directories-first'
    alias la  'ls -la --color=auto -h --group-directories-first'
    alias lt  'ls --tree --color=auto -L 2'
endif


# File operations with safety
alias cp      'cp -iv'
alias mv      'mv -iv'
alias rm      'rm -iv'
alias mkdir   'mkdir -pv'
alias ln      'ln -v'


# Better defaults
alias grep    'grep --color=auto -i'
alias egrep   'egrep --color=auto -i'
alias fgrep   'fgrep --color=auto -i'
alias diff    'diff --color=auto -u'
alias less    'less -R'
alias more    'less'
alias cat     'bat --style=plain --paging=never 2>/dev/null || cat'
alias head    'head -n 20'
alias tail    'tail -n 20'


# System info
alias df      'df -h'
alias du      'du -h'
alias free    'free -h'
alias ps      'ps auxf'
alias psg     'ps aux | grep -v grep | grep -i'
alias ports   'netstat -tulanp 2>/dev/null || ss -tulanp'
alias myip    'curl -s ifconfig.me && echo'
alias localip 'ip route get 1 | awk "{print $7}"'


# Git shortcuts
alias g       'git'
alias gs      'git status -sb'
alias ga      'git add'
alias gc      'git commit -v'
alias gca     'git commit -va'
alias gp      'git push'
alias gl      'git pull --rebase'
alias glog    'git log --oneline --graph --decorate --all -20'
alias gd      'git diff'
alias gds     'git diff --staged'
alias gco     'git checkout'
alias gcb     'git checkout -b'
alias gb      'git branch -v'
alias gba     'git branch -av'
alias gr      'git remote -v'
alias gst     'git stash'
alias gstp    'git stash pop'
alias gf      'git fetch --all --prune'


# Docker shortcuts
alias d       'docker'
alias dc      'docker compose'
alias dps     'docker ps --format "table {{.Names}}\t{{.Image}}\t{{.Status}}\t{{.Ports}}"'
alias dpsa    'docker ps -a --format "table {{.Names}}\t{{.Image}}\t{{.Status}}\t{{.Ports}}"'
alias di      'docker images --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedSince}}"'
alias dlog    'docker logs -f --tail=100'
alias dexec   'docker exec -it'
alias dstop   'docker stop'
alias drm     'docker rm'
alias drmi    'docker rmi'
alias dprune  'docker system prune -af --volumes'


# Kubernetes shortcuts
alias k       'kubectl'
alias kgp     'kubectl get pods -o wide'
alias kgs     'kubectl get svc -o wide'
alias kgd     'kubectl get deploy -o wide'
alias kgn     'kubectl get nodes -o wide'
alias kdp     'kubectl describe pod'
alias kds     'kubectl describe svc'
alias kdd     'kubectl describe deploy'
alias kl      'kubectl logs -f --tail=100'
alias kex     'kubectl exec -it'


# Directory shortcuts
alias dls     'dirs -v'
alias pd      'pushd'
alias ppd     'popd'

# Archive shortcuts
alias tarcz   'tar -czvf'
alias tarxz   'tar -xzvf'
alias tarcj   'tar -cjvf'
alias tarxj   'tar -xjvf'


# Utility
alias h       'history'
alias j       'jobs -l'
alias which   'type -a'
alias path    'echo $PATH | tr ":" "\n"'
alias reload  'source ~/.cshrc && echo "~/.cshrc reloaded"'
alias cls     'clear && ls'
alias please  'sudo \!\!'
alias fuck    'sudo \!\!'  # For when you forget sudo


# =============================================================================
# KEY BINDINGS
# =============================================================================

# Emacs-style bindings (default, but explicit)
bindkey -e


# History search with arrow keys
bindkey -k up history-search-backward
bindkey -k down history-search-forward

# Word navigation
bindkey "^[f" forward-word
bindkey "^[b" backward-word

# Delete key
bindkey "^[[3~" delete-char


# =============================================================================
# COMPLETION ENHANCEMENTS
# =============================================================================

# Case-insensitive completion
set complete = enhance

# Complete from history
set histlit

# Show completions horizontally
set listjobs = long


# =============================================================================
# WINDOW TITLE
# =============================================================================

# Window title is now set in the main precmd alias above
# alias settitle 'echo -n "\[\033]0;%n@%m: %~\007\]"'
# alias precmd 'settitle'


# =============================================================================
# ENVIRONMENT VARIABLES
# =============================================================================

# Development paths
setenv PATH "/usr/local/bin:/usr/bin:/bin:/usr/local/sbin:/usr/sbin:/sbin:${HOME}/.local/bin:${HOME}/bin:${PATH}"


# Go
if ( -d /usr/local/go/bin ) setenv PATH "${PATH}:/usr/local/go/bin"
if ( -d ${HOME}/go/bin ) setenv PATH "${PATH}:${HOME}/go/bin"


# Rust
if ( -f ${HOME}/.cargo/env ) then
    # Source cargo env in a tcsh-compatible way
    # The .cargo/env is bash syntax, so we extract the PATH addition manually
    echo $PATH | grep -q "${HOME}/.cargo/bin"
    if ( $status != 0 ) then
        setenv PATH "${HOME}/.cargo/bin:${PATH}"
    endif
endif


# Python (pyenv)
if ( -d ${HOME}/.pyenv ) then
    setenv PYENV_ROOT "${HOME}/.pyenv"
    setenv PATH "${PYENV_ROOT}/bin:${PATH}"
    eval `pyenv init -`
endif


# Node (nvm)
if ( -d ${HOME}/.nvm ) then
    setenv NVM_DIR "${HOME}/.nvm"
    if ( -f "${NVM_DIR}/nvm.sh" ) source "${NVM_DIR}/nvm.sh"
    if ( -f "${NVM_DIR}/bash_completion" ) source "${NVM_DIR}/bash_completion"
endif


# =============================================================================
# FUNCTIONS (tcsh aliases)
# =============================================================================


# Create directory and cd into it
alias mkcd 'mkdir -p \!* && cd \!*'

# Extract any archive - using alias with tcsh-compatible switch
alias extract 'set _file = \!*; if ( ! -f "$_file" ) then echo "'\''$_file'\'' is not a valid file"; else; set _ext = `echo "$_file" | sed "s/.*\\.//"`; switch ("$_ext") case tar.bz2: tar xjf "$_file"; breaksw; case tar.gz: tar xzf "$_file"; breaksw; case bz2: bunzip2 "$_file"; breaksw; case rar: unrar x "$_file"; breaksw; case gz: gunzip "$_file"; breaksw; case tar: tar xf "$_file"; breaksw; case tbz2: tar xjf "$_file"; breaksw; case tgz: tar xzf "$_file"; breaksw; case zip: unzip "$_file"; breaksw; case Z: uncompress "$_file"; breaksw; case 7z: 7z x "$_file"; breaksw; default: echo "'\''$_file'\'' cannot be extracted via extract()"; endsw; endif'

# Find files by name
alias ff 'find . -type f -iname "*\!* *" 2>/dev/null'

# Find directories by name
alias fd 'find . -type d -iname "*\!* *" 2>/dev/null'

# Search for text in files
alias sg 'grep -r -i "\!*" . 2>/dev/null | head -50'

# Quick backup
alias bak 'cp \!* \!*.bak.`date +%Y%m%d_%H%M%S`'

# Weather
alias wttr 'curl -s "wttr.in/\!*" | head -38'

# Cheat sheet
alias cheat 'curl -s "cheat.sh/\!*"'


# =============================================================================
# LOCAL OVERRIDES (not tracked in git)
# =============================================================================

if ( -f ~/.cshrc.local ) then
    source ~/.cshrc.local
endif


# =============================================================================
# WELCOME MESSAGE
# =============================================================================

#if ( $?prompt ) then
	#    echo ""
	#    echo "${green}Welcome back, %n\!${reset}"
	#    echo "${gray}Host: %m | Shell: tcsh $tcsh | Time: %T${reset}"
	#    echo "${gray}Directory: %~${reset}"
	#    echo ""
	#endif

if ( $?prompt ) then
    echo ""
    echo "\033[1;32mWelcome back, $USER\!\033[0m"
    echo "\033[0;90mHost: $HOST | Shell: tcsh $tcsh | Time: `date +%T`\033[0m"
    echo "\033[0;90mDirectory: $cwd\033[0m"
    echo ""
endif


# End of ~/.cshrc

# Autostart tmux unless already inside a session or running non-interactively
#if ( ! $?TMUX && $?prompt ) then
    #    tmux attach-session -t default || tmux new-session -s default
    #endif
# Autostart tmux cleanly without session errors
if ( ! $?TMUX && $?prompt ) then
    tmux has-session -t default >& /dev/null
    if ( $status == 0 ) then
        tmux attach-session -t default
    else
        tmux new-session -s default
    endif
endif

