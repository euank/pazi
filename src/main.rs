#[macro_use]
extern crate clap;
extern crate libc;
extern crate rmp_serde;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate xdg;
mod frecency;

use serde::Serialize;
use clap::{App, Arg, ArgGroup};

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
"#
        );
        std::process::exit(0);
    };


    let xdg_dirs =
        xdg::BaseDirectories::with_prefix("pazi").expect("unable to determine xdg config path");

    let frecency_path = xdg_dirs
        .place_config_file("pazi_dirs.msgpack")
        .expect("could not create xdg 'pazi_dirs.msgpack' path");
    let frecency_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(frecency_path.clone())
        .unwrap();
    let metadata = frecency_file.metadata().unwrap();

    // remember 500 entries total
    let mut frecency = frecency::Frecency::<String>::new(500);

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
            .place_config_file(format!(".pazi_dirs.msgpack.{}", my_pid))
            .expect("could not create xdg 'pazi_dirs.msgpack' path");
        let frecency_update_file = std::fs::File::create(frecency_update_path.clone()).unwrap();

        frecency
            .serialize(&mut rmp_serde::Serializer::new(frecency_update_file))
            .unwrap();
        std::fs::rename(frecency_update_path, frecency_path).expect("could not update msgpack db");
        std::process::exit(0);
    };

    if let Some(to) = flags.value_of("dir") {
        for dir in frecency.items() {
            if dir.contains(to) {
                print!("{}", dir);
                std::process::exit(0);
            }
        }
        std::process::exit(1);
    };
}
