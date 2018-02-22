extern crate pazi;
extern crate tempdir;

use integ::pazi::shells::SUPPORTED_SHELLS;
use tempdir::TempDir;
use harness::{Fasd, Harness, Pazi, Shell};
use std::time::Duration;
use std::thread::sleep;

#[test]
fn it_jumps() {
    for shell in SUPPORTED_SHELLS.iter() {
        println!("testing: {}", shell);
        let s = Shell::from_str(shell);
        it_jumps_shell(&s);
    }
}

fn it_jumps_shell(shell: &Shell) {
    let tmpdir = TempDir::new("pazi_integ").unwrap();
    let root = tmpdir.path();
    let mut h = Harness::new(&root, &Pazi, shell);
    let slash_tmp_path = root.join("tmp");
    let slash_tmp = slash_tmp_path.to_string_lossy();

    h.create_dir(&slash_tmp);
    h.visit_dir(&slash_tmp);
    assert_eq!(h.jump("tmp"), slash_tmp);
}

#[test]
fn it_jumps_to_exact_directory() {
    for shell in SUPPORTED_SHELLS.iter() {
        println!("testing: {}", shell);
        let s = Shell::from_str(shell);
        it_jumps_to_exact_directory_shell(&s);
    }
}

fn it_jumps_to_exact_directory_shell(shell: &Shell) {
    let tmpdir = TempDir::new("pazi_integ").unwrap();
    let root = tmpdir.path();
    let mut h = Harness::new(&root, &Pazi, shell);
    let slash_tmp_path = root.join("tmp");
    let slash_tmp = slash_tmp_path.to_string_lossy();
    let unvisited_dir_path = slash_tmp_path.join("asdf");
    let unvisited_dir = unvisited_dir_path.to_string_lossy();

    h.create_dir(&unvisited_dir);
    h.visit_dir(&slash_tmp);
    assert_eq!(h.jump("asdf"), unvisited_dir);
}

#[test]
fn it_jumps_to_more_frecent_items() {
    for shell in SUPPORTED_SHELLS.iter() {
        println!("testing: {}", shell);
        let s = Shell::from_str(shell);
        it_jumps_to_more_frecent_items_shell(&s);
    }
}

