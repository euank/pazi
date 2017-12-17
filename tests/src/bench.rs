extern crate test;
extern crate tempdir;

use tempdir::TempDir;
use harness::{Harness, Autojumper, Shell, Pazi, Fasd, NoJumper};
use self::test::Bencher;

fn cd_bench(b: &mut Bencher, jumper: &Autojumper, shell: &Shell) {
    let tmpdir = TempDir::new("pazi_bench").unwrap();
    let root = tmpdir.path();
    let mut h = Harness::new(&root, jumper, shell);
    let dir1p = root.join("tmp1");
    let dir2p = root.join("tmp2");
    let dir1 = dir1p.to_str().unwrap();
    let dir2 = dir1p.to_str().unwrap();

    h.create_dir(&dir1);
    h.create_dir(&dir2);

    // ensure we hit different directories on adjacent iterations; autojumpers may validly avoid
    // doing work on 'cd .'.
    let mut iter = 0;
    b.iter(move || {
        let dir = if iter % 2 == 0 {
            &dir1
        } else {
            &dir2
        };
        iter += 1;
        h.visit_dir(dir)
    });
}


fn cd_50_bench(b: &mut Bencher, jumper: &Autojumper, shell: &Shell) {
    let tmpdir = TempDir::new("pazi_bench").unwrap();
    let root = tmpdir.path();
    let mut h = Harness::new(&root, jumper, shell);
    let dirs = (0..50).map(|num| {
        format!("{}/dir{}", root.to_str().unwrap(), num)
    }).collect::<Vec<_>>();
    for dir in &dirs {
        h.create_dir(&dir);
    }

    let cmd = dirs.iter().map(|el| {
        format!("cd '{}'", el)
    }).collect::<Vec<_>>().join(" && ");

    b.iter(move || {
        h.run_cmd(&cmd)
    });
}


#[bench]
fn it_cds_to_50_directories_none_zsh(b: &mut Bencher) {
    cd_50_bench(b, &NoJumper, &Shell::Zsh);
}

#[bench]
fn it_cds_to_50_directories_none_bash(b: &mut Bencher) {
    cd_50_bench(b, &NoJumper, &Shell::Bash);
}

#[bench]
fn it_cds_to_50_directories_pazi_zsh(b: &mut Bencher) {
    cd_50_bench(b, &Pazi, &Shell::Zsh);
}

#[bench]
fn it_cds_to_50_directories_pazi_bash(b: &mut Bencher) {
    cd_50_bench(b, &Pazi, &Shell::Bash);
}


#[bench]
fn it_cds_to_50_directories_fasd_zsh(b: &mut Bencher) {
    cd_50_bench(b, &Fasd, &Shell::Zsh);
}

#[bench]
fn it_cds_to_50_directories_fasd_bash(b: &mut Bencher) {
    cd_50_bench(b, &Fasd, &Shell::Bash);
}

#[bench]
fn it_cds_to_2_directories_none_zsh(b: &mut Bencher) {
    cd_bench(b, &NoJumper, &Shell::Zsh);
}

#[bench]
fn it_cds_to_2_directories_none_bash(b: &mut Bencher) {
    cd_bench(b, &NoJumper, &Shell::Bash);
}

#[bench]
fn it_cds_to_2_directories_pazi_zsh(b: &mut Bencher) {
    cd_bench(b, &Pazi, &Shell::Zsh);
}

#[bench]
fn it_cds_to_2_directories_pazi_bash(b: &mut Bencher) {
    cd_bench(b, &Pazi, &Shell::Bash);
}


#[bench]
fn it_cds_to_2_directories_fasd_zsh(b: &mut Bencher) {
    cd_bench(b, &Fasd, &Shell::Zsh);
}

#[bench]
fn it_cds_to_2_directories_fasd_bash(b: &mut Bencher) {
    cd_bench(b, &Fasd, &Shell::Bash);
}
