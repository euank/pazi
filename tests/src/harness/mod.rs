mod testshell;
mod shells;
mod autojumpers;

use std::path::Path;
use self::testshell::TestShell;
use std::fs;

pub use self::shells::Shell;
pub use self::autojumpers::Autojumper;
pub use self::autojumpers::pazi::Pazi;
pub use self::autojumpers::fasd::Fasd;
pub use self::autojumpers::autojump::Autojump;
pub use self::autojumpers::None as NoJumper;

pub struct Harness<'a> {
    testshell: TestShell,
    jumper: &'a Autojumper,
}

impl<'a> Harness<'a> {
    pub fn new_with_preinit(root: &Path, jumper: &'a Autojumper, shell: &Shell, preinit: &str) -> Self {
        Self::new_helper(root, jumper, shell, preinit)
    }

    pub fn new(root: &Path, jumper: &'a Autojumper, shell: &Shell) -> Self {
        Self::new_helper(root, jumper, shell, "")
    }

    fn new_helper(root: &Path, jumper: &'a Autojumper, shell: &Shell, preinit: &str) -> Self {
        let ps1 = "==PAZI==> ";
        shell.setup(&root, jumper, ps1, preinit);

        let cmd = shell.command(&root);
        let testshell = TestShell::new(cmd, ps1);
        Harness {
            testshell: testshell,
            jumper: jumper,
        }
    }

    pub fn create_dir(&self, path: &str) {
        fs::create_dir_all(path).unwrap();
    }

    pub fn delete_dir(&self, path: &str) {
        fs::remove_dir_all(path).unwrap();
    }

    pub fn visit_dir(&mut self, path: &str) {
        self.testshell.run(&format!("cd '{}'", path));
    }

    pub fn jump(&mut self, search: &str) -> String {
        self.testshell.run(&format!("{} '{}' >/dev/null && pwd", self.jumper.jump_alias(), search))
    }

    pub fn run_cmd(&mut self, cmd: &str) -> String {
        self.testshell.run(cmd)
    }
}

impl<'a> Drop for Harness<'a> {
    fn drop(&mut self) {
        self.testshell.shutdown();
    }
}
