# Hemingway's Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.1]

### Changed

-   Revamped the feed fetching code by using future streams. Saw speedups of around 70% (~7 seconds for 14 feeds to ~2 seconds)! Big thanks to Pat Shaughnessy for his [blog post](http://patshaughnessy.net/2020/1/20/downloading-100000-files-using-async-rust) on downloading files in parallel with async Rust!

## [0.7.0] - 2020-07-20

### Added

-   The `remove` command: lets you select feeds and remove them.
-   Loading indicators, courtesy of [indicatif](https://docs.rs/indicatif/0.15.0/indicatif/).

## Fixed

-   Refactored the two main functions! Extracted the logic of processing an individual feed to separate functions.
-   Refactored `config_to_rust` utility function so that it handles case where there is no .hemrc.
-   Switched to using the `<link>` tag instead of the item's `id` for getting the item's URI. Some feeds place an actual GUID in the `id` instead of a URI; `link` is more consistent.

## [0.6.0] - 2020-07-11

### Added

-   The `list` command: list out your saved RSS/Atom feeds
-   `top <num_items>`: the `top` command has replaced the `top5` command. You can now pass in however many entries you want to list for each of your feeds. Defaults to 1 per feed.
-   This changelog!

### Deprecated

-   `top5`: the `top5` command has been removed; use `top` instead (see `Added`).

### Fixed

-   Started using a reqwest `Client` instead of `reqwest::get()`
-   Added handling for feeds that don't have an "updated" field
-   Utility functions have been moved into the `lib.rs` file where they belong
