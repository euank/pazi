#[cfg(feature = "integ-tests")]
mod integ_tests {
    extern crate tempdir;
    use self::tempdir::TempDir;
    use std::process::Command;
    use std::path::{Path, PathBuf};
    use std::fs;
    use std::io::Write;
    use std::env;

    #[test]
    fn it_works() {
        // target/.../deps/integ...
        let mut pazi = env::current_exe().unwrap();
        pazi.pop(); // integ-... bin
        pazi.pop(); // deps folder

        let h = Harness::new(&pazi.join("pazi"), "zsh");

        h.create_dir("/tmp");
        h.visit_dir("/tmp");
        assert_eq!(h.jump("t"), "/tmp");
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
            let status = Command::new(&self.shell)
                .env("HOME", Path::new(&self.root).join("home/pazi"))
                .args(vec!["-i", "-c", &format!("cd '{}'", path)])
                .status()
                .unwrap();
            assert!(status.success());
        }

        fn jump(&self, search: &str) -> String {
            let output = Command::new(&self.shell)
                .env("HOME", Path::new(&self.root).join("home/pazi"))
                .args(vec!["-i", "-c", &format!("z '{}' && pwd", search)])
                .output()
                .unwrap();
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
