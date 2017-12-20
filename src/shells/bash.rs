use super::Shell;

pub struct Bash;

impl Shell for Bash {
    fn pazi_init(&self) -> &'static str {
        // ty to mklement0 for this suggested append method:
        // https://stackoverflow.com/questions/3276247/is-there-a-hook-in-bash-to-find-out-when-the-cwd-changes#comment35222599_3276280
        // Used under cc by-sa 3.0
        r#"
__pazi_add_dir() {
    # TODO: should pazi keep track of this itself in its datadir?
    if [[ "${__PAZI_LAST_PWD}" != "${PWD}" ]]; then
        pazi --add-dir "${PWD}"
    fi
    __PAZI_LAST_PWD="${PWD}"
}

if [[ -z "${PROMPT_COMMAND}" ]]; then
    PROMPT_COMMAND="__pazi_add_dir;"
else
    PROMPT_COMMAND="$(read newVal <<<"$PROMPT_COMMAND"; echo "${newVal%;}; __pazi_add_dir;")"
fi

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
