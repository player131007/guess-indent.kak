declare-option -docstring "Valid space indentations to consider" \
int-list guess_indent_standard_widths 2 4 8

declare-option -docstring "Maximum number of lines to guess indentation (-1 for infinite)" \
int guess_indent_max_lines -1

define-command -params 1 -docstring %{
    guess-indent-from-file <filename>: guess the indentation from the contents of <file>
} guess-indent-from-file %{
    evaluate-commands %sh{
        # replace space-separated list with comma-separated list
        widths=$(printf "%s," $kak_opt_guess_indent_standard_widths)
        widths=${widths%,}

        standard_widths=''
        if [ -n "$widths" ]; then
            standard_widths='--standard-widths="$widths"'
        fi

        max_lines=''
        if [ "$kak_opt_guess_indent_max_lines" -ge 0 ]; then
            max_lines='--max-lines="$kak_opt_guess_indent_max_lines"'
        fi

        block_comments=''
        if [ -n "$kak_opt_comment_block_begin" ] && [ -n "$kak_opt_comment_block_end" ]; then
            block_comments='--block-comment-start="$kak_opt_comment_block_begin" --block-comment-end="$kak_opt_comment_block_end"'
        fi

        eval kak-guess-indent "$max_lines" "$standard_widths" "$block_comments" '"$1"'
    }
}

hook -group guess-indent global BufOpenFile .* %{
    guess-indent-from-file %val{hook_param}
}

hook -group guess-indent global BufReload .* %{
    guess-indent-from-file %val{hook_param}
}

hook -group guess-indent global BufNewFile .* %{
    hook -group guess-indent -once buffer BufWritePost .* %{
        guess-indent-from-file %val{hook_param}
    }
}
