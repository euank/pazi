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
pub use self::autojumpers::None as NoJumper;

pub struct Harness {
    testshell: TestShell,
}

impl Harness {
    pub fn new(root: &Path, jumper: &Autojumper, shell: &Shell) -> Self {
        let ps1 = "==PAZI==> ";
        let home = Path::new(&root).join("home/pazi");
        shell.setup(&root, jumper, ps1);

        let mut cmd = shell.command(&root);
        let testshell = TestShell::new(cmd, ps1);
        Harness {
            testshell: testshell,
        }
    }

    pub fn create_dir(&self, path: &str) {
        fs::create_dir_all(path).unwrap();
    }

    pub fn visit_dir(&mut self, path: &str) {
        self.testshell.run(&format!("cd '{}'", path));
    }

    pub fn jump(&mut self, search: &str) -> String {
        self.testshell
            .run(&format!("z '{}' && pwd", search))
            .to_string()
    }

    pub fn run_cmd(&mut self, cmd: &str) -> String {
        self.testshell.run(cmd)
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        self.testshell.shutdown();
    }
}
