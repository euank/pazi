#[cfg(feature = "integ-tests")]
mod integ_tests {
    extern crate tempdir;
    use self::tempdir::TempDir;
    use std::process::{Command, Stdio};
    use std::path::{Path, PathBuf};
    use std::fs;
    use std::io::Write;
    use std::env;
    use std::time::Duration;
    use std::thread::sleep;

    fn pazi_bin() -> PathBuf {
        // target/.../deps/integ...
        let mut pazi = env::current_exe().unwrap();
        pazi.pop(); // integ-... bin
        pazi.pop(); // deps folder
        pazi
    }

    #[test]
    fn it_jumps() {
        let h = Harness::new(&pazi_bin().join("pazi"), "zsh");

        h.create_dir("/tmp");
        h.visit_dir("/tmp");
        assert_eq!(h.jump("t"), "/tmp");
    }

    #[test]
    fn it_jumps_to_more_frecent_items() {
        let h = Harness::new(&pazi_bin().join("pazi"), "zsh");

        h.create_dir("/a/tmp");
        h.create_dir("/b/tmp");
        // Visiting 'b' more recently means it shouldbe more frecent.
        h.visit_dir("/a/tmp");
        sleep(Duration::from_millis(5));
        h.visit_dir("/b/tmp");
        assert_eq!(h.jump("tmp"), "/b/tmp");

        // Visiting 'a' more often should make it more 'frecent'
        for _ in (0..10) {
            h.visit_dir("/a/tmp");
        };
        h.visit_dir("/b/tmp");
        assert_eq!(h.jump("tmp"), "/a/tmp");

    }

    struct Harness {
        root: PathBuf,
        shell: String,
    }

    impl Harness {
        fn new(pazi: &Path, shell: &str) -> Self {
            let h = Harness{
                root: TempDir::new("pazi_integ").unwrap().into_path(),
                shell: shell.to_string(),
            };
            h.create_dir("/home/pazi");
            let shellrc = Path::new(&h.root).join(format!("home/pazi/.{}rc", shell));
            let mut rc = fs::File::create(shellrc).unwrap();
            rc.write_all(format!(r#"eval "$("{}" --init)""#, pazi.to_str().unwrap()).as_bytes()).unwrap();
            h
        }

        fn create_dir(&self, path: &str) {
            let p = Path::new(&self.root).join(Path::new(path).strip_prefix("/").unwrap());
            fs::create_dir_all(p).unwrap();
        }

        fn visit_dir(&self, path: &str) {
            let p = Path::new(&self.root).join(Path::new(path).strip_prefix("/").unwrap());
            let status = Command::new(&self.shell)
                .env("HOME", Path::new(&self.root).join("home/pazi"))
                .args(vec!["-i", "-c", &format!("cd '{}'", p.to_string_lossy().to_string())])
                .stdin(Stdio::null())
                .status()
                .unwrap();
            assert!(status.success());
        }

        fn jump(&self, search: &str) -> String {
            let output = Command::new(&self.shell)
                .env("HOME", Path::new(&self.root).join("home/pazi"))
                .args(vec!["-i", "-c", &format!("z '{}' && pwd", search)])
                .stdin(Stdio::null())
                .output()
                .unwrap();
            if !output.status.success() {
                panic!(
                    "jumping exited with error: {}: {}, {}",
                    output.status,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr),
                );
            }
            assert!(output.status.success());
            String::from_utf8(output.stdout).unwrap().to_string()
                .trim_left_matches(&self.root.to_string_lossy().to_string())
                .trim_right()
                .to_string()
        }
    }

    impl Drop for Harness {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).unwrap();
        }
    }
}
