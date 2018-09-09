# :zap: pazi :zap: &mdash; A fast autojump helper

## What is pazi?

Pazi is an autojump utility. That is to say, pazi remembers visited directories
in the past and makes it easier to get back to them. A typical use of pazi
might look like the following:

```sh
user@host ~ $ cd go/src/k8s.io/kubernetes
user@host ~/go/src/k8s.io/kubernetes $ cd /usr/src/linux

# The primary way to interact with pazi is via the 'z', zap-to-directory, alias
user@host /usr/src/linux $ z kuber
user@host ~/go/src/k8s.io/kubernetes $ # pazi zapped to the best match for 'kuber' that it remembers having been in
user@host ~/go/src/k8s.io/kubernetes $ z linux
user@host /usr/src/linux $

# If multiple items match a query, they can be interactively chosen between with '-i':
user@host /usr/src/linux $ cd ~/dev/linux
user@host ~/dev/linux $ z -i linux
2	0.7200000000000001	/usr/src/linux
1	0.9200000000000002	/home/user/dev/linux
> 1

user@host ~/dev/linux
```

## How do I install pazi?

First, you need to install the `pazi` binary somewhere in your `$PATH`.

Prebuilt binaries are available on the
[releases][releases] page.

If you have the rust toolchain installed, you may alternatively compile from
this repository or run `cargo install pazi`.

After installing the pazi binary, add the following to your `.zshrc` or
`.bashrc`:

```sh
if command -v pazi &>/dev/null; then
  eval "$(pazi init zsh)" # or 'bash'
fi
```

Or if you are a fish user, add the following to your `config.fish`

```sh
if command -v pazi >/dev/null
  status --is-interactive; and pazi init fish | source
end
```

Finally, re-launch the shell and start zapping around :)

## What makes pazi different from *X*

There are several autojump utilities, including [fasd][fasd] (or a better
maintained [fork][fasd-fork]), [z][z], and [autojump][autojump].

This implementation aims to be faster than any of the others (in no small part
due to being in [Rust][rust]), and also safer than `fasd` and `z` which, being
shell-parsers written entirely in shell, are [tricky to get right][fasd-pr].

### So, is it faster?

Pazi is faster than the other autojump implementations it has been benchmarked
against. The results of these benchmarks are documented [here][benchmarks].

## Status

Pazi is currently a work-in-progress. It mostly works, but it's not ready for a
1.0 release yet.

The data-format is likely stable (or will be migrated automatically), so now's
a fine time to try it... but it's quite possible there are bugs and rough
edges. Please do file issues or PRs as appropriate!

## License

GPLv3

## Contributions

Welcome and encouraged; unfortunately, no contributing.md yet.

[releases]: https://github.com/euank/pazi/releases
[fasd]: https://github.com/clvv/fasd
[fasd-fork]: https://github.com/whjvenyl/fasd
[z]: https://github.com/rupa/z
[autojump]: https://github.com/wting/autojump
[rust]: https://www.rust-lang.org/en-US/
[fasd-pr]: https://github.com/clvv/fasd/pull/99
[benchmarks]: docs/Benchmarks.md
