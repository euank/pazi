# pazi tests and benchmarks

This directorty contains integration tests and benchmarks for pazi.

Since it contains benchmarks, it uses the `release` target of pazi found in the
parent directory when running.

It contains autojumpers that are benchmarked against as git submodules in the `testbins` subdirectory.

## Running benchmarks

Running pazi's benchmarks is somewhat involved, mostly because it's being benchmarked against several other pieces of software with varying requirements.

It depends on your system's version of the following:

* bash -- Used for all benchmarks, minimum required version unknown.
* zsh -- Used for all benchmarks, minimum required version unknown.
* go -- Used for `jump`, version >=1.11.
* python -- Used for `autojump`, version 2.7 or 3.x.
* cgroups -- Used for "sync" benchmarks for z and autojump, hybrid or unified cgroupsv2 must be mounted.
* root -- Used for "sync" benchmarks.
* rust -- Used for everything, nightly needed for benchmark support.

Once you've got all that sorted out, running `make bench` or `make bench-all` should probably work.
