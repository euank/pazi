#[macro_use]
extern crate clap;
#[macro_use]
extern crate crossbeam_channel as channel;
extern crate directories;
extern crate env_logger;
extern crate libc;
#[macro_use]
extern crate log;
extern crate rmp_serde;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate signal_hook;
extern crate snailquote;
extern crate tempfile;
extern crate termion;
extern crate which;

#[macro_use]
mod pazi_result;

mod edit;
mod frecency;
mod frecent_paths;
mod importers;
mod interactive;
mod matcher;
mod shells;

use std::env;
use std::path::PathBuf;

use clap::{App, AppSettings, Arg, ArgGroup, ArgMatches, SubCommand};
use crate::frecent_paths::PathFrecency;
use crate::pazi_result::*;
use crate::shells::SUPPORTED_SHELLS;

const PAZI_DB_NAME: &str = "pazi_dirs.msgpack";

fn main() {
    let res = _main();
    let extended_exit_codes = match std::env::var(PAZI_EXTENDED_EXIT_CODES_ENV!()) {
        Ok(_) => true,
        Err(_) => false,
    };
    if extended_exit_codes {
        debug!("using extended exit codes");
        std::process::exit(res.extended_exit_code());
    } else {
        std::process::exit(res.exit_code());
    }
}

// SUBCOMMAND is a macro-enum of all subcommands.
// This should be replaced by a normal enum + const "as_str" for each variant once rust stable
// supports const functions.
macro_rules! SUBCOMMAND {
    (Edit) => {
        "edit"
    };
    (Import) => {
        "import"
    };
    (Init) => {
        "init"
    };
    (Jump) => {
        "jump"
    };
    (View) => {
        "view"
    };
    (Visit) => {
        "visit"
    };
    (Complete) => {
        "complete"
    }
}

