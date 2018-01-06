## v0.0.2 - 2018-01-05

This is a bug-fix release of pazi. This fixes a major issue when using pazi
under bash with an existing, suitably complex, `PROMPT_COMMAND` set.

#### Changes
* Default `z` output now has a fixed-width rank value (for better alignment)

#### Bug Fixes
* Fix bug where `pazi init bash` could conflict with an existing `PROMPT_COMMAND` ([#41](https://github.com/euank/pazi/issues/41))

## v0.0.1 - 2017-12-22

This is the initial release of pazi! This v0.0.1 release indicates that pazi's
generally usable, although its featureset is sparse.

Please file issues for any features you think it should have or any issues you run into.

#### Features
* The `z` alias supports jumping between directories in `zsh` and `bash`
* `pazi import` can import directories from `fasd`
* `z -i` displays an interactive prompt
