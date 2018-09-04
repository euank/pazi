extern crate tempdir;
extern crate test;

use tempdir::TempDir;
use harness::{Autojumper, Fasd, Z, HarnessBuilder, NoJumper, Pazi, Shell, Autojump};
use self::test::Bencher;

fn cd_bench(b: &mut Bencher, jumper: &Autojumper, shell: &Shell, sync: bool) {
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
        h.visit_dir(dir);
        if sync {
            h.wait_children()
        }
    });
}

fn jump_bench(b: &mut Bencher, jumper: &Autojumper, shell: &Shell, sync: bool) {
    let tmpdir = TempDir::new("pazi_bench").unwrap();
    let root = tmpdir.path();
    let mut h = HarnessBuilder::new(&root, jumper, shell).cgroup(sync).finish();
    let dir1p = root.join("tmp1");
    let dir1 = dir1p.to_str().unwrap();

    h.create_dir(&dir1);
    h.visit_dir(&dir1);
    if sync {
        h.wait_children();
    }

    b.iter(move || {
        assert_eq!(&h.jump("tmp1"), dir1);
    });
}

fn jump_large_db_bench(b: &mut Bencher, jumper: &Autojumper, shell: &Shell, sync: bool) {
    let tmpdir = TempDir::new("pazi_bench").unwrap();
    let root = tmpdir.path();
    let mut h = HarnessBuilder::new(&root, jumper, shell).cgroup(sync).finish();
    let dirp = root.join("tmp_target");
    let dir = dirp.to_str().unwrap();

    // Add about 1000 items to the db
    for i in 1..1000 {
        let dirn = root.join(format!("tmp{}", i));
        h.create_dir(&dirn.to_string_lossy());
        h.visit_dir(&dirn.to_string_lossy());
        if sync {
            h.wait_children();
        }
    }

    h.create_dir(&dir);
    h.visit_dir(&dir);
    if sync {
        h.wait_children();
    }

    b.iter(move || {
        assert_eq!(&h.jump("tmp_target"), &dir);
    });
}

// This file is generated with 'build.rs' based on the contents of 'src/benches.csv'; to change the
// contents of it, edit those files.
include!("./benches_generated.rs");
