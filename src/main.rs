#[macro_use]
extern crate clap;
extern crate libc;
extern crate rmp_serde;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate xdg;

mod frecency;

use std::path::Path;
use std::{fs, process};

use clap::{App, Arg, ArgGroup};
use frecency::Frecency;
use serde::Serialize;


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
        .group(
            ArgGroup::with_name("operation").args(&["init", "dir", "add-dir"]),
        )
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

    let frecency_path = xdg_dirs
        .place_config_file(PAZI_DB_NAME)
        .expect(&format!("could not create xdg '{}' path", PAZI_DB_NAME));
    let frecency_file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(frecency_path.clone())
        .unwrap();
    let metadata = frecency_file.metadata().unwrap();

    // remember 500 entries total
    let mut frecency = Frecency::<String>::new(500);

    if metadata.len() > 0 {
        // existing file, unmarshal that sucker
        let mut de = rmp_serde::Deserializer::from_read(frecency_file);
        frecency = serde::Deserialize::deserialize(&mut de).unwrap();
    }

    if let Some(dir) = flags.value_of("add-dir") {
        frecency.visit(dir.to_string());

        let my_pid = unsafe { libc::getpid() };
        if my_pid == 0 {
            panic!("getpid returned 0");
        }

        let frecency_update_path = xdg_dirs
            .place_config_file(format!(".{}.{}", PAZI_DB_NAME, my_pid))
            .expect(&format!(
                "could not create xdg '.{}.{}' path",
                PAZI_DB_NAME,
                my_pid
            ));

        write_frecency(&frecency, &frecency_path, &frecency_update_path)
            .expect("could not update frecency path");
        process::exit(0);
    };

    if let Some(to) = flags.value_of("dir") {
        for dir in frecency.items() {
            if dir.contains(to) {
                print!("{}", dir);
                process::exit(0);
            }
        }
        process::exit(1);
    };

    // By default print the frecency
    for el in frecency.items_with_frecency() {
        println!("{}\t{}", el.1, el.0);
    }
}

fn write_frecency(
    frecency: &Frecency<String>,
    target: &Path,
    tmp_path: &Path,
) -> std::io::Result<()> {
    let tmpfile = fs::File::create(tmp_path)?;
    frecency
        .serialize(&mut rmp_serde::Serializer::new(tmpfile))
        .unwrap();

    fs::rename(tmp_path, target)
}
