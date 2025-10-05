# fogo - Navigate forest worktrees
# Fish shell function
function fogo
    if test "$argv[1]" = "-h" -o "$argv[1]" = "--help"
        echo "fogo - Navigate forest worktrees"
        echo ""
        echo "Usage:"
        echo "  fogo               List all trees"
        echo "  fogo <root>        Navigate to root directory"
        echo "  fogo <root> <tree> Navigate to tree directory"
        echo "  fogo -h            Show this help"
        return 0
    end

    set argc (count $argv)

    if test $argc -eq 0
        forest trees list
    else if test $argc -eq 1
        set target_path (forest roots path $argv[1] 2>/dev/null)
        if test -n "$target_path"
            builtin cd $target_path
        else
            echo "Error: root '$argv[1]' not found" >&2
            return 1
        end
    else if test $argc -eq 2
        set target_path (forest trees path $argv[1] $argv[2] 2>/dev/null)
        if test -n "$target_path"
            builtin cd $target_path
        else
            echo "Error: tree '$argv[1]/$argv[2]' not found" >&2
            return 1
        end
    else
        echo "Error: too many arguments" >&2
        echo "Run 'fogo -h' for usage information" >&2
        return 1
    end
end
