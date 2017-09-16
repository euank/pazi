#[macro_use]
extern crate clap;

use clap::{App, Arg, ArgGroup};

fn main() {
    let flags = App::new("pazi")
        .about("An autojump tool for zsh")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::with_name("init")
            .help("provide initialization hooks to eval in your shell")
            .long("init"))
        .arg(Arg::with_name("dir")
             .help("show a directory matching a pattern")
             .long("dir")
             .short("d")
             .takes_value(true)
             .value_name("fuzzy directory search"))
        .arg(Arg::with_name("add-dir")
             .help("add a directory to the frecency index")
             .long("add-dir")
             .takes_value(true)
             .value_name("directory"))
        .group(ArgGroup::with_name("operation")
               .args(&["init", "dir", "add-dir"]))
        .get_matches();

    if flags.is_present("init") {
        println!("{}", r#"
pazi_add_dir() {
    pazi --add-dir "${PWD}"
}

autoload -Uz add-zsh-hook
add-zsh-hook chpwd pazi_add_dir

pazi_cd() {
    local to=$(pazi --dir "$@")
    [ -z "${to}" ] && return 1
    cd "${to}"
}
alias z='pazi_cd'
"#);
        std::process::exit(0);
    };

    if let Some(dir) = flags.value_of("add-dir") {
        println!("I would add {}", dir);
        std::process::exit(0);
    };

    if let Some(to) = flags.value_of("dir") {
        println!("{}", "/tmp/");
        std::process::exit(0);
    };
}
