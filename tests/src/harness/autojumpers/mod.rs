pub mod autojump;
pub mod fasd;
pub mod jump;
pub mod zoxide;
pub mod pazi;
pub mod z;

use std::path::PathBuf;

use crate::harness::Shell;

pub trait Autojumper {
    fn bin_path(&self) -> PathBuf;
    fn init_for(&self, shell: &Shell) -> String;
    fn jump_alias(&self) -> &'static str;
    fn supported_shells(&self) -> Vec<Shell>;
    fn to_str(&self) -> &'static str;
}

// None is a non-autojumping shell for benchmarking against.
// Testshells configured with it cannot jump
pub struct None;

impl Autojumper for None {
    fn bin_path(&self) -> PathBuf {
        "".into()
    }
    fn init_for(&self, _: &Shell) -> String {
        "".to_owned()
    }
    fn supported_shells(&self) -> Vec<Shell> {
        Vec::new()
    }
    fn jump_alias(&self) -> &'static str {
        panic!("'None' can't jump");
    }
    fn to_str(&self) -> &'static str {
        "none"
    }
}
