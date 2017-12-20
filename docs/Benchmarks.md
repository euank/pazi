# Benchmarking pazi

Pazi is benchmarked against other autojump utilities. Currently, it's only
benchmarked against [`fasd`](https://github.com/clvv/fasd/).

The benchmarks are maintained under the 'tests' directory in this repository.

## Benchmark methodology

### Test system

These benchmarks were all run on a Thinkpad X260 with an i7-6600U. Notably, it
also has an SSD. The following versions of relevant software were installed:

| software | version |
|:--------:|:-------:|
| bash     | `GNU bash, version 4.4.12` |
| zsh | `zsh 5.4.1` |
| fasd | commit [90b531a](https://github.com/clvv/fasd/commit/90b531a5daaa545c74c7d98974b54cbdb92659fc) |
| pazi | commit [e64a11f](https://github.com/euank/pazi/commit/e64a11f9df2427d01ce7b91db48f18eaa28175ba) |
| rust | 1.24.0-nightly |

### Procedure

Each benchmark was run once to warm up, and then once to get the value used. Benchmarks were run from the repository via:

```
$ cargo build --release && cd tests
$ cargo bench -j 1 --features nightly <benchmark name>
```

Analysis is based on my understanding of the benchmarks and software under analysis.

## Benchmarks

The below is a list of benchmarks which I think can be meaningfully interpreted.

### cd bench

The `cd_bench` benchmark tries to evaluate the cost of the autojump program
maintaining its frecency list during regular usage.
The benchmark accesses directories, causing the autojump program under test to
add the directory to its history.

This benchmark also includes a benchmark for performing the same operations
with no autojump program installed. This provides a baseline for how slow the
shell+vte+test harness are without any additional autojump accounting.

Without further adieu, the results:

| Autojump Program | Shell  | `cd`/s                 |  ms/`cd`  |
|------------------|--------|------------------------|-----------|
| None             | `bash` |  11831 <!-- ± 169 -->  |  .084     |
| None             | `zsh`  |  31611 <!-- ± 1610 --> |  .031     |
| `fasd`           | `bash` | 52     <!-- ± 1   -->  |  19.275   |
| `fasd`           | `zsh`  | 54     <!-- ± 1   -->  |  18.432   |
| `pazi`           | `bash` | 720    <!-- ± 44 -->   |   1.389   |
| `pazi`           | `zsh`  | 710    <!-- ± 37 -->   |   1.409   |


<!--
test bench::cd_bench_nojumper_bash        ... bench:      84_524 ns/iter (+/- 1_208)
test bench::cd_bench_nojumper_zsh         ... bench:      31_635 ns/iter (+/- 1_611)
test bench::cd_bench_fasd_bash            ... bench:  19_275_454 ns/iter (+/- 455_824)
test bench::cd_bench_fasd_zsh             ... bench:  18_431_650 ns/iter (+/- 439_026)
test bench::cd_bench_pazi_bash            ... bench:   1_389_428 ns/iter (+/- 84_931)
test bench::cd_bench_pazi_zsh             ... bench:   1_408_851 ns/iter (+/- 72_714)
-->

Using an autojump program clearly has a significant impact on a shell's performance.
However, with pazi that impact is clearly minimized.

Using pazi introduces around 1.5 milliseconds of delay between a command completing and a new shell prompt being available. This 1.5 milliseconds will often not be noticible, especially since 60hz displays are only refreshing every 16ms.

Pazi also clearly outperforms fasd, which delays a new shell prompt by around 20ms.

## jump bench

The `jump_bench` benchmark attempts to measure how long it takes from typing `z
<directory>` to having your shell's CWD changed to that directory.

Notably, this benchmark is performed with only a minimal number of entries in
the benchmark program's database, meaning the autojump software need not load
much data from disk.

| Autojump Program | Shell  | jumps/s                |  ms/jump  |
|------------------|--------|------------------------|-----------|
| `fasd`           | `bash` | 25                     |  40.099   |
| `fasd`           | `zsh`  | 25                     |  40.643   |
| `pazi`           | `bash` | 741                    |   1.349   |
| `pazi`           | `zsh`  | 373                    |   2.679   |

<!--
```
test bench::jump_bench_fasd_bash          ... bench:  40_098_873 ns/iter (+/- 866_826)
test bench::jump_bench_fasd_zsh           ... bench:  40_642_529 ns/iter (+/- 881_586)
test bench::jump_bench_pazi_bash          ... bench:   1_348_949 ns/iter (+/- 102_444)
test bench::jump_bench_pazi_zsh           ... bench:   2_679_199 ns/iter (+/- 123_427)
```
-->

## Large database jump bench

The `jump_large_db` benchmark measures the same thing as the previous jump
benchmark, but after filling the database with 1000 entries.

This measures more typical jump performance since the database will usually have quite a few entries.

Pazi still does better than the competition here, but its performance degrades
significantly worse than `fasd`s. This is a clear sign that pazi's use of
msgpack may be non-optimial. Further investigation will be done to improve this
performance in Pazi.

That being said, Pazi is still the fastest by a large margin.


| Autojump Program | Shell  | jumps/s                |  ms/jump  |
|------------------|--------|------------------------|-----------|
| `fasd`           | `bash` | 22                     |  45.129   |
| `fasd`           | `zsh`  | 21                     |  48.572   |
| `pazi`           | `bash` | 49                     |  20.479   |
| `pazi`           | `zsh`  | 40                     |  24.989   |


<!--
```
test bench::jump_large_db_bench_fasd_bash ... bench:  45_128_565 ns/iter (+/- 1_098_832)
test bench::jump_large_db_bench_fasd_zsh  ... bench:  48_571_516 ns/iter (+/- 862_932)
test bench::jump_large_db_bench_pazi_bash ... bench:  20_478_762 ns/iter (+/- 970_543)
test bench::jump_large_db_bench_pazi_zsh  ... bench:  24_988_902 ns/iter (+/- 858_057)
```
-->
