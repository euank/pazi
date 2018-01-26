use super::Shell;

pub struct Bash;

impl Shell for Bash {
    fn pazi_init(&self) -> &'static str {
        // PROMPT_COMMAND modification inspired by https://github.com/clvv/fasd/blob/90b531a5daaa545c74c7d98974b54cbdb92659fc/fasd#L132-L136
        concat!(
            r#"
__pazi_add_dir() {
    # TODO: should pazi keep track of this itself in its datadir?
    if [[ "${__PAZI_LAST_PWD}" != "${PWD}" ]]; then
        pazi --add-dir "${PWD}"
    fi
    __PAZI_LAST_PWD="${PWD}"
}

case \$PROMPT_COMMAND in
    *__pazi_add_dir\;*) ;;
    *) PROMPT_COMMAND="__pazi_add_dir;\$PROMPT_COMMMAND" ;;
esac

pazi_cd() {
    if [ "$#" -eq 0 ]; then pazi; return $?; fi
    local res; "#, /* note: this is declared separately because 'local' clobbers pazi's return
    code, see https://lists.gnu.org/archive/html/bug-bash/2010-03/msg00007.html */
            r#"
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
    *) echo "${res}" && return $ret;;
    esac
}
alias z='pazi_cd'
"#
        )
    }
}
