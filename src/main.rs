#[macro_use]
extern crate chan;
extern crate chan_signal;
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

mod shells;
mod importers;
mod matcher;
mod frecency;
mod frecent_paths;
mod interactive;

use std::env;
use std::process;

use clap::{App, Arg, ArgGroup, SubCommand};
use frecent_paths::PathFrecency;
use shells::SUPPORTED_SHELLS;

const PAZI_DB_NAME: &str = "pazi_dirs.msgpack";

fn main() {
    let flags = App::new("pazi")
        .about("A fast autojump tool")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::with_name("debug")
                .help("print debug information to stderr")
                .long("debug")
                .env("PAZI_DEBUG"),
        )
        .subcommand(
            SubCommand::with_name("init")
                .about("Prints intialization logic for the given shell to eval")
                .usage(format!("pazi init [ {} ]", SUPPORTED_SHELLS.join(" | ")).as_str())
                .arg(Arg::with_name("shell").help(&format!(
                    "the shell to print initialization code for: one of {}",
                    SUPPORTED_SHELLS.join(", ")
                ))),
        )
        .subcommand(
            SubCommand::with_name("import")
                .about("Import from another autojump program")
                .usage("pazi import fasd")
                .arg(Arg::with_name("autojumper").help(&format!(
                    "the other autojump program to import from, only fasd is currently supported",
                ))),
        )
        .arg(
            Arg::with_name("dir")
                .help(
                    "print a directory matching a pattern; should be used via the 'z' function \
                     'init' creates",
                )
                .long("dir")
                .short("d"),
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
        // Note: dir_target is a positional argument since it is desirable that both of the
        // following work:
        //
        // $ z -i asdf
        // $ z asdf -i
        // 
        // A positional argument was the only way I could figure out to do that without writing
        // more shell in init.
        .arg(Arg::with_name("dir_target"))
        .group(ArgGroup::with_name("operation").args(&["dir", "add-dir"]))
        .get_matches();

    if let Some(init_matches) = flags.subcommand_matches("init") {
        match init_matches.value_of("shell") {
            Some(s) => match shells::from_name(s) {
                Some(s) => {
                    println!("{}", s.pazi_init());
                    std::process::exit(0);
                }
                None => {
                    println!("{}\n\nUnsupported shell: {}", init_matches.usage(), s);
                    std::process::exit(1);
                }
            },
            None => {
                println!("{}\n\ninit requires an argument", init_matches.usage());
                std::process::exit(1);
            }
        }
    }

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

    if let Some(import_matches) = flags.subcommand_matches("import") {
        match import_matches.value_of("autojumper") {
            Some("fasd") => match importers::Fasd::import(&mut frecency) {
                Ok(stats) => match frecency.save_to_disk() {
                    Ok(_) => {
                        println!(
                            "imported {} items from fasd (out of {} in its db)",
                            stats.items_visited, stats.items_considered
                        );
                        process::exit(0);
                    }
                    Err(e) => {
                        println!("pazi: error adding directory: {}", e);
                        process::exit(1);
                    }
                },
                Err(e) => {
                    println!("pazi: error importing from fasd: {}", e);
                    process::exit(1);
                }
            },
            Some(s) => {
                println!(
                    "{}\n\nUnsupported import target: {}",
                    import_matches.usage(),
                    s
                );
                std::process::exit(1);
            }
            None => {
                println!("{}\n\nimport requires an argument", import_matches.usage());
                std::process::exit(1);
            }
        }
    }

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
    } else if flags.is_present("dir") {
        // Safe to unwrap because 'dir' requires 'dir_target'
        let matches = match flags.value_of("dir_target") {
            Some(to) => {
                env::current_dir().map(|cwd| {
                    frecency.maybe_add_relative_to(cwd, to);
                }).unwrap_or(()); // truly ignore failure to get cwd
                frecency.directory_matches(to)
            },
            None => frecency.items_with_frecency(),
        };
        if matches.len() == 0 {
            process::exit(1);
        }

        if flags.is_present("interactive") {
            let stdout = termion::get_tty().unwrap();
            match interactive::filter(matches, std::io::stdin(), stdout) {
                Ok(el) => {
                    print!("{}", el);
                }
                Err(e) => {
                    println!("{}", e);
                    process::exit(1);
                }
            }
        } else {
            // unwrap is safe because of the 'matches.len() == 0' check above.
            print!("{}", matches.last().unwrap().0);
        }
    } else if flags.value_of("dir_target") != None {
        // Something got interpreted as 'dir_target' even though this wasn't in '--dir'
        println!("pazi: could not parse given flags.\n\n{}", flags.usage());
        process::exit(1);
    } else {
        // By default print the frecency
        for el in frecency.items_with_frecency() {
            println!("{:.4}\t{}", el.1 * 100f64, el.0);
        }
    }
    if let Err(e) = frecency.save_to_disk() {
        // leading newline in case it was after a 'print' above
        println!("\npazi: error saving db changes: {}", e);
        process::exit(1);
    }
}
