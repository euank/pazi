use super::Autojumper;
use harness::Shell;
use std::env;
use std::process::Command;
use std::path::Path;
use std::fs;
use std::os::unix::fs::PermissionsExt;

pub struct Fasd;

const FASD_VERSION: &str = "90b531a";

impl Autojumper for Fasd {
    fn bin_path(&self) -> String {
        let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("build with cargo");
        let fasd_path = Path::new(&crate_dir)
            .join(format!(".cache/fasd/{}/fasd", FASD_VERSION));

        if fasd_path.exists() {
            return fasd_path.canonicalize().unwrap().to_string_lossy().to_string();
        }
        // doesn't exist, let's create it..
        let dir = fasd_path.parent().expect("could not get parent");
        fs::create_dir_all(dir).expect("could not create dir");
        println!("2");
        let url = format!("https://raw.githubusercontent.com/clvv/fasd/{}/fasd", FASD_VERSION);
        Command::new("curl")
            .args(vec!["-s", "-o", &fasd_path.to_string_lossy(), &url])
            .status().unwrap();

        fs::set_permissions(&fasd_path, fs::Permissions::from_mode(0o755)).unwrap();

        fasd_path.canonicalize().unwrap().to_string_lossy().to_string()
    }

    fn init_for(&self, shell: &Shell) -> String {
        match shell {
            &Shell::Bash | &Shell::Zsh => {
                r#"eval "$(fasd --init auto)""#.to_string()
            }
            &Shell::Conch => unimplemented!(),
        }
    }
}
