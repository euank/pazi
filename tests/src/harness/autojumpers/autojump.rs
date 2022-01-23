use super::Autojumper;
use crate::harness::Shell;
use std::env;
use std::path::{PathBuf, Path};

// https://github.com/wting/autojump
pub struct Autojump;

impl Autojump {
    fn shell_path(&self, shell: &str) -> String {
        let shell_path = Path::new(&env::var("AUTOJUMP_HOOKS").expect("AUTOJUMP_HOOKS environment variable should be set"))
            .join(format!("autojump.{}", shell));
        if !shell_path.exists() {
            panic!("expected {:?} to exist", shell_path);
        }
        shell_path
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }
}

impl Autojumper for Autojump {
    fn bin_path(&self) -> PathBuf {
        let bin = env::var("AUTOJUMP_BIN").expect("AUTOJUMP_BIN environment variable should be set for the test harness");
        let bin_path = Path::new(&bin);
        if !bin_path.exists() {
            panic!("update submodules before running benches");
        }
        bin_path.into()
    }

    fn init_for(&self, shell: &Shell) -> String {
        match *shell {
            Shell::Bash => format!("source '{}'", self.shell_path("bash")),
            Shell::Zsh => format!("source '{}'", self.shell_path("zsh")),
            Shell::Fish => format!("source '{}'", self.shell_path("fish")),
        }
    }

    fn supported_shells(&self) -> Vec<Shell> {
        vec![Shell::Bash, Shell::Zsh]
    }

    fn jump_alias(&self) -> &'static str {
        "j"
    }

    fn to_str(&self) -> &'static str {
        "autojump"
    }
}
