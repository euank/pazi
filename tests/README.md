# pazi tests and benchmarks

This directorty contains integration tests and benchmarks for pazi.

Since it contains benchmarks, it uses the `release` target of pazi found in the
parent directory when running.

Autojumpers that are benchmarked against are all modeled with nix, as are the shell versions used for tests and benchmarks.

## Running benchmarks

Running pazi's benchmarks is somewhat involved. In order to control the versions of software involved (namely other autojumpers and shells), we use [nix](https://nixos.org/download.html). The [flakes](https://nixos.wiki/wiki/Flakes) feature must be enabled as well.

We also use the host rust (rather than a nix controlled one) since it's assumed you have rust handy anyway if you're working in pazi's codebase.

Finally, we depend on using [`cgroupsv2`](https://www.kernel.org/doc/Documentation/cgroup-v2.txt) to properly track children (for autojumpers that fork child processes), so your kernel must have cgroupsv2 enabled.
This additionally requires root access since we create cgroups to spawn benchmark shells in, which we do by executing sudo.

After getting the above sorted out, run `make bench`.
