use super::Shell;

pub struct Zsh;

impl Shell for Zsh {
    fn pazi_init(&self) -> &'static str {
        r#"
__pazi_add_dir() {
    pazi --add-dir "${PWD}"
}

autoload -Uz add-zsh-hook
add-zsh-hook chpwd __pazi_add_dir

pazi_cd() {
    if [ "$#" -eq 0 ]; then pazi; return $?; fi
    [[ "$@[(r)--help]" == "--help" ]] && pazi --help && return 0
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
