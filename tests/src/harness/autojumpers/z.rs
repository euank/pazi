use super::Autojumper;
use harness::Shell;
use std::env;
use std::path::Path;

pub struct Z;

impl Autojumper for Z {
    fn bin_path(&self) -> String {
        let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("build with cargo");
        let bin_path = Path::new(&crate_dir).join(format!("testbins/z/z.sh"));

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
            &Shell::Bash | &Shell::Zsh => format!(
                r#"
. "{}"
"#,
                self.bin_path()
            ),
            &Shell::Conch => unimplemented!(),
        }
    }

    fn jump_alias(&self) -> &'static str {
        "z"
    }
}
