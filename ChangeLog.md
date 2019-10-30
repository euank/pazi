## v0.4.1 - 2019-10-29

This is a bugfix release fixing a notable issue in v0.4.0

### Bug Fixes

* Fix a bug, present only inv v0.4.0, which caused `z -i` to reliably error out.

## v0.4.0 - 2019-10-28

This release adds the `--pipe` flag to `z`, which allows using programs like
[fzf](https://github.com/junegunn/fzf) with pazi.

For more information, please see the
[docs](https://github.com/euank/pazi/blob/v0.4.0/docs/usage/pipe.md) for this
feature.

#### Features

* `z --pipe=<pipe program>` has been added! See the docs folder for usage information.

## v0.3.0 - 2019-05-17

This release adds support for zsh autocompletion.

#### Features

* `pazi init zsh` includes autocompletion code now!

## v0.2.0 - 2018-11-13

This release adds support for the [fish shell](https://fishshell.com/)!

### Features

* `pazi init fish` has been added! See the readme for how to use it.
* pazi is now builds on (and is tested on) macOS. Pre-built release binaries will come in a future release.

### Changes

* [`jump`](https://github.com/gsamokovarov/jump) is now  included in the benchmark code, though new results have not been added to the docs yet.
* Various dependencies have been updated.

### Bug Fixes

* `PROMPT_COMMAND` under `bash` is now correctly updated ([#84](https://github.com/euank/pazi/pull/84)).

## v0.1.0 - 2018-08-06

This release introduces the new subcommand `pazi edit` in addition to fixing a number of bugs.

### Features

* `pazi edit` allows a user to manually remove and re-score entries in the frecency database ([#33](https://github.com/euank/pazi/issues/33)).
* pazi is now benchmarked against `z` and `autojump` ([#47](https://github.com/euank/pazi/issues/47)).

### Changes

* Deleted directories are pruned from the database less aggressively ([#32](https://github.com/euank/pazi/issues/32))
* Various operations have been moved from flags to subcommands. The supported user interface should remain identical.

### Bug Fixes

* Directories with names similar to pazi subcommands (like `igni` or `init`) can now be jumped to ([#70](https://github.com/euank/pazi/issues/70), [#60](https://github.com/euank/pazi/issues/60)).
* Fixed an issue where a path could be added in a non-canonical form ([#76](https://github.com/euank/pazi/issues/76)).


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
