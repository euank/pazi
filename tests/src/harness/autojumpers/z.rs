use super::Autojumper;
use crate::harness::Shell;
use std::env;
use std::path::{Path, PathBuf};

pub struct Z;

impl Autojumper for Z {
    fn bin_path(&self) -> PathBuf {
        let bin = env::var("RUPA_Z_BIN").expect("RUPA_Z_BIN should be set");
        let bin_path = Path::new(&bin);

        if !bin_path.exists() {
            panic!("update submodules before running benches");
        }
        bin_path
            .canonicalize()
            .unwrap()
    }

    fn init_for(&self, shell: &Shell) -> String {
        match shell {
            &Shell::Bash | &Shell::Zsh => format!(
                r#"
. "{}"
"#,
                self.bin_path().to_string_lossy()
            ),
            &Shell::Fish => unimplemented!("z does not support fish"),
        }
    }

    fn supported_shells(&self) -> Vec<Shell> {
        vec![Shell::Bash, Shell::Zsh]
    }

    fn jump_alias(&self) -> &'static str {
        "z"
    }

    fn to_str(&self) -> &'static str {
        "rupa-z"
    }
}