fn _main() -> PaziResult {
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
            SubCommand::with_name(SUBCOMMAND!(Edit))
                .about("Edit the frecency database")
                .usage("pazi edit [<filter>]")
                .arg(Arg::with_name("filter").help(
                    "filter matches down further and edit that subset"
                )),
        )
        .subcommand(
            SubCommand::with_name(SUBCOMMAND!(Init))
                .about("Prints initialization logic for the given shell to eval")
                .usage(format!("pazi init [ {} ]", SUPPORTED_SHELLS.join(" | ")).as_str())
                .arg(Arg::with_name("shell").help(&format!(
                    "the shell to print initialization code for: one of {}",
                    SUPPORTED_SHELLS.join(", ")
                ))),
        )
        .subcommand(
            SubCommand::with_name(SUBCOMMAND!(Import))
                .about("Import from another autojump program")
                .usage("pazi import fasd")
                .arg(Arg::with_name("autojumper").help(&format!(
                    "the other autojump program to import from, only fasd is currently supported",
                ))),
        )
        .subcommand(
            SubCommand::with_name(SUBCOMMAND!(Jump))
                // used by the shell alias internally, it shouldn't be called directly
                .setting(AppSettings::Hidden)
                .setting(AppSettings::DisableHelpSubcommand)
                .about("Select a directory to jump to")
                .arg(
                    Arg::with_name("interactive")
                        .help("interactively search directory matches")
                        .long("interactive")
                        .short("i"),
                )
                .arg(Arg::with_name("dir_target"))
        )
        .subcommand(
            SubCommand::with_name(SUBCOMMAND!(View))
                .setting(AppSettings::DisableHelpSubcommand)
                .about("View the frecency database")
                .arg(
                    Arg::with_name("dir_target")
                        .help("filter matches down further")
                )
        )
        .subcommand(
            SubCommand::with_name(SUBCOMMAND!(Visit))
                // used by the shell hooks internally, it shouldn't be called directly
                .setting(AppSettings::Hidden)
                .setting(AppSettings::DisableHelpSubcommand)
                .about("Add or visit a directory in the frecency database")
                .arg(Arg::with_name("dir_target"))
        )
        .subcommand(
            SubCommand::with_name(SUBCOMMAND!(Complete))
            // used by the shell completion functions internally, it shouldn't be called directly
                .setting(AppSettings::Hidden)
                .setting(AppSettings::DisableHelpSubcommand)
                .about("Returns possible completions for shells")
                .arg(Arg::with_name("dir_target"))
        )
        // Code after this comment is deprecated in favor of .PaziSubcommand::Jump, but is left in
        // temporarily for backwards compatibility. Remove before 1.0
        .arg(
            Arg::with_name("dir")
                .help(
                    "print a directory matching a pattern; should be used via the 'z' function \
                     'init' creates",
                )
                .hidden(true)
                .long("dir")
                .short("d"),
        )
        .arg(
            Arg::with_name("interactive")
                .help("interactively search directory matches")
                .long("interactive")
                .short("i"),
        )
        // Deprecated in favor of .PaziSubcommand::Visit
        // left temporarily for backwards compatibility
        // Remove before 1.0
        .arg(
            Arg::with_name("add-dir")
                .help("add a directory to the frecency index")
                .hidden(true)
                .long("add-dir")
                .takes_value(true)
                .value_name("directory"),
        )
        // Deprecated per above deprecations
        // Remove before 1.0
        // Note: dir_target is a positional argument since it is desirable that both of the
        // following work:
        //
        // $ z -i asdf
        // $ z asdf -i
        // 
        // A positional argument was the only way I could figure out to do that without writing
        // more shell in init.
        .arg(Arg::with_name("dir_target").hidden(true))
        .group(ArgGroup::with_name("operation").args(&["dir", "add-dir"]))
        .get_matches();

    if flags.is_present("debug") {
        env_logger::Builder::new()
            .filter(None, log::LevelFilter::Debug)
            .try_init()
            .unwrap();

        // Capture ctrl-c so calling script
        // can print debug output
        if let Err(()) = intercept_ctrl_c() {
            return PaziResult::Error;
        }
    }

    match flags.subcommand() {
        (SUBCOMMAND!(Edit), Some(edit)) => {
            return handle_edit(edit);
        }
        (SUBCOMMAND!(Import), Some(import)) => {
            return handle_import(import);
        }
        (SUBCOMMAND!(Init), Some(init)) => {
            return handle_init(init);
        }
        (SUBCOMMAND!(Jump), Some(jump)) => {
            return handle_jump(jump);
        }
        (SUBCOMMAND!(View), Some(view)) => {
            return handle_print_frecency(view);
        }
        (SUBCOMMAND!(Visit), Some(visit)) => {
            return handle_visit(visit);
        }
        (SUBCOMMAND!(Complete), Some(completion)) => {
            return handle_completion(completion);
        }
        unknown => debug!(
            "unrecognized subcommand: not an error for backwards compatibility: {:?}",
            unknown
        ),
    };

    // the remainder of this fn is backwards compatibility code, all of this should vanish before
    // 1.0
    let path = frecency_path().expect("could not get frecency db path");
    let mut frecency = PathFrecency::load(&path);

    let res;
    if let Some(dir) = flags.value_of("add-dir") {
        frecency.visit(dir.to_string());

        match frecency.save_to_disk() {
            Ok(_) => {
                return PaziResult::Success;
            }
            Err(e) => {
                println!("pazi: error adding directory: {}", e);
                return PaziResult::Error;
            }
        }
    } else if flags.is_present("dir") {
        let mut matches = match flags.value_of("dir_target") {
            Some(to) => {
                env::current_dir()
                    .map(|cwd| {
                        frecency.maybe_add_relative_to(cwd, to);
                    })
                    .unwrap_or(()); // truly ignore failure to get cwd
                frecency.directory_matches(to)
            }
            None => frecency.items_with_frecency(),
        };
        if flags.is_present("interactive") {
            let stdout = termion::get_tty().unwrap();
            match interactive::filter(matches, std::io::stdin(), stdout) {
                Ok(el) => {
                    print!("{}", el);
                    res = PaziResult::SuccessDirectory;
                }
                Err(interactive::FilterError::NoSelection) => {
                    return PaziResult::ErrorNoInput;
                }
                Err(e) => {
                    println!("{}", e);
                    return PaziResult::Error;
                }
            }
        } else if let Some((path, _)) = matches.next() {
            print!("{}", path);
            res = PaziResult::SuccessDirectory;
        } else {
            res = PaziResult::Error;
        }
    } else if flags.value_of("dir_target") != None {
        // Something got interpreted as 'dir_target' even though this wasn't in '--dir'
        println!("pazi: could not parse given flags.\n\n{}", flags.usage());
        return PaziResult::Error;
    } else {
        // By default print the frecency
        for el in frecency.items_with_frecency() {
            // precision for floats only handles the floating part, which leads to unaligned
            // output, e.g., for a precision value of '3', you might get:
            // 1.000
            // 100.000
            //
            // By converting it to a string first, and then truncating it, we can get nice prettily
            // aligned strings.
            // Note: the string's precision should be at least as long as the printed precision so
            // there are enough characters.
            let str_val = format!("{:.5}", (el.1 * 100f64));
            println!("{:.5}\t{}", str_val, el.0);
        }
        res = PaziResult::Success;
    }
    if let Err(e) = frecency.save_to_disk() {
        // leading newline in case it was after a 'print' above
        println!("\npazi: error saving db changes: {}", e);
        return PaziResult::Error;
    }
    res
}

