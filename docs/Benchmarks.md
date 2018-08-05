# Benchmarking pazi

Pazi is benchmarked against other autojump utilities. Currently, it's
benchmarked against [`fasd`](https://github.com/clvv/fasd/), [`autojump`](https://github.com/wting/autojump), and [`z`](https://github.com/rupa/z).

The benchmarks are maintained under the 'tests' directory in this repository.

## Benchmark methodology

### Test system

These benchmarks were all run on a machine with an i5-4670K and a Samsung 850 EVO SSD.
The following versions of relevant software were installed:

| software | version |
|:--------:|:-------:|
| bash     | `GNU bash, version 4.4.12` |
| zsh | `zsh 5.5.1` |
| autojump | commit [6a529f4](https://github.com/wting/autojump/commit/6a529f4f929042a37e7394d8c91ba31b479189a2) |
| fasd | commit [90b531a](https://github.com/clvv/fasd/commit/90b531a5daaa545c74c7d98974b54cbdb92659fc) |
| pazi | commit [e56d571](https://github.com/euank/pazi/commit/e56d5713d8f492cccde46cdbeba9e9fbf2d2ed2c) |
| z | commit [ea5ec78](https://github.com/rupa/z/commit/ea5ec7834398ee2244de26953b3d1ef785d3f942) |
| rust | 1.29.0-nightly |

### Procedure

Each benchmark was run once to warm up, and then once to get the value used. Benchmarks were run from the repository via:

```
$ cargo build --release && cd tests
$ cargo bench --features=nightly,cgroups2 <benchmark name>
```

Analysis is based on my understanding of the benchmarks and software under analysis.

## Benchmarks

The below is a list of benchmarks which I think can be meaningfully interpreted.

### cd bench

The `cd_bench_normal` benchmark tries to evaluate the cost of the autojump program
maintaining its frecency list during regular usage.
The benchmark accesses directories, causing the autojump program under test to
add the directory to its history.

This benchmark also includes a benchmark for performing the same operations
with no autojump program installed. This provides a baseline for how slow the
shell+vte+test harness are without any additional autojump accounting.

Without further adieu, the results:

| Autojump Program | Shell  | `cd`/s   |  ms/`cd`  |
|------------------|--------|----------|-----------|
| None             | `bash` | 10870    |  .092     |
| None             | `zsh`  | 32258    |  .031     |
| `autojump`       | `bash` | 60       |  16.771   |
| `autojump`       | `zsh`  | 56       |  17.765   |
| `fasd`           | `bash` | 66       |  15.218   |
| `fasd`           | `zsh`  | 73       |  13.770   |
| `pazi`           | `bash` | 599      |   1.669   |
| `pazi`           | `zsh`  | 702      |   1.423   |
| `z`              | `bash` | 754      |   1.327   |


<!--
test bench::cd_bench_normal_autojump_bash     ... bench:  16,771,453 ns/iter (+/- 9,867,754)
test bench::cd_bench_normal_autojump_zsh      ... bench:  17,764,760 ns/iter (+/- 7,924,858)
test bench::cd_bench_normal_fasd_bash         ... bench:  15,218,044 ns/iter (+/- 553,219)
test bench::cd_bench_normal_fasd_zsh          ... bench:  13,769,654 ns/iter (+/- 542,622)
test bench::cd_bench_normal_nojumper_bash     ... bench:      92,764 ns/iter (+/- 4,920)
test bench::cd_bench_normal_nojumper_zsh      ... bench:      31,373 ns/iter (+/- 3,654)
test bench::cd_bench_normal_pazi_bash         ... bench:   1,669,345 ns/iter (+/- 215,517)
test bench::cd_bench_normal_pazi_zsh          ... bench:   1,422,955 ns/iter (+/- 150,071)
test bench::cd_bench_normal_z_bash            ... bench:   1,327,210 ns/iter (+/- 281,970)
-->

Using an autojump program clearly has a significant impact on a shell's performance.
However, with pazi that impact is clearly very small.

Using pazi introduces around 1.5 milliseconds of delay between a command completing and a new shell prompt being available. This 1.5 milliseconds will often not be noticible, especially since 60hz displays are only refreshing every 16ms.

The fastest autojumper in this specific benchmark is `z`, which is
neck-and-neck with pazi. Unfortunately, under `zsh`, `z` exhibits a serious bug
often enough that it cannot be benchmarked in this method.

This test does not show all of the differences between the impact these programs have though.
In addition to the visible "time to next command prompt" this benchmark shows, there's the "time to directory processed". This time is the same for pazi and fasd.
However, `autojump` and `z` both fork work into the background during `cd`s.
Even if a new shell prompt is visible in a millisecond with `z`, it continues
to consume resources for a short while more.

The next benchmark tries to measure that difference as well.

## cd sync bench

The `cd_bench` benchmark above, simply measures how long it takes a new shell prompt to be available for use after running a `cd` command.
This does not capture any work an autojumper might do in the background.
The `cd_sync` benchmark is meant to measure that as well. It operates by
running `cd`, and then waiting for a shell to be visible *and* for no
additional background processes to be running under the shell.

This captures how long it takes autojumpers that fork into the background to quit using resources.

The results for this benchmark are:

| Autojump Program | Shell  | `cd`/s   |  ms/`cd`  |
|------------------|--------|----------|-----------|
| None             | `bash` | 10000    |  .100     |
| None             | `zsh`  | 32258    |  .031     |
| `autojump`       | `bash` | 19       |  52.293   |
| `autojump`       | `zsh`  | 19       |  52.123   |
| `fasd`           | `bash` | 66       |  15.237   |
| `fasd`           | `zsh`  | 73       |  13.728   |
| `pazi`           | `bash` | 644      |   1.552   |
| `pazi`           | `zsh`  | 733      |   1.364   |
| `z`              | `bash` | 262      |   3.813   |
| `z`              | `zsh`  | 319      |   3.133   |


<!--
test bench::cd_bench_sync_autojump_bash       ... bench:  52,292,682 ns/iter (+/- 1,024,399)
test bench::cd_bench_sync_autojump_zsh        ... bench:  52,123,162 ns/iter (+/- 1,738,625)
test bench::cd_bench_sync_fasd_bash           ... bench:  15,237,464 ns/iter (+/- 369,652)
test bench::cd_bench_sync_fasd_zsh            ... bench:  13,728,382 ns/iter (+/- 326,905)
test bench::cd_bench_sync_nojumper_bash       ... bench:     100,451 ns/iter (+/- 3,779)
test bench::cd_bench_sync_nojumper_zsh        ... bench:      30,548 ns/iter (+/- 1,674)
test bench::cd_bench_sync_pazi_bash           ... bench:   1,551,545 ns/iter (+/- 168,553)
test bench::cd_bench_sync_pazi_zsh            ... bench:   1,363,778 ns/iter (+/- 102,843)
test bench::cd_bench_sync_z_bash              ... bench:   3,812,556 ns/iter (+/- 267,864)
test bench::cd_bench_sync_z_zsh               ... bench:   3,132,578 ns/iter (+/- 138,502)
-->

For this benchmark, we can run `z` under both `bash` and `zsh` since the bugs
which prevented  it from working in the previous cd benchmark happen to be
caused by multiple processes forking and modifying its data file in parallel.

This benchmark shows that, when taking into account processes forking into the background, pazi is clearly the fastest. Vroom vroom.

## jump bench

The `jump_bench` benchmark attempts to measure how long it takes from typing `z
<directory>` to having your shell's CWD changed to that directory.

Notably, this benchmark is performed with only a minimal number of entries in
the benchmark program's database, meaning the autojump software need not load
much data from disk.

| Autojump Program | Shell  | jumps/s                |  ms/jump  |
|------------------|--------|------------------------|-----------|
| `autojump`       | `bash` | 18                     |  55.875   |
| `autojump`       | `zsh`  | 18                     |  56.440   |
| `fasd`           | `bash` | 32                     |  31.463   |
| `fasd`           | `zsh`  | 33                     |  29.888   |
| `pazi`           | `bash` | 502                    |   1.993   |
| `pazi`           | `zsh`  | 348                    |   2.873   |
| `z`              | `bash` | 242                    |   4.126   |

<!--
test bench::jump_bench_autojump_bash          ... bench:  55,874,896 ns/iter (+/- 2,322,104)
test bench::jump_bench_autojump_zsh           ... bench:  56,440,482 ns/iter (+/- 1,956,512)
test bench::jump_bench_fasd_bash              ... bench:  31,462,861 ns/iter (+/- 739,573)
test bench::jump_bench_fasd_zsh               ... bench:  29,888,266 ns/iter (+/- 396,160)
test bench::jump_bench_pazi_bash              ... bench:   1,992,635 ns/iter (+/- 149,604)
test bench::jump_bench_pazi_zsh               ... bench:   2,872,545 ns/iter (+/- 334,004)
test bench::jump_bench_z_bash                 ... bench:   4,125,595 ns/iter (+/- 428,407)
-->

We can see that pazi is twice as quick as the alternatives, though again we can't measure `z` on `zsh` due to bugs.

Both `autojump` and `fasd` end up over 10 times slower.
I suspect autojump's slowness her, and elsewhere, is in no small part due  to loading the python interpreter.
fasd's speed, I suspect, is limited by its `awk` program used to parse its
database, though I have not done in depth profiling to be sure.

## Large database jump bench

The `jump_large_db` benchmark measures the same thing as the previous jump
benchmark, but after filling the database with 1000 entries.

This measures more typical jump performance since the database will usually have quite a few entries.

| Autojump Program | Shell  | jumps/s                |  ms/jump  |
|------------------|--------|------------------------|-----------|
| `autojump`       | `bash` | 22                     |  95.099   |
| `autojump`       | `zsh`  | 21                     |  94.594   |
| `fasd`           | `bash` | 49                     |  35.512   |
| `fasd`           | `zsh`  | 40                     |  34.214   |
| `pazi`           | `bash` | 49                     |  17.763   |
| `pazi`           | `zsh`  | 40                     |  22.021   |
| `z`              | `bash` | 40                     |  17.811   |


<!--
test bench::jump_large_db_bench_autojump_bash ... bench:  95,099,457 ns/iter (+/- 2,912,535)
test bench::jump_large_db_bench_autojump_zsh  ... bench:  94,594,256 ns/iter (+/- 2,973,848)
test bench::jump_large_db_bench_fasd_bash     ... bench:  35,512,474 ns/iter (+/- 1,097,605)
test bench::jump_large_db_bench_fasd_zsh      ... bench:  34,214,217 ns/iter (+/- 645,826)
test bench::jump_large_db_bench_pazi_bash     ... bench:  17,762,955 ns/iter (+/- 538,958)
test bench::jump_large_db_bench_pazi_zsh      ... bench:  22,021,296 ns/iter (+/- 343,753)
test bench::jump_large_db_bench_z_bash        ... bench:  17,811,464 ns/iter (+/- 1,512,789)
-->

Pazi does better than the competition here, but its performance degrades
significantly worse than `fasd`s. This is a clear sign that pazi's use of
msgpack may be non-optimial. Further investigation will be done to improve this
performance in Pazi.

Pazi is also  close enough to `z` we might as well call that comparison a tie.
