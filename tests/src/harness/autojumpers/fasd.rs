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
            &Shell::Bash => {
                format!(r#"
# Ensure history is not blank; fasd grabs commands to process from history
# (https://github.com/clvv/fasd/blob/90b531a5daaa545c74c7d98974b54cbdb92659fc/fasd#L127-L130)
# and, if history is empty, will error out
echo "echo hello world" >> ~/.bash_history

eval "$({} --init posix-alias bash-hook)"
"#, self.bin_path())
            }
            &Shell::Zsh => {
                format!(r#"eval "$({} --init posix-alias zsh-hook)""#, self.bin_path())
            }
            &Shell::Conch => unimplemented!(),
        }
    }
}
