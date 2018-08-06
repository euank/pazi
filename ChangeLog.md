## v0.1.0 - 2018-08-06

This release introduces the new subcommand `pazi edit` in addition to fixing a number of bugs.

### Features

* `pazi edit` allows a user to manually remove and re-score entries in the frecency database ([#33](https://github.com/euank/pazi/issues/33)).
* pazi is now benchmarked against `z` and `autojump` ([#47](https://github.com/euank/pazi/issues/47)).

### Changes

* Deleted directories are pruned from the database less aggressively ([#32](https://github.com/euank/pazi/issues/32))
* Various operations have been moved from being flags to being subcommands. The supported user interface should remain identical.

### Bugfixes

* Directories named similar to subcommands (like `igni` or `init`) can now be jumped to ([#70](https://github.com/euank/pazi/issues/70), [#60](https://github.com/euank/pazi/issues/60)).
* Providing a non-canonical directory path as an argument to `z` could result in it being added to the database in that non-canonical form. The path will now be canonicalized ([#76](https://github.com/euank/pazi/issues/76)).


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
