use super::Shell;

pub struct Fish;

impl Shell for Fish {
    fn pazi_init(&self) -> &'static str {
        concat!(
            r#"
function pazi_cd
    if [ (count $argv) -eq 0 ]
        pazi view
        return $status
    else
        set -l res (env "#,
            PAZI_EXTENDED_EXIT_CODES_ENV!(),
            r#"=1 pazi jump $argv)
        set -l ret $status
        switch $ret
        case "#,
            EXIT_CODE!(SUCCESS),
            r#"; echo $res
        case "#,
            EXIT_CODE!(SUCCESS_DIR),
            r#"; cd $res
        case "#,
            EXIT_CODE!(ERROR),
            r#"; echo $res; and return 1
        case "#,
            EXIT_CODE!(ERROR_NO_INPUT),
            r#"; return 1
        case '*'
            echo $res; and return $ret
        end
    end
end

function __pazi_preexec --on-variable PWD
    status --is-command-substitution; and return
    pazi visit (pwd) &; disown 2>/dev/null
end

alias z 'pazi_cd'
"#
        )
    }
}