fn load_frecency() -> PathFrecency {
    let path = frecency_path().expect("could not get frecency db path");

    PathFrecency::load(&path)
}

fn handle_edit(cmd: &ArgMatches) -> PaziResult {
    let mut frecency = load_frecency();

    let mut fclone = frecency.clone();
    let matches = match cmd.value_of("filter") {
        Some(filter) => fclone.directory_matches_raw(filter),
        None => fclone.items_with_frecency_raw(),
    };
    let match_vec = matches.collect();
    let diff = match edit::edit(&match_vec) {
        Ok(d) => d,
        Err(e) => {
            println!("Error editing: {}", e);
            return PaziResult::Error;
        }
    };
    match frecency.apply_diff(diff) {
        Err(e) => {
            println!("Error applying edit diff: {}", e);
            return PaziResult::Error;
        }
        Ok(()) => {}
    };
    match frecency.save_to_disk() {
        Ok(_) => {
            return PaziResult::Success;
        }
        Err(e) => {
            println!("pazi: error saving db: {}", e);
            return PaziResult::Error;
        }
    }
}

fn handle_init(cmd: &ArgMatches) -> PaziResult {
    match cmd.value_of("shell") {
        Some(s) => match shells::from_name(s) {
            Some(s) => {
                println!("{}", s.pazi_init());
                return PaziResult::Success;
            }
            None => {
                println!("{}\n\nUnsupported shell: {}", cmd.usage(), s);
                return PaziResult::Error;
            }
        },
        None => {
            println!("{}\n\ninit requires an argument", cmd.usage());
            return PaziResult::Error;
        }
    }
}

fn handle_import(cmd: &ArgMatches) -> PaziResult {
    let mut frecency = load_frecency();

    let stats = match cmd.value_of("autojumper") {
        Some("fasd") => match importers::Fasd::import(&mut frecency) {
            Ok(stats) => stats,
            Err(e) => {
                println!("error importing: {}", e);
                return PaziResult::Error;
            }
        },
        Some(s) => {
            println!("{}\n\nUnsupported import target: {}", cmd.usage(), s);
            return PaziResult::Error;
        }
        None => {
            println!("{}\n\nimport requires an argument", cmd.usage());
            return PaziResult::Error;
        }
    };

    match frecency.save_to_disk() {
        Ok(_) => {
            println!(
                "imported {} items from fasd (out of {} in its db)",
                stats.items_visited, stats.items_considered
            );
            PaziResult::Success
        }
        Err(e) => {
            println!("pazi: error adding directory: {}", e);
            PaziResult::Error
        }
    }
}

fn handle_jump(cmd: &ArgMatches) -> PaziResult {
    let mut frecency = load_frecency();
    let res;

    let mut matches = match cmd.value_of("dir_target") {
        Some(to) => {
            env::current_dir()
                .map(|cwd| {
                    frecency.maybe_add_relative_to(cwd, to);
                })
                .unwrap_or(()); // truly ignore failure to get cwd
            frecency.directory_matches(to)
        }
        None => frecency.items_with_frecency(),
    };

    res = if cmd.is_present("interactive") {
        let stdout = termion::get_tty().unwrap();
        match interactive::filter(matches, std::io::stdin(), stdout) {
            Ok(el) => {
                print!("{}", el);
                PaziResult::SuccessDirectory
            }
            Err(interactive::FilterError::NoSelection) => {
                // early return since no selection arbitrarily implies not trimming non-existent
                // paths. The early return skips the save_to_disk below
                return PaziResult::ErrorNoInput;
            }
            Err(e) => {
                println!("{}", e);
                return PaziResult::Error;
            }
        }
    } else if let Some((path, _)) = matches.next() {
        print!("{}", path);
        PaziResult::SuccessDirectory
    } else {
        PaziResult::Error
    };

    if let Err(e) = frecency.save_to_disk() {
        // leading newline in case it was after a 'print' above
        println!("\npazi: error saving db changes: {}", e);
        return PaziResult::Error;
    }
    res
}

fn handle_visit(cmd: &ArgMatches) -> PaziResult {
    let dir = match cmd.value_of("dir_target") {
        Some(dir) => dir,
        None => {
            println!("visit: visit requires a directory target to visit");
            return PaziResult::Error;
        }
    };

    let mut frecency = load_frecency();
    frecency.visit(dir.to_string());

    match frecency.save_to_disk() {
        Ok(_) => {
            return PaziResult::Success;
        }
        Err(e) => {
            println!("pazi: error adding directory: {}", e);
            return PaziResult::Error;
        }
    }
}

