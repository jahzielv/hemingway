# Hemingway

A small terminal RSS reader. I needed an RSS feed reader and I'm learning Rust, so I decided to build one myself. Hemingway aims to be minimal and easy to use.

## Misc.

Hemingway stores your feeds list in a `.hemrc` file in your home directory. This file is in JSON format, mostly because it's easy to work with. Hemingway will create the `.hemrc` file the first time you run it.

## Installation

```bash
$ cargo install hemingway
```

## Usage

### Check for updates

**Shows you up to 5 of the newest articles if there's been an update to the site since you last ran Hemingway**

```bash
$ hem
```

> 👉 Heads up! This will create a `.hemrc` in your home folder if the file doesn't exist (ie you're running Hemingway for the first time).

### Display 5 newest articles from all your feeds

```bash
$ hem top5
```

### Add a feed

```bash
$ hem add https://example.com/feed.xml
```
