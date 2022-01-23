use super::Autojumper;
use crate::harness::Shell;
use std::env;
use std::path::{PathBuf, Path};

pub struct Fasd;

impl Autojumper for Fasd {
    fn bin_path(&self) -> PathBuf {
        let bin = env::var("FASD_BIN").expect("run with the makefile");
        let fasd_path = Path::new(&bin);

        if !fasd_path.exists() {
            panic!("{:?} should exist", fasd_path);
        }
        fasd_path
            .canonicalize()
            .unwrap()
    }

    fn init_for(&self, shell: &Shell) -> String {
        match *shell {
            Shell::Bash => format!(
                r#"
# Ensure history is not blank; fasd grabs commands to process from history
# (https://github.com/clvv/fasd/blob/90b531a5daaa545c74c7d98974b54cbdb92659fc/fasd#L127-L130)
# and, if history is empty, will error out
echo "echo hello world" >> ~/.bash_history

eval "$({} --init posix-alias bash-hook)"
"#,
                self.bin_path().to_string_lossy()
            ),
            Shell::Zsh => format!(
                r#"eval "$({} --init posix-alias zsh-hook)""#,
                self.bin_path().to_string_lossy()
            ),
            Shell::Fish => {
                unimplemented!("fasd does not support fish");
            }
        }
    }

    fn supported_shells(&self) -> Vec<Shell> {
        vec![Shell::Bash, Shell::Zsh]
    }

    fn jump_alias(&self) -> &'static str {
        "z"
    }

    fn to_str(&self) -> &'static str {
        "fasd"
    }
}
