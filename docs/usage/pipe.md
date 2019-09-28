# `z --pipe`

Pazi provides the `--pipe` option to allow filtering potential jump targets
through a program prior to being jumped to.

The main use-case for this feature is integration with fuzzy-finders like
[`fzf`](fzf) and [`skim`](skim).

## Usage

`z --pipe="fzf" foo` may be used to filter all results matching `z foo` further
before jumping.

You may wish to make an alias, such as `alias zf='z --pipe="fzf"'`, for ease of
use.

## Pipe program semantics

The program passed to `--pipe` should behave as follows:

1. It should accept a list of newline separated strings on stdin.
2. It should return one or more of the strings passed into it on stdout.
3. It should exit 0 on success.

Examples of programs that may be used as pipe programs include `fzf`, `sk`,
`head`, `sort -R`, and so on.

[fzf][https://github.com/junegunn/fzf]
[skim][https://github.com/lotabout/skim]