fn handle_print_frecency(cmd: &ArgMatches) -> PaziResult {
    let mut frecency = load_frecency();

    let matches = match cmd.value_of("dir_target") {
        Some(to) => frecency.directory_matches(to),
        None => frecency.items_with_frecency(),
    };

    for el in matches {
        // precision for floats only handles the floating part, which leads to unaligned
        // output, e.g., for a precision value of '3', you might get:
        // 1.000
        // 100.000
        //
        // By converting it to a string first, and then truncating it, we can get nice prettily
        // aligned strings.
        // Note: the string's precision should be at least as long as the printed precision so
        // there are enough characters.
        let str_val = format!("{:.5}", (el.1 * 100f64));
        println!("{:.5}\t{}", str_val, el.0);
    }

    PaziResult::Success
}

fn handle_completion(cmd: &ArgMatches) -> PaziResult {
    let mut frecency = load_frecency();

    let matches = match cmd.value_of("dir_target") {
        Some(to) => frecency.directory_matches(to),
        None => frecency.items_with_frecency(),
    };

    for el in matches {
        let str_val = format!("{:.5}", (el.1 * 100f64));
        println!("{}:{}", el.0, str_val);
    }

    PaziResult::Success
}

fn frecency_path() -> Result<PathBuf, String> {
    let project_dir = directories::ProjectDirs::from("", "", "pazi")
        .ok_or("unable to determine config path".to_owned())?;
    let config_dir = project_dir.config_dir();

    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("could not create config dir: {}", e))?;

    let db_path = config_dir.join(PAZI_DB_NAME);
    Ok(db_path)
}

fn intercept_ctrl_c() -> Result<(), ()> {
    // When Pazi is run from a script or shell function,
    // pressing ctrl-c will send SIGINT to the process group
    // containing both Pazi *and* the shell function.
    //
    // However, sometimes we just want to SIGINT Pazi but
    // allow the caller to keep running (e.g., to print output).
    // To accomplish this, we need to put Pazi in its own
    // process group and make that the foreground process group.
    // That way, when ctrl-c sends a SIGINT, the only process
    // to receive it is Pazi.
    //
    unsafe fn get_errno() -> *mut libc::c_int {
        #[cfg(target_os = "linux")]
        return libc::__errno_location();
        #[cfg(target_os = "macos")]
        return libc::__error();
    }

    unsafe {
        // Create a new process group with this process in it.
        let setpgid_res = libc::setpgid(0, 0);
        let errno = *get_errno();
        if setpgid_res != 0 {
            debug!("Got {} from setpgid with errno {}", setpgid_res, errno);
            return Err(());
        }

        // Get the ID of the process group we just made.
        let pgrp = libc::getpgrp();

        // Make this process group the foreground process.
        // SIGTTOU is sent if this process group isn't already foreground,
        // so we ignore it during the change.

        // New SIGTTOU handler that ignores the signal
        let ignore_action = libc::sigaction {
            sa_sigaction: libc::SIG_IGN,
            sa_mask: std::mem::zeroed(),
            sa_flags: 0,
            #[cfg(target_os = "linux")]
            sa_restorer: None,
        };
        // Place to save old SIGTTOU handler
        let mut old_action = std::mem::zeroed();

        // Ignore SIGTTOU and save previous action
        let sigaction_res = libc::sigaction(libc::SIGTTOU, &ignore_action, &mut old_action);
        let errno = *get_errno();
        if sigaction_res != 0 {
            debug!("Got {} from sigaction with errno {}", sigaction_res, errno);
            return Err(());
        }

        // Make our process group the foreground process group
        // (giving us access to stdin, etc)
        let tcsetpgrp_res = libc::tcsetpgrp(libc::STDIN_FILENO, pgrp);
        let errno = *get_errno();

        // Put the old SIGTTOU signal handler back
        // We try to do this even if tcsetpgrp failed!
        let sigaction_res = libc::sigaction(libc::SIGTTOU, &old_action, std::ptr::null_mut());
        let sigaction_errno = *get_errno();

        // Handle tcsetpgrp and sigaction errors
        if tcsetpgrp_res != 0 || sigaction_res != 0 {
            debug!("Got pgrp {}", pgrp);
            debug!("Got {} from tcsetpgrp with errno {}", tcsetpgrp_res, errno);
            debug!(
                "Got {} from sigaction with errno {}",
                sigaction_res, sigaction_errno
            );
            return Err(());
        }
    }

    Ok(())
}
