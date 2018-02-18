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

pub struct HarnessBuilder<'a> {
    root: &'a Path,
    shell: &'a Shell,
    jumper: &'a Autojumper,
    preinit: Option<&'a str>,
    cgroup: bool,
}

impl<'a> HarnessBuilder<'a> {
    pub fn new(root: &'a Path, jumper: &'a Autojumper, shell: &'a Shell) -> Self {
        HarnessBuilder {
            root: root,
            shell: shell,
            jumper: jumper,
            cgroup: false,
            preinit: None,
        }
    }

    pub fn preinit(mut self, preinit: &'a str) -> Self {
        self.preinit = Some(preinit);
        self
    }

    pub fn cgroup(mut self, cgroup: bool) -> Self {
        self.cgroup = cgroup;
        self
    }

    pub fn finish(self) -> Harness<'a> {
        Harness::new(self.root, self.shell, self.jumper, self.preinit, self.cgroup)
    }
}

impl<'a> Harness<'a> {
    fn new(root: &Path, shell: &Shell, jumper: &'a Autojumper, preinit: Option<&str>, cgroup: bool) -> Self {
        let ps1 = "==PAZI==> ";
        shell.setup(&root, jumper, ps1, preinit.unwrap_or(""));

        let cmd = shell.command(&root);
        let testshell = if cgroup {
            TestShell::new_in_cgroup(cmd, ps1, PAZI_CG)
        } else {
            TestShell::new(cmd, ps1)
        };
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

    // wait for any children of the shell to vanish; this is approximated by assuming that the
    // shell will be the only child task of this process.
    pub fn wait_children(&mut self) {
        self.testshell.wait_children()
    }
}

const PAZI_CG: &'static str = "pazi_integ";

impl<'a> Drop for Harness<'a> {
    fn drop(&mut self) {
        self.testshell.shutdown();
    }
}
