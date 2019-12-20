# toonmux

[![crates.io](https://img.shields.io/crates/v/toonmux)](https://crates.io/crates/toonmux)
[![GPL v3+](https://img.shields.io/badge/license-GNU%20GPL%20v3%2B-bd0000)](./LICENSE)
[![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/JonathanHelianthicusDoe/toonmux)](https://github.com/JonathanHelianthicusDoe/toonmux)

Multi-toon controller for
[Toontown](https://en.wikipedia.org/wiki/Toontown_Online)-based MMORPGs. Uses
[X11](https://en.wikipedia.org/wiki/X_Window_System) and
[GTK](https://en.wikipedia.org/wiki/GTK), and is designed for use with
[GNU/Linux](https://en.wikipedia.org/wiki/Linux) operating systems (but
*possibly* works on anything that is using X11 as a windowing system).

## Version

**toonmux is in an *ALPHA* state, and should only be used at your own risk!**
This will continue to be the case until the version of toonmux is `>=0.1.0`.

## Install

### Requirements

* [X11](https://en.wikipedia.org/wiki/X_Window_System)
* [GTK](https://en.wikipedia.org/wiki/GTK) &ge;3.22 (including development
  files, `libgtk-3-dev` in Debian)
* [libxdo](https://www.semicomplete.com/projects/xdotool/) 3.x (including
  development files, `libxdo-dev` in Debian)
* [rustc &amp; cargo](https://rustup.rs/)

### How

```bash
git clone https://github.com/JonathanHelianthicusDoe/toonmux.git
cd toonmux
cargo rustc --release -- -C target-cpu=native
strip ./target/release/toonmux
./target/release/toonmux
```

## FAQ

### Why is everything spaghetti code?

[The GUI ecosystem](https://areweguiyet.com/) for Rust is not very mature yet,
and essentially all of the options for making a GUI in Rust are either too
immature for serious use (and may possibly vanish at any time), and/or perform
unacceptably poorly for use in something that should just be a lightweight
desktop applet.

As a result, toonmux uses [gtk-rs](https://gtk-rs.org/), which are just Rust
bindings to [GTK](https://en.wikipedia.org/wiki/GTK). Unfortunately, GTK is a
[C](https://en.wikipedia.org/wiki/C_%28programming_language%29) API that is not
only geared towards more &ldquo;object-oriented&rdquo;/&ldquo;classical&rdquo;
(read: spaghetti) approaches to GUI, but also does not in any way respect the
ownership model of Rust. This means using a lot of atomically reference-counted
pointers ([`Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html)) that get
passed into closures, as well as internal mutability (mostly in the form of
atomics for toonmux, but also reader-writer locks &amp; mutexes). This is
essentially writing out explicitly things that are required to be used anyways
to use GTK safely, but Rust doesn&rsquo;t have the luxury of a large runtime to
make things easier (c.f. PyGTK).

That being said, toonmux is still written in Rust in order to stay as
responsive and lightweight as possible while still being safe (for some value
of &ldquo;safe&rdquo;).

## Legal

toonmux is licensed to anyone under the terms of the [GNU General Public
License, version 3](https://www.gnu.org/licenses/gpl-3.0.html) (or any later
version of the same license, at the licensee&rsquo;s option).

[![GPL v3+](https://www.gnu.org/graphics/gplv3-or-later.png
"GPL v3+")](https://www.gnu.org/licenses/gpl-3.0.html)
