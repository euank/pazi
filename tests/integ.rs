#[cfg(feature = "integ-tests")]
mod harness;
#[cfg(feature = "integ-tests")]
mod integ_tests {
    extern crate pazi;
    use integ_tests::pazi::supported_shells::SUPPORTED_SHELLS;
    use harness::Harness;
    use std::path::PathBuf;
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
        for shell in SUPPORTED_SHELLS.iter() {
            println!("testing: {}", shell);
            it_jumps_shell(shell);
        }
    }

    fn it_jumps_shell(shell: &str) {
        let mut h = Harness::new(&pazi_bin().join("pazi"), shell);

        h.create_dir("/tmp");
        h.visit_dir("/tmp");
        assert_eq!(h.jump("tmp"), "/tmp");
    }

    #[test]
    fn it_jumps_to_more_frecent_items() {
        for shell in SUPPORTED_SHELLS.iter() {
            println!("testing: {}", shell);
            it_jumps_to_more_frecent_items_shell(shell);
        }
    }

    fn it_jumps_to_more_frecent_items_shell(shell: &str) {
        let mut h = Harness::new(&pazi_bin().join("pazi"), shell);

        h.create_dir("/a/tmp");
        h.create_dir("/b/tmp");
        // Visiting 'b' more recently means it shouldbe more frecent.
        h.visit_dir("/a/tmp");
        sleep(Duration::from_millis(5));
        h.visit_dir("/b/tmp");
        assert_eq!(h.jump("tmp"), "/b/tmp");

        // Visiting 'a' more often should make it more 'frecent'
        for _ in 0..10 {
            h.visit_dir("/a/tmp");
            // visit another directory between since 'cd' to the same directory you're in doesn't
            // necessarily increase frecency.
            h.visit_dir("/");
        }
        h.visit_dir("/b/tmp");
        assert_eq!(h.jump("tmp"), "/a/tmp");
    }
}
