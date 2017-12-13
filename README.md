# :zap: Pazi :zap: &mdash; A fast autojump helper

## What is Pazi?

Pazi is an autojump utility. That is to say, pazi remembers visited directories in the past and makes it easier to get back to them.
A typical use of pazi might look like the following:

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

## How do I use it?

Installing pazi is easy. Put the pazi binary somewhere in your `$PATH`, and then
add the following to your `.zshrc` or `.bashrc` (no other shells are currently
supported):

```sh
if command -v pazi &>/dev/null; then
  eval "$(pazi --init zsh)" # or 'bash'
fi
```

## What makes pazi different from *X*

There are several autojump utilities, including [fasd](https://github.com/clvv/fasd)(or a better maintained [fork](https://github.com/whjvenyl/fasd)), [z](https://github.com/rupa/z), and [autojump](https://github.com/wting/autojump).

This implementation aims to be faster than any of the others (in no small part due to being in [Rust](https://www.rust-lang.org/en-US/), and also safer than `fasd` and `z` which, being shell-parsers written entirely in shell, are [tricky to get right](https://github.com/clvv/fasd/pull/99).

### So, is it faster?

Benchmarks are coming soon! I'm pretty confident pazi will win, but until it has I can't actually claim it's faster... Soon though.

## Status

Pazi is currently a work-in-progress. It mostly works, but it's not ready for a 1.0 release yet.

I think the data-format is stable, so if you want to try it now's a fine time... but it's quite possible you'll run into bugs and rough edges. Please do file issues or PRs if you do!

## License

GPLv3

## Contributions

Welcome and encouraged; unfortunately, no contributing.md yet.
