#[macro_use]
extern crate clap;
extern crate libc;
extern crate rmp_serde;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate xdg;

mod matcher;
mod frecency;
mod frecent_paths;

use std::process;

use clap::{App, Arg, ArgGroup};
use frecent_paths::PathFrecency;


const PAZI_DB_NAME: &str = "pazi_dirs.msgpack";

fn main() {
    let flags = App::new("pazi")
        .about("An autojump tool for zsh")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::with_name("init")
                .help("provide initialization hooks to eval in your shell")
                .long("init"),
        )
        .arg(
            Arg::with_name("dir")
                .help("show a directory matching a pattern")
                .long("dir")
                .short("d")
                .takes_value(true)
                .value_name("fuzzy directory search"),
        )
        .arg(
            Arg::with_name("add-dir")
                .help("add a directory to the frecency index")
                .long("add-dir")
                .takes_value(true)
                .value_name("directory"),
        )
        .group(ArgGroup::with_name("operation").args(
            &["init", "dir", "add-dir"],
        ))
        .get_matches();

    if flags.is_present("init") {
        println!(
            "{}",
            r#"
__pazi_add_dir() {
    pazi --add-dir "${PWD}"
}

autoload -Uz add-zsh-hook
add-zsh-hook chpwd __pazi_add_dir

pazi_cd() {
    [ "$#" -eq 0 ] && pazi && return 0
    local to=$(pazi --dir "$@")
    [ -z "${to}" ] && return 1
    cd "${to}"
}
alias z='pazi_cd'
"#
        );
        std::process::exit(0);
    };


    let xdg_dirs =
        xdg::BaseDirectories::with_prefix("pazi").expect("unable to determine xdg config path");

    let frecency_path = xdg_dirs.place_config_file(PAZI_DB_NAME).expect(&format!(
        "could not create xdg '{}' path",
        PAZI_DB_NAME
    ));

    let mut frecency = PathFrecency::load(&frecency_path);

    if let Some(dir) = flags.value_of("add-dir") {
        frecency.visit(dir.to_string());

        match frecency.save_to_disk() {
            Ok(_) => {
                process::exit(0);
            }
            Err(e) => {
                println!("pazi: error adding directory: {}", e);
                process::exit(1);
            }
        }
    };

    if let Some(to) = flags.value_of("dir") {
        match frecency.best_directory_match(to) {
            Some(dir) => {
                print!("{}", dir);
                process::exit(0);
            }
            None => process::exit(1),
        }
    };

    // By default print the frecency
    for el in frecency.items_with_normalized_frecency() {
        println!("{:.4}\t{}", el.1 * 100f64, el.0);
    }
}
