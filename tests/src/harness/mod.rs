mod autojumpers;
mod shells;
mod testshell;

use self::testshell::TestShell;
use std::fs;
use std::path::Path;

pub use self::autojumpers::autojump::Autojump;
pub use self::autojumpers::fasd::Fasd;
pub use self::autojumpers::jump::Jump;
pub use self::autojumpers::pazi::Pazi;
pub use self::autojumpers::z::Z;
pub use self::autojumpers::Autojumper;
pub use self::autojumpers::None as NoJumper;
pub use self::shells::Shell;

pub struct Harness<'a> {
    testshell: TestShell,
    jumper: &'a Autojumper,
    shell: &'a Shell,
}

pub struct HarnessBuilder<'a> {
    root: &'a Path,
    shell: &'a Shell,
    jumper: &'a Autojumper,
    preinit: Option<&'a str>,
}

impl<'a> HarnessBuilder<'a> {
    pub fn new(root: &'a Path, jumper: &'a Autojumper, shell: &'a Shell) -> Self {
        HarnessBuilder {
            root: root,
            shell: shell,
            jumper: jumper,
            preinit: None,
        }
    }

    pub fn preinit(mut self, preinit: &'a str) -> Self {
        self.preinit = Some(preinit);
        self
    }

    pub fn finish(self) -> Harness<'a> {
        Harness::new(
            self.root,
            self.shell,
            self.jumper,
            self.preinit,
        )
    }
}

impl<'a> Harness<'a> {
    fn new(
        root: &Path,
        shell: &'a Shell,
        jumper: &'a dyn Autojumper,
        preinit: Option<&str>,
    ) -> Self {
        let ps1 = &format!("=={}=={}==>", shell.name(), jumper.to_str());
        shell.setup(&root, jumper, ps1, preinit.unwrap_or(""));

        let use_cgroup = std::env::var("PAZI_TEST_CGROUP") == Ok("true".to_string());
        let cmd = shell.command(&root);
        let testshell = if use_cgroup {
            TestShell::new_in_cgroup(cmd, ps1, PAZI_CG)
        } else {
            TestShell::new(cmd, ps1)
        };
        Harness {
            testshell: testshell,
            shell: shell,
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
        self.wait_children();
    }

    pub fn jump(&mut self, search: &str) -> String {
        let cmd = match self.shell {
            Shell::Bash | Shell::Zsh => {
                format!("{} '{}' >/dev/null && pwd", self.jumper.jump_alias(), search)
            },
            Shell::Fish => {
                format!("{} '{}' >/dev/null; and pwd", self.jumper.jump_alias(), search)
            }
        };

        self.testshell.run(&cmd)
    }

    pub fn run_cmd(&mut self, cmd: &str) -> String {
        self.testshell.run(cmd)
    }

    pub fn run_cmd_with_status(&mut self, cmd: &str) -> String {
        let cmd = match self.shell {
            Shell::Bash | Shell::Zsh => {
                format!("{} && echo $?", cmd)
            },
            Shell::Fish => {
                format!("{}; and echo $status", cmd)
            }
        };
        self.testshell.run(&cmd)
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
