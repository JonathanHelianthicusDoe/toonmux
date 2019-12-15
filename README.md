# toonmux

Multi-toon controller for
[Toontown](https://en.wikipedia.org/wiki/Toontown_Online)-based MMORPGs. Uses
[X11](https://en.wikipedia.org/wiki/X_Window_System) and
[GTK](https://en.wikipedia.org/wiki/GTK), and is designed for use with
[GNU/Linux](https://en.wikipedia.org/wiki/Linux) operating systems (but
*possibly* works on anything that is using X11 as a windowing system).

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

## Legal

toonmux is licensed to anyone under the terms of the [GNU General Public
License, version 3](https://www.gnu.org/licenses/gpl-3.0.html) (or any higher
version of the same license, at the licensee&rsquo;s option).

[![GPL v3+](https://www.gnu.org/graphics/gplv3-or-later.png
"GPL v3+")](https://www.gnu.org/licenses/gpl-3.0.html)
