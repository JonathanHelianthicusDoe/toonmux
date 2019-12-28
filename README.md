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

toonmux adheres to [version 2.0.0 of the Semantic Versioning
specification](https://semver.org/spec/v2.0.0.html).

## Install

### Requirements

* [X11](https://en.wikipedia.org/wiki/X_Window_System)
* [GTK](https://en.wikipedia.org/wiki/GTK) `>=3.22 && <4.0` (including
  development files, `libgtk-3-dev` in Debian)
* [libxdo](https://www.semicomplete.com/projects/xdotool/) `>=3.0 && <4.0`
  (including development files, `libxdo-dev` in Debian)
* [rustc &amp; cargo](https://rustup.rs/)

### How

```bash
git clone https://github.com/JonathanHelianthicusDoe/toonmux.git
cd toonmux
cargo rustc --release -- -C target-cpu=native
strip ./target/release/toonmux
./target/release/toonmux
```

## Features

* [x] Multiple controllers (up to 64\* simultaneously) with independent
      bindings
* [x] Rebindable main controls (the controls that all controllers&rsquo;
      bindings map to)
* [x] Collapsable UI (minimal screen space while still being focusable with the
      mouse)
* [x] Ability to have a controller mirror another controller
* [x] toonmux&rsquo;s state is automatically persisted to disk as JSON
* [x] Ability to re-bind controller from one window to another
* [x] Special binding for a &ldquo;low throw&rdquo; of cream pies/evidence
* [x] Ability to add controllers
* [x] Ability to remove controllers
* [x] Speedchat+ support
* [ ] Ability to toggle mirroring globally on and off using a key press
* [ ] Bindable controls for viewing gags and tasks
    * [ ] Automatic keep-alive

\*Actually 32 on 32-bit architectures.

## FAQ

### Why can&rsquo;t I talk using a controller that is mirroring another?

If a controller `A` is mirroring a controller `B`, then `A`&rsquo;s own
&ldquo;talk&rdquo; hotkey gets suppressed; in this situation, the only way for
you to use Speedchat+ with controller `A` is by using `B`&rsquo;s hotkey. If
this is not desired, you can just toggle off mirroring before chatting.

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
(read: [spaghetti](https://en.wikipedia.org/wiki/Spaghetti_code)) approaches to
GUI, but also does not in any way respect the ownership model of Rust. This
means using a lot of atomically reference-counted pointers
([`Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html)) that get passed
into closures, as well as internal mutability (mostly in the form of atomics
for toonmux, but also reader-writer locks &amp; mutexes). This is essentially
writing out explicitly things that are required to be used anyways to use GTK
safely, but Rust doesn&rsquo;t have the luxury of a large runtime to make
things easier (c.f. PyGTK).

That being said, toonmux is still written in Rust in order to stay as
responsive and lightweight as possible while still being safe (for some value
of &ldquo;safe&rdquo;).

### Why don&rsquo;t you use any weak `Arc` references?

The only things wrapped in `Arc` are the global &ldquo;state&rdquo; and the
global UI state. Holding weak references is obviously useless in this case
because the global state never gets deallocated. Only holding references to the
&ldquo;root&rdquo; of global state might seem like an unfortunate choice, but
it works quite well since there is no actual graph structure, ownership-wise
(except internally in GTK&rsquo;s implementation of the UI).

### Why are all of your atomic accesses sequentially consistent?

Because I&rsquo;m a coward. Also because I expect the synchronization
bottleneck to be at the level of the `RwLock`s anyways, not the atomics. If
you&rsquo;re an atomics-semantical wizard, feel free to submit a PR to weaken
the ordering constraints&hellip;

## Legal

toonmux is licensed to anyone under the terms of the [GNU General Public
License, version 3](https://www.gnu.org/licenses/gpl-3.0.html) (or any later
version of the same license, at the licensee&rsquo;s option).

[![GPL v3+](https://www.gnu.org/graphics/gplv3-or-later.png
"GPL v3+")](https://www.gnu.org/licenses/gpl-3.0.html)
