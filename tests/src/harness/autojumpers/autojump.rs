use super::Autojumper;
use crate::harness::Shell;
use std::env;
use std::path::Path;

// https://github.com/wting/autojump
pub struct Autojump;

impl Autojump {
    fn shell_path(&self, shell: &str) -> String {
        let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("build with cargo");
        let shell_path =
            Path::new(&crate_dir).join(format!("testbins/autojump/bin/autojump.{}", shell));

        if !shell_path.exists() {
            panic!("update submodules before running benches");
        }
        shell_path
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }
}

impl Autojumper for Autojump {
    fn bin_path(&self) -> String {
        let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("build with cargo");
        let bin_path = Path::new(&crate_dir).join(format!("testbins/autojump/bin/autojump"));

        if !bin_path.exists() {
            panic!("update submodules before running benches");
        }
        bin_path
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }

    fn init_for(&self, shell: &Shell) -> String {
        match shell {
            &Shell::Bash => format!("source '{}'", self.shell_path("bash")),
            &Shell::Zsh => format!("source '{}'", self.shell_path("zsh")),
            &Shell::Fish => format!("source '{}'", self.shell_path("fish")),
        }
    }

    fn supported_shells(&self) -> Vec<Shell> {
        vec![Shell::Bash, Shell::Zsh, Shell::Fish]
    }

    fn jump_alias(&self) -> &'static str {
        "j"
    }

    fn to_str(&self) -> &'static str {
        "autojump"
    }
}
