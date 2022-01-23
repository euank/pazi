use super::Autojumper;
use crate::harness::Shell;
use std::env;
use std::path::{PathBuf, Path};

pub struct Zoxide;

impl Autojumper for Zoxide {
    fn bin_path(&self) -> PathBuf {
        let bin = env::var("ZOXIDE_BIN").expect("ZOXIDE_BIN environment variable should be set");
        let bin_path = Path::new(&bin);

        if !bin_path.exists() {
            panic!("run tests with the makefile");
        }
        bin_path
            .canonicalize()
            .unwrap()
    }

    fn init_for(&self, shell: &Shell) -> String {
        match shell {
            &Shell::Bash | &Shell::Zsh => format!(
                r#"
eval "$({} init {})"
"#,
                self.bin_path().to_string_lossy(),
                shell.name(),
            ),
            &Shell::Fish => format!("{} init fish | source", self.bin_path().to_string_lossy()),
        }
    }

    fn supported_shells(&self) -> Vec<Shell> {
        vec![Shell::Bash, Shell::Zsh]
    }

    fn jump_alias(&self) -> &'static str {
        "z"
    }

    fn to_str(&self) -> &'static str {
        "zoxide"
    }
}
