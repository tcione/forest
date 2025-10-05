# fogo - Navigate forest worktrees
# Compatible with bash and zsh
fogo() {
  if [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
    echo "fogo - Navigate forest worktrees"
    echo ""
    echo "Usage:"
    echo "  fogo               List all trees"
    echo "  fogo <root>        Navigate to root directory"
    echo "  fogo <root> <tree> Navigate to tree directory"
    echo "  fogo -h            Show this help"
    return 0
  fi

  if [ $# -eq 0 ]; then
    forest trees list
  elif [ $# -eq 1 ]; then
    local target_path
    target_path=$(forest roots path "$1" 2>/dev/null)
    if [ -n "$target_path" ]; then
      builtin cd "$target_path" || return 1
    else
      echo "Error: root '$1' not found" >&2
      return 1
    fi
  elif [ $# -eq 2 ]; then
    local target_path
    target_path=$(forest trees path "$1" "$2" 2>/dev/null)
    if [ -n "$target_path" ]; then
      builtin cd "$target_path" || return 1
    else
      echo "Error: tree '$1/$2' not found" >&2
      return 1
    fi
  else
    echo "Error: too many arguments" >&2
    echo "Run 'fogo -h' for usage information" >&2
    return 1
  fi
}
