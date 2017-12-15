use harness::shells::Shell;
use std::path::Path;
use std::env;

pub enum Autojumper {
    None,
    Pazi,
}

impl Autojumper {
    pub fn bin_path(&self) -> String {
        match *self {
            Autojumper::None => "".to_owned(),
            Autojumper::Pazi => {
                let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("build with cargo");
                let pazi = Path::new(&crate_dir)
                    .join("../target/release/pazi")
                    .canonicalize()
                    .unwrap();
                if !pazi.exists() {
                    panic!("compile pazi in release mode before running bench/integ tests");
                }
                println!("pazi path: {}", pazi.to_string_lossy());
                pazi.to_str().unwrap().to_string()
            }
        }
    }
    pub fn init_for(&self, shell: &Shell) -> String {
        match (self, shell) {
            (&Autojumper::Pazi, &Shell::Bash) | (&Autojumper::Pazi, &Shell::Zsh) => {
                format!(r#"eval "$(pazi init {})""#, shell.name())
            }
            (&Autojumper::None, _) => "".to_string(),
            (_, &Shell::Conch) => unimplemented!(),
        }
    }
}

