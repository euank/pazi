use super::Autojumper;
use crate::harness::Shell;
use std::env;
use std::path::Path;

pub struct Pazi;

impl Autojumper for Pazi {
    fn bin_path(&self) -> String {
        let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("build with cargo");
        let pazi = Path::new(&crate_dir)
            .join("../target/release/pazi")
            .canonicalize()
            .unwrap();
        if !pazi.exists() {
            panic!("compile pazi in release mode before running bench/integ tests");
        }
        pazi.to_str().unwrap().to_string()
    }

    fn init_for(&self, shell: &Shell) -> String {
        match shell {
            &Shell::Bash => format!(r#"set -u; eval "$(pazi init {})""#, shell.name()),
            &Shell::Zsh => format!(r#"set -u; autoload -Uz compinit; compinit -u; eval "$(pazi init {})""#, shell.name()),
            &Shell::Fish => "status --is-interactive; and pazi init fish | source".to_string(),
        }
    }

    fn supported_shells(&self) -> Vec<Shell> {
        vec![Shell::Bash, Shell::Zsh, Shell::Fish]
    }

    fn jump_alias(&self) -> &'static str {
        "z"
    }

    fn to_str(&self) -> &'static str {
        "pazi"
    }
}
