#![cfg_attr(feature = "nightly", feature(test))]

extern crate pty;
extern crate tempdir;

mod harness;
mod integ;

#[cfg(feature = "nightly")]
mod bench;
