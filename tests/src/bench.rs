extern crate tempdir;
extern crate test;

use tempdir::TempDir;
use harness::{Autojumper, Fasd, HarnessBuilder, NoJumper, Pazi, Shell, Autojump};
use self::test::Bencher;

fn cd_bench(b: &mut Bencher, jumper: &Autojumper, shell: &Shell) {
    let tmpdir = TempDir::new("pazi_bench").unwrap();
    let root = tmpdir.path();
    let mut h = HarnessBuilder::new(&root, jumper, shell).finish();
    let dir1p = root.join("tmp1");
    let dir2p = root.join("tmp2");
    let dir1 = dir1p.to_str().unwrap();
    let dir2 = dir2p.to_str().unwrap();

    h.create_dir(&dir1);
    h.create_dir(&dir2);

    // ensure we hit different directories on adjacent iterations; autojumpers may validly avoid
    // doing work on 'cd .'.
    let mut iter = 0;

    b.iter(move || {
        let dir = if iter % 2 == 0 { &dir1 } else { &dir2 };
        iter += 1;
        h.visit_dir(dir)
    });
}

fn cd_bench_sync(b: &mut Bencher, jumper: &Autojumper, shell: &Shell) {
    let tmpdir = TempDir::new("pazi_bench").unwrap();
    let root = tmpdir.path();
    let mut h = HarnessBuilder::new(&root, jumper, shell).cgroup(true).finish();
    let dir1p = root.join("tmp1");
    let dir2p = root.join("tmp2");
    let dir1 = dir1p.to_str().unwrap();
    let dir2 = dir2p.to_str().unwrap();

    h.create_dir(&dir1);
    h.create_dir(&dir2);

    // ensure we hit different directories on adjacent iterations; autojumpers may validly avoid
    // doing work on 'cd .'.
    let mut iter = 0;

    b.iter(move || {
        let dir = if iter % 2 == 0 { &dir1 } else { &dir2 };
        iter += 1;
        h.visit_dir(dir);
        h.wait_children();
        true
    });
}

fn jump_bench(b: &mut Bencher, jumper: &Autojumper, shell: &Shell) {
    let tmpdir = TempDir::new("pazi_bench").unwrap();
    let root = tmpdir.path();
    let mut h = HarnessBuilder::new(&root, jumper, shell).cgroup(true).finish();
    let dir1p = root.join("tmp1");
    let dir1 = dir1p.to_str().unwrap();

    h.create_dir(&dir1);
    h.visit_dir(&dir1);
    h.wait_children();

    b.iter(move || {
        assert_eq!(&h.jump("tmp1"), dir1);
    });
}

fn jump_large_db_bench(b: &mut Bencher, jumper: &Autojumper, shell: &Shell) {
    let tmpdir = TempDir::new("pazi_bench").unwrap();
    let root = tmpdir.path();
    let mut h = HarnessBuilder::new(&root, jumper, shell).finish();
    let dirp = root.join("tmp_target");
    let dir = dirp.to_str().unwrap();

    // Add about 1000 items to the db
    for i in 1..1000 {
        let dirn = root.join(format!("tmp{}", i));
        h.create_dir(&dirn.to_string_lossy());
        h.visit_dir(&dirn.to_string_lossy());
    }

    h.create_dir(&dir);
    h.visit_dir(&dir);

    b.iter(move || {
        assert_eq!(&h.jump("tmp_target"), &dir);
    });
}

fn cd_50_bench(b: &mut Bencher, jumper: &Autojumper, shell: &Shell) {
    let tmpdir = TempDir::new("pazi_bench").unwrap();
    let root = tmpdir.path();
    let mut h = HarnessBuilder::new(&root, jumper, shell).finish();
    let dirs = (0..50)
        .map(|num| format!("{}/dir{}", root.to_str().unwrap(), num))
        .collect::<Vec<_>>();
    for dir in &dirs {
        h.create_dir(&dir);
    }

    let cmd = dirs.iter()
        .map(|el| format!("cd '{}'", el))
        .collect::<Vec<_>>()
        .join(" && ");

    b.iter(move || h.run_cmd(&cmd));
}

// This file is generated with 'build.rs' based on the contents of 'src/benches.csv'; to change the
// contents of it, edit those files.
include!("./benches_generated.rs");
