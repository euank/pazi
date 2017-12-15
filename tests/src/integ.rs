extern crate pazi;
extern crate tempdir;

use integ::pazi::supported_shells::SUPPORTED_SHELLS;
use tempdir::TempDir;
use harness::Harness;
use std::path::{Path, PathBuf};
use std::env;
use std::time::Duration;
use std::thread::sleep;

fn pazi_bin() -> PathBuf {
    // target/.../deps/integ...
    let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("build with cargo");
    let pazi = Path::new(&crate_dir)
        .join("../target/release/pazi")
        .canonicalize()
        .unwrap();
    if !pazi.exists() {
        panic!("compile pazi in release mode before running bench/integ tests");
    }
    println!("pazi path: {}", pazi.to_string_lossy());
    pazi
}

#[test]
fn it_jumps() {
    for shell in SUPPORTED_SHELLS.iter() {
        println!("testing: {}", shell);
        it_jumps_shell(shell);
    }
}

fn it_jumps_shell(shell: &str) {
    let root = TempDir::new("pazi_integ").unwrap().into_path();
    let mut h = Harness::new(&root, &pazi_bin(), shell);
    let slash_tmp_path = root.join("tmp");
    let slash_tmp = slash_tmp_path.to_string_lossy();

    h.create_dir(&slash_tmp);
    h.visit_dir(&slash_tmp);
    assert_eq!(h.jump("tmp"), slash_tmp);
}

#[test]
fn it_jumps_to_more_frecent_items() {
    for shell in SUPPORTED_SHELLS.iter() {
        println!("testing: {}", shell);
        it_jumps_to_more_frecent_items_shell(shell);
    }
}

fn it_jumps_to_more_frecent_items_shell(shell: &str) {
    let root = TempDir::new("pazi_integ").unwrap().into_path();
    let mut h = Harness::new(&root, &pazi_bin(), shell);
    let a_dir_path = root.join("a/tmp");
    let b_dir_path = root.join("b/tmp");
    let a_dir = a_dir_path.to_string_lossy();
    let b_dir = b_dir_path.to_string_lossy();

    h.create_dir(&a_dir);
    h.create_dir(&b_dir);
    // Visiting 'b' more recently means it shouldbe more frecent.
    h.visit_dir(&a_dir);
    sleep(Duration::from_millis(5));
    h.visit_dir(&b_dir);
    assert_eq!(h.jump("tmp"), b_dir);

    // Visiting 'a' more often should make it more 'frecent'
    for _ in 0..10 {
        h.visit_dir(&a_dir);
        // visit another directory between since 'cd' to the same directory you're in doesn't
        // necessarily increase frecency.
        h.visit_dir(&root.to_string_lossy());
    }
    h.visit_dir(&b_dir);
    assert_eq!(h.jump("tmp"), a_dir);
}
