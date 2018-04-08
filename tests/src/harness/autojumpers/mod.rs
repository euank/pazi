pub mod pazi;
pub mod fasd;
pub mod autojump;
pub mod z;

use harness::Shell;

pub trait Autojumper {
    fn bin_path(&self) -> String;
    fn init_for(&self, shell: &Shell) -> String;
    fn jump_alias(&self) -> &'static str;
}

// None is a non-autojumping shell for benchmarking against.
// Testshells configured with it cannot jump
pub struct None;

impl Autojumper for None {
    fn bin_path(&self) -> String {
        "".to_owned()
    }
    fn init_for(&self, _: &Shell) -> String {
        "".to_owned()
    }
    fn jump_alias(&self) -> &'static str{
        panic!("'None' can't jump");
    }
}
