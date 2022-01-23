use super::Autojumper;
use crate::harness::Shell;
use std::env;
use std::path::{Path, PathBuf};

pub struct Z;

impl Autojumper for Z {
    fn bin_path(&self) -> PathBuf {
        let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("build with cargo");
        let bin_path = Path::new(&crate_dir).join(format!("testbins/z/z.sh"));

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
