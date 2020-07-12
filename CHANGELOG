# Hemingway's Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

TODO:

-   [ ] add tests
-   [ ] make it faster? find a way to benchmark

## [0.6.0] - 2020-07-11

### Added

-   The `list` command: list out your saved RSS/Atom feeds
-   `top <numfeeds>`: the `top` command has replaced the `top5` command. You can now pass in however many entries you want to list for each of your feeds. Defaults to 1 per feed.
-   This changelog!

### Deprecated

-   `top5`: the `top5` command has been removed; use `top` instead (see `Added`).

### Fixed

-   Started using a reqwest `Client` instead of `reqwest::get()`
-   Added handling for feeds that don't have an "updated" field
-   Utility functions have been moved into the `lib.rs` file where they belong