fn it_jumps_to_more_frecent_items_shell(shell: &Shell) {
    let tmpdir = TempDir::new("pazi_integ").unwrap();
    let root = tmpdir.path();
    let mut h = Harness::new(&root, &Pazi, shell);
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

#[test]
fn it_imports_from_fasd() {
    for shell in SUPPORTED_SHELLS.iter() {
        println!("testing: {}", shell);
        let s = Shell::from_str(shell);
        it_imports_from_fasd_shell(&s);
    }
}

fn it_imports_from_fasd_shell(shell: &Shell) {
    let tmpdir = TempDir::new("pazi_integ").unwrap();
    let root = tmpdir.path();

    {
        let mut fasd = Harness::new(&root, &Fasd, shell);
        fasd.create_dir(&root.join("tmp").to_string_lossy());
        // visit twice because fasd uses 'history 1' to do stuff in bash... which means yeah, it's
        // 1-command-delayed
        fasd.visit_dir(&root.join("tmp").to_string_lossy());
        fasd.visit_dir(&root.join("tmp").to_string_lossy());
    }

    {
        let mut h = Harness::new(&root, &Pazi, shell);
        assert_eq!(
            h.run_cmd("pazi import fasd").trim(),
            "imported 1 items from fasd (out of 1 in its db)"
        );
        assert_eq!(h.jump("tmp"), root.join("tmp").to_string_lossy());
    }
}

#[test]
fn it_ignores_dead_dirs_on_cd() {
    for shell in SUPPORTED_SHELLS.iter() {
        println!("testing: {}", shell);
        let s = Shell::from_str(shell);
        it_ignores_dead_dirs_on_cd_shell(&s);
    }
}

fn it_ignores_dead_dirs_on_cd_shell(shell: &Shell) {
    let tmpdir = TempDir::new("pazi_integ").unwrap();
    let root = tmpdir.path();
    let mut h = Harness::new(&root, &Pazi, shell);

    h.create_dir(&root.join("1/tmp").to_string_lossy());
    h.create_dir(&root.join("2/tmp").to_string_lossy());

    h.visit_dir(&root.join("1/tmp").to_string_lossy());
    h.visit_dir(&root.join("2/tmp").to_string_lossy());
    h.visit_dir(&root.join("2/tmp").to_string_lossy());
    h.visit_dir(&root.join("2/tmp").to_string_lossy());

    assert_eq!(h.jump("tmp"), root.join("2/tmp").to_string_lossy());
    h.delete_dir(&root.join("2/tmp").to_string_lossy());
    assert_eq!(h.jump("tmp"), root.join("1/tmp").to_string_lossy());
}

#[test]
fn it_prints_list_on_lonely_z() {
    // running just 'z' or just 'pazi' should print a directory listing, not error
    for shell in SUPPORTED_SHELLS.iter() {
        let s = Shell::from_str(shell);
        it_prints_list_on_lonely_z_shell(&s);
    }
}

fn it_prints_list_on_lonely_z_shell(shell: &Shell) {
    let tmpdir = TempDir::new("pazi_integ").unwrap();
    let root = tmpdir.path();
    let mut h = Harness::new(&root, &Pazi, shell);

    h.create_dir(&root.join("1/tmp").to_string_lossy());
    h.create_dir(&root.join("2/tmp").to_string_lossy());
    h.visit_dir(&root.join("1/tmp").to_string_lossy());
    h.visit_dir(&root.join("2/tmp").to_string_lossy());

    let z_res = h.run_cmd("z");
    let pazi_res = h.run_cmd("pazi");

    assert_eq!(z_res, pazi_res);
    assert!(z_res.contains(&root.join("1/tmp").to_string_lossy().to_string()));
}

// Regression test for https://github.com/euank/pazi/issues/41
#[test]
fn it_handles_existing_bash_prompt_command() {
    let tmpdir = TempDir::new("pazi_integ").unwrap();
    let root = tmpdir.path();
    let prompt_cmd = r#"
PROMPT_COMMAND='printf "\033k%s@%s:%s\033\\" "${USER}" "${HOSTNAME%%.*}" "${PWD/#$HOME/\~}"'
"#;
    let mut h = Harness::new_with_preinit(&root, &Pazi, &Shell::Bash, prompt_cmd);
    let slash_tmp_path = root.join("tmp");
    let slash_tmp = slash_tmp_path.to_string_lossy();

    h.create_dir(&slash_tmp);
    h.visit_dir(&slash_tmp);
    assert_eq!(h.jump("tmp"), slash_tmp);
}

// Test for https://github.com/euank/pazi/issues/49
#[test]
fn it_handles_help_output() {
    for shell in SUPPORTED_SHELLS.iter() {
        let s = Shell::from_str(shell);
        it_handles_help_output_shell(&s);
    }
}

fn it_handles_help_output_shell(shell: &Shell) {
    let tmpdir = TempDir::new("pazi_integ").unwrap();
    let root = tmpdir.path();
    let mut h = Harness::new(&root, &Pazi, shell);
    let help1 = h.run_cmd("pazi --help && echo $?");
    let help2 = h.run_cmd("z -h && echo $?");
    let help3 = h.run_cmd("z --help && echo $?");
    assert_eq!(help1, help2);
    assert_eq!(help2, help3);
    assert!(help1.ends_with("\n0"));
}

// Test for https://github.com/euank/pazi/issues/60
#[test]
fn it_handles_things_that_look_sorta_like_init_but_not_really() {
    for shell in SUPPORTED_SHELLS.iter() {
        let s = Shell::from_str(shell);
        it_handles_things_that_look_sorta_etc_shell(&s);
    }
}

fn it_handles_things_that_look_sorta_etc_shell(shell: &Shell) {
    let tmpdir = TempDir::new("pazi_integ").unwrap();
    let root = tmpdir.path();
    let mut h = Harness::new(&root, &Pazi, shell);
    let igni = root.join("ignition").into_os_string().into_string().unwrap();

    h.create_dir(&igni);
    h.visit_dir(&igni);
    h.visit_dir(&root.to_string_lossy());
    assert_eq!(h.jump("igni"), igni);
}
