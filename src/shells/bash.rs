use super::Shell;

pub struct Bash;

impl Shell for Bash {
    fn pazi_init(&self) -> &'static str {
        // PROMPT_COMMAND modification inspired by https://github.com/clvv/fasd/blob/90b531a5daaa545c74c7d98974b54cbdb92659fc/fasd#L132-L136
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
    local to="$(pazi --dir "$@")"
    local ret=$?
    if [ "${ret}" != "0" ]; then return "$ret"; fi
    [ -z "${to}" ] && return 1
    cd "${to}"
}
alias z='pazi_cd'
"#
    }
}
