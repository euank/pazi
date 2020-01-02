#![cfg_attr(feature = "nightly", feature(test))]

#[cfg(test)]
mod harness;
#[cfg(test)]
mod integ;
#[cfg(all(feature = "nightly", test))]
mod bench;
