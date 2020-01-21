use std::fs;
use std::io::Read;
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use rand::prelude::*;

use super::shells;

pub struct TestShell {
    // fork is here for lifetime reasons; can't drop it until the pty is done
    #[allow(unused)]
    fork: pty::fork::Fork,
    pty: pty::fork::Master,
    pid: libc::pid_t,
    output: mpsc::Receiver<String>,
    eof: mpsc::Receiver<()>,
    // cgroup, if set, is the cgroup this test shell is running in
    cgroup: Option<String>,
}

// VTEData is to handle lines after the mess of vte terminal stuff.
// It keeps track of newlines and such
#[derive(Debug)]
struct VTEData {
    current_line_cursor: usize,
    pub current_line: String,
    pub scrollback: Vec<String>,
}

// VTEDataLen is used as a sorta cheap hash for comparing whether VTEData has changed in a
// meaningful way.
#[derive(PartialEq, Clone, Debug)]
struct VTEDataLen {
    pub current_line: usize,
    pub scrollback: usize,
}

impl VTEData {
    fn new() -> Self {
        VTEData {
            current_line_cursor: 0,
            current_line: String::new(),
            scrollback: Vec::new(),
        }
    }

    fn len(&self) -> VTEDataLen {
        VTEDataLen {
            current_line: self.current_line.len(),
            scrollback: self.scrollback.len(),
        }
    }
}

impl vte::Perform for VTEData {
    fn print(&mut self, c: char) {
        self.current_line.truncate(self.current_line_cursor);
        self.current_line_cursor += c.len_utf8();
        self.current_line.push(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte as char {
            '\n' => {
                self.scrollback.push(self.current_line.clone());
                self.current_line.truncate(0);
            }
            '\r' => {
                self.current_line_cursor = 0;
            }
            '\x08' => {
                // backspace
                if self.current_line_cursor > 0 {
                    self.current_line_cursor -= 1;
                    self.current_line.pop();
                }
            }
            '\t' => {
                self.print('\t');
            }
            '\x0b' => {
                // vertical tab
                self.print('\x0b');
            }
            _ => {
                println!("[VTEData execute]: ignoring {}", byte);
            }
        }
    }

    fn hook(&mut self, _: &[i64], _: &[u8], _: bool, _: char) {
        // ignore
    }

    fn put(&mut self, _: u8) {
        // ignore
    }

    fn unhook(&mut self) {
        // ignore
    }

    fn osc_dispatch(&mut self, _: &[&[u8]]) {
        // ignore
    }

    fn csi_dispatch(&mut self, _: &[i64], _: &[u8], _: bool, _: char) {
        // ignore
    }

    fn esc_dispatch(&mut self, _: &[i64], _: &[u8], _: bool, _: u8) {
        // ignore
    }
}

impl TestShell {
    // new creates a new testshell. It is assumed that the passed in command is for a posix-ish
    // shell. The shell should print output generally line-by-line and after executing a command,
    // it should print the PS1 variable.
    // This PS1 variable is used to determine when commands have executed, so no commands run in
    // this testshell may print the PS1 value.
    // Note: this command does fork off a child. There are dragons. Handle with care.
    pub fn new(cmd: shells::ShellCmd, ps1: &str) -> Self {
        Self::new_internal(cmd, ps1, None)
    }

    pub fn new_in_cgroup(cmd: shells::ShellCmd, ps1: &str, cgroup: &str) -> Self {
        Self::new_internal(cmd, ps1, Some(cgroup))
    }

