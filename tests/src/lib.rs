// For benchmarks we need to 'extern crate test' for 'test::Bencher', so we need uniform paths
// still. This can be removed once 'bencher' is stable or once we switch to a different benchmark
// library.
#![cfg_attr(feature = "nightly", feature(test, uniform_paths))]

#[cfg(test)]
mod harness;
#[cfg(test)]
mod integ;
#[cfg(all(feature = "nightly", test))]
mod bench;
