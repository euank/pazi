#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate libc;
#[macro_use]
extern crate log;
extern crate rmp_serde;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate termion;
extern crate xdg;

mod matcher;
mod frecency;
mod frecent_paths;
mod interactive;

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
            Arg::with_name("debug")
                .help("print debug information to stderr")
                .long("debug")
                .env("PAZI_DEBUG"),
        )
        .arg(
            Arg::with_name("init")
                .help("provide initialization hooks to eval in your shell")
                .long("init"),
        )
        .arg(
            Arg::with_name("dir")
                .help("print a directory matching a pattern; should be used via the 'z' function \
                      '--init' creates")
                .long("dir")
                .short("d")
                .requires("dir_target")
        )
        .arg(
            Arg::with_name("interactive")
                .help("interactively search directory matches")
                .long("interactive")
                .short("i"),
        )
        .arg(
            Arg::with_name("add-dir")
                .help("add a directory to the frecency index")
                .long("add-dir")
                .takes_value(true)
                .value_name("directory"),
        )
        .arg(
            Arg::with_name("dir_target")
        )
        .group(ArgGroup::with_name("operation").args(&["init", "dir", "add-dir"]))
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
    [[ "$@[(r)--help]" == "--help" ]] && pazi --help && return 0
    local to=$(pazi --dir "$@")
    [ -z "${to}" ] && return 1
    cd "${to}"
}
alias z='pazi_cd'
"#
        );
        std::process::exit(0);
    };

    if flags.is_present("debug") {
        env_logger::LogBuilder::new()
            .filter(None, log::LogLevelFilter::Debug)
            .init()
            .unwrap();
    }


    let xdg_dirs =
        xdg::BaseDirectories::with_prefix("pazi").expect("unable to determine xdg config path");

    let frecency_path = xdg_dirs
        .place_config_file(PAZI_DB_NAME)
        .expect(&format!("could not create xdg '{}' path", PAZI_DB_NAME));

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

    if flags.is_present("dir") {
        // Safe to unwrap because 'dir' requires 'dir_target'
        let to = flags.value_of("dir_target").unwrap();
        let matches = frecency.directory_matches(to);
        if matches.len() == 0 {
            process::exit(1);
        }

        if flags.is_present("interactive") {
            let stdout = termion::get_tty().unwrap();
            match interactive::filter(matches, std::io::stdin(), &stdout) {
                Ok(el) => {
                    print!("{}", el);
                    process::exit(0);
                }
                Err(e) => {
                    println!("{}", e);
                    process::exit(1);
                }
            }
        } else {
            // unwrap is safe because of the 'matches.len() == 0' check above.
            print!("{}", matches.first().unwrap().0);
            process::exit(0);
        }
    };

    // By default print the frecency
    for el in frecency.items_with_normalized_frecency() {
        println!("{:.4}\t{}", el.1 * 100f64, el.0);
    }
}