    fn new_internal(cmd: shells::ShellCmd, ps1: &str, cgroup: Option<&str>) -> Self {
        let mut cgpath = None;
        if let Some(cg) = cgroup {
            if unsafe { libc::geteuid() } != 0 {
                panic!("cgroup pid tracking requires root");
            }
            let scope = format!("test_shell_{}", rand::thread_rng().gen::<u64>());
            let unified_path = format!("/sys/fs/cgroup/unified/{}.slice/{}.scope", cg, scope);
            fs::create_dir_all(&unified_path).expect("could not create cg directory");
            cgpath = Some(unified_path);
        };
        let mut shellcmd = Command::new(cmd.cmd);
        shellcmd.env_clear();
        shellcmd.env("PS1", ps1);
        shellcmd.env("PATH", std::env::var("PATH").unwrap());
        shellcmd.env("TERM", "xterm");
        for env in cmd.env {
            shellcmd.env(env.0, env.1);
        }
        let fork = pty::fork::Fork::from_ptmx().unwrap();

        let child_pid;
        let mut pty = match fork {
            pty::fork::Fork::Child(_) => {
                let err = shellcmd.exec();
                panic!("exec failed: {}", err);
            }
            pty::fork::Fork::Parent(c, m) => {
                child_pid = c;
                m.grantpt().unwrap();
                m.unlockpt().unwrap();
                m
            }
        };

        if let Some(cg) = cgpath.clone() {
            let mut f = fs::OpenOptions::new()
                .write(true)
                .open(format!("{}/cgroup.procs", cg))
                .expect("no cgroup.procs file");
            f.write(format!("{}\n", child_pid).as_bytes())
                .expect("write pid err");
        }

        let (write_command_out, command_out) = mpsc::channel();
        let (write_eof_got, eof_got) = mpsc::channel();

        // To move into the thread
        let ps12 = ps1.to_owned();
        thread::spawn(move || {
            // vte stuff
            let mut data = VTEData::new();
            let mut statemachine = vte::Parser::new();
            // Keep a record of the last vte-length info we saw so we can detect meaningful
            // changes.
            let mut last_len = data.len();

            // Have we seen the starting PS1 yet?
            let mut last_prompt_scrollback_count = -1;
            // What's been output since the last PS1 + command happened?
            let mut current_command_output = Vec::new();
            loop {
                let mut buf: [u8; 4 * 1024] = [0; 4 * 1024];
                let nread = pty.read(&mut buf).unwrap();
                if nread == 0 {
                    // EOF
                    if current_command_output.len() > 0
                        && last_prompt_scrollback_count != data.scrollback.len() as i32
                    {
                        // Have unsent scrollback data; go ahead and send it
                        write_command_out
                            .send(current_command_output.join("\n"))
                            .unwrap();
                    }
                    write_eof_got.send(()).unwrap();
                    return;
                }
                for byte in &buf[..nread] {
                    statemachine.advance(&mut data, *byte);
                    if last_len == data.len() {
                        // control character or whatever, we don't care
                        continue;
                    }

                    if data.current_line == ps12
                        && last_prompt_scrollback_count < data.scrollback.len() as i32
                    {
                        // Exactly equal to PS1 means that there's a new blank PS1 prompt
                        // Either we just started up, or a command just finished.
                        write_command_out
                            .send(current_command_output.join("\n"))
                            .unwrap();
                        current_command_output.truncate(0);
                        // mark that we've seen this prompt, don't handle it again even if there's
                        // backspacing
                        last_prompt_scrollback_count = data.scrollback.len() as i32;
                    } else if data.scrollback.len() > last_len.scrollback {
                        // this only happens if the last character was a newline since we're
                        // checking this every statemachine advance.
                        let last_line = data.scrollback.last().unwrap();
                        // skip PS1 starting things since we assume that's a command being entered,
                        // e.g....
                        //     PS1 $ ls
                        //     file1 file2
                        //     PS1 $
                        // We're avoiding the first line there
                        if !last_line.starts_with(&ps12) {
                            current_command_output.push(last_line.to_string());
                        }
                    }
                    last_len = data.len();
                }
            }
        });

        let first_output = command_out
            .recv_timeout(Duration::from_secs(5))
            .expect("did not get initial prompt");

        // Happens if the shell prints errors, etc. during startup
        if first_output != "" {
            panic!(
                "Encountered errors during shell startup: {:?}",
                first_output
            );
        }

        TestShell {
            fork: fork,
            pid: child_pid,
            pty: pty,
            eof: eof_got,
            output: command_out,
            cgroup: cgpath,
        }
    }

    pub fn run(&mut self, cmd: &str) -> String {
        self.pty.write(format!("{}\n", cmd).as_bytes()).unwrap();
        self.output.recv_timeout(Duration::from_secs(100)).unwrap()
    }

    pub fn wait_children(&mut self) {
        if self.cgroup.is_none() {
            self.wait_children_pgrep();
        } else {
            self.wait_children_cgroup()
        }
    }

    fn wait_children_pgrep(&mut self) {
        // a pgrep based implementation which is slower than the group based implementation.
        // It's used so integ tests can run without cgroups (e.g. on macos), and as a nice side
        // benefit allows them to run without root on linux.
        // Benchmarks should use the cgroup method for performance reasons.
        loop {
            let child_pids = Command::new("pgrep").args(vec!["-P", &format!("{}", self.pid)]).output().unwrap();
            // don't check status because 'pgrep -P $x' returns '1' if it matches no pids, which is
            // a success case for us
            if child_pids.stderr.len() > 0 {
                panic!("unable to wait for children. Error running 'pgrep -P {}': {}", self.pid, String::from_utf8_lossy(&child_pids.stderr));
            }
            let pids: Vec<i32> = String::from_utf8(child_pids.stdout).unwrap().lines().map(str::parse).collect::<Result<Vec<_>, _>>().unwrap();
            if pids.len() == 0 {
                return;
            };
            unsafe {
                let mut status = 0;
                libc::waitpid(pids[0], &mut status, libc::WEXITED);
            };
        }
    }

    fn wait_children_cgroup(&mut self) {
        // TODO: don't assume unified
        let cgprocfile = format!("{}/cgroup.procs", self.cgroup.clone().unwrap());

        let mut output = String::new();
        let mut pids = Vec::new();
        loop {
            let mut f = fs::File::open(Path::new(&cgprocfile)).unwrap();
            output.truncate(0);
            pids.truncate(0);
            f.read_to_string(&mut output).unwrap();
            for line in output.lines() {
                let pid = line.parse::<i32>().unwrap();
                if pid == self.pid {
                    continue;
                }
                pids.push(pid);
            }
            if pids.len() == 0 {
                return;
            }
            unsafe {
                let mut status = 0;
                libc::waitpid(pids[0], &mut status, libc::WEXITED);
            };
        }
    }

    pub fn shutdown(&mut self) {
        self.pty.write("exit\n".as_bytes()).unwrap();
        self.eof.recv().unwrap();
    }
}

#[cfg(features = "testshell-dev")]
mod dev {
    use super::TestShell;
    use std::process::Command;
    #[test]
    fn testshell() {
        let mut cmd = Command::new("zsh");
        let mut ts = TestShell::new(cmd, "==> ");
        assert_eq!(ts.run("cd /tmp"), "");
        assert_eq!(ts.run("echo foo"), "foo");
        assert_eq!(ts.run(r#"echo -e "foo\nbar\nbaz" | tac"#), "baz\nbar\nfoo");
        ts.shutdown();
    }
}
