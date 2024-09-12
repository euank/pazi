#![cfg_attr(feature = "nightly", feature(test))]

#[cfg(all(feature = "nightly", test))]
mod bench;
#[cfg(test)]
mod harness;
#[cfg(test)]
mod integ;
