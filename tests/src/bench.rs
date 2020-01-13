extern crate test;

use test::Bencher;
use crate::harness::{
    Autojump, Autojumper, Fasd, Harness, HarnessBuilder, Jump, NoJumper, Pazi, Shell, Z,
};
use std::path::Path;
use tempdir::TempDir;

fn cd_bench(b: &mut Bencher, jumper: &Autojumper, shell: &Shell) {
    let tmpdir = TempDir::new("pazi_bench").unwrap();
    let root = tmpdir.path();
    let mut h = HarnessBuilder::new(&root, jumper, shell)
        .finish();

    // ensure we hit different directories on adjacent iterations; autojumpers may validly avoid
    // doing work on 'cd .'.
    let mut iter = 0;
    let dirs = create_and_visit_dirs(&mut h, &root, "tmp_target", 2);

    b.iter(move || {
        let target = &dirs[iter % 2];
        iter += 1;
        h.visit_dir(&target.path);
    });
}

fn jump_bench(b: &mut Bencher, jumper: &Autojumper, shell: &Shell) {
    let tmpdir = TempDir::new("pazi_bench").unwrap();
    let root = tmpdir.into_path();
    let mut h = HarnessBuilder::new(&root, jumper, shell)
        .finish();

    // ensure we hit different directories on adjacent iterations; some autojumpers (cough `jump`)
    // refuse to jump to cwd
    let mut iter = 0;
    // "prewarm" the directories because it makes fasd less flaky on bash.
    // I suspect it's due to the use of 'history' to populate fasd's database, though I'm not
    // actually certain why it flakes so much without this.
    create_and_visit_dirs(&mut h, &root, "tmp_target", 2);
    let dirs = create_and_visit_dirs(&mut h, &root, "tmp_target", 2);

    b.iter(move || {
        let target = &dirs[iter % 2];
        iter += 1;
        assert_eq!(&h.jump(&target.name), &target.path);
    });
}

fn jump_large_db_bench(b: &mut Bencher, jumper: &Autojumper, shell: &Shell) {
    let tmpdir = TempDir::new("pazi_bench").unwrap();
    let root = tmpdir.path();
    let mut h = HarnessBuilder::new(&root, jumper, shell)
        .finish();

    create_and_visit_dirs(&mut h, &root, "dbnoise", 1000);

    // ensure we hit different directories on adjacent iterations; some autojumpers (cough `jump`)
    // refuse to jump to cwd
    let mut iter = 0;
    // prewarm for fasd, see above.
    create_and_visit_dirs(&mut h, &root, "tmp_target", 2);
    let dirs = create_and_visit_dirs(&mut h, &root, "tmp_target", 2);

    b.iter(move || {
        let target = &dirs[iter % 2];
        iter += 1;
        assert_eq!(&h.jump(&target.name), &target.path);
    });
}

struct JumpTarget {
    path: String,
    name: String,
}

fn create_and_visit_dirs(
    h: &mut Harness,
    root: &Path,
    prefix: &str,
    n: isize,
) -> Vec<JumpTarget> {
    let mut res = Vec::new();
    for i in 0..n {
        let name = format!("{}_{}", prefix, i);
        let path = root.join(&name).to_str().unwrap().to_string();
        h.create_dir(&path);
        h.visit_dir(&path);
        res.push(JumpTarget {
            path: path,
            name: name,
        });
    }
    res
}

// This file is generated with 'build.rs' based on the contents of 'src/benches.csv'; to change the
// contents of it, edit those files.
include!("./benches_generated.rs");
