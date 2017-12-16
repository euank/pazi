use super::Autojumper;
use harness::Shell;
use std::env;
use std::path::Path;

pub struct Fasd;

impl Autojumper for Fasd {
    fn bin_path(&self) -> String {
        let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("build with cargo");
        let fasd_path = Path::new(&crate_dir)
            .join(format!("testbins/fasd/fasd"));

        if !fasd_path.exists() {
            panic!("update submodules before running benches");
        }
        fasd_path.canonicalize().unwrap().to_string_lossy().to_string()
    }

    fn init_for(&self, shell: &Shell) -> String {
        match shell {
            &Shell::Bash | &Shell::Zsh => {
                format!(r#"eval "$({} --init posix-alias {}-hook)""#, self.bin_path(), shell.name())
            }
            &Shell::Conch => unimplemented!(),
        }
    }
}
