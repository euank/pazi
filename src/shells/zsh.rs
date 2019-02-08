use super::Shell;

pub struct Zsh;

impl Shell for Zsh {
    fn pazi_init(&self) -> &'static str {
        concat!(
            r#"
__pazi_add_dir() {
    pazi visit "${PWD}"
}

autoload -Uz add-zsh-hook
add-zsh-hook chpwd __pazi_add_dir

pazi_cd() {
    if [ "$#" -eq 0 ]; then pazi view; return $?; fi
    local res
    res="$("#,
            PAZI_EXTENDED_EXIT_CODES_ENV!(),
            r#"=1 pazi jump "$@")"
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
"#,
            r#"_pazi_cd() {
  CURRENTWORD="${LBUFFER/* /}${RBUFFER/ */}"
  local subcmds=(${(f)"$(pazi complete $CURRENTWORD)"})
  _describe -t pazi-dirs 'pazi' subcmds
}

compdef _pazi_cd pazi_cd 'pazi jump'
zstyle ':completion::complete:pazi_cd:*:pazi-dirs' matcher 'l:|=* r:|=*'
"#
        )
    }
}
