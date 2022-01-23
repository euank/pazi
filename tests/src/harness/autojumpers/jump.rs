use super::Autojumper;
use crate::harness::Shell;
use std::env;
use std::path::{PathBuf, Path};

pub struct Jump;

impl Autojumper for Jump {
    fn bin_path(&self) -> PathBuf {
        let bin = env::var("JUMP_BIN").expect("JUMP_BIN environment variable should be set");
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
eval "$({} shell {})"
"#,
                self.bin_path().to_string_lossy(),
                shell.name(),
            ),
            &Shell::Fish => format!(
                r#"
status --is-interactive; and . ({} shell {} | psub)
"#,
                self.bin_path().to_string_lossy(),
                shell.name(),
            ),
        }
    }

    fn supported_shells(&self) -> Vec<Shell> {
        vec![Shell::Bash, Shell::Zsh]
    }

    fn jump_alias(&self) -> &'static str {
        "j"
    }

    fn to_str(&self) -> &'static str {
        "jump"
    }
}
