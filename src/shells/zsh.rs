use super::Shell;

pub struct Zsh;

impl Shell for Zsh {
    fn pazi_init(&self) -> &'static str {
        concat!(
            r#"
__pazi_add_dir() {
    pazi --add-dir "${PWD}"
}

autoload -Uz add-zsh-hook
add-zsh-hook chpwd __pazi_add_dir

pazi_cd() {
    if [ "$#" -eq 0 ]; then pazi; return $?; fi
    local res
    res="$("#,
            PAZI_EXTENDED_EXIT_CODES_ENV!(),
            r#"=1 pazi --dir "$@")"
    local ret=$?
    case $ret in
    "#,
            EXIT_CODE!(SUCCESS),
            r#") echo "${res}";;
    "#,
            EXIT_CODE!(SUCCESS_DIR),
            r#") cd "${res}";;
    "#,
            EXIT_CODE!(ERROR),
            r#") echo "${res}" && return 1;;
    "#,
            EXIT_CODE!(ERROR_NO_INPUT),
            r#") return 1;;
    *) echo "${res}" && return $ret;;
    esac
}
alias z='pazi_cd'
"#
        )
    }
}
