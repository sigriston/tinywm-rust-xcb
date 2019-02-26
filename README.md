# tinywm-rust-xcb

[Rust] + [XCB] port of [tinywm]. Written for fun and study purposes, just
following the current source code of [tinywm].

## Getting Started

Here are the instructions for how to run this on your system.

### Prerequisites

- [Rust] with [cargo] (you get this via the default installation method
  presented on the [Rust] website, using the `rustup` tool).

### Compiling

You can compile the project by just using the default build command for
`cargo`-based projects:

```console
cargo build
```

Or, if you want an optimized build:

```console
cargo build --release
```

### Executing

Using [Xephyr] (a X-on-X implementation) is recommended. First start it on a
new display, for example:

```console
Xephyr :1
```

Then open some windows inside the new display:
```console
DISPLAY=":1" urxvt &
```

Then run the window manager:
```console
DISPLAY=":1" ./target/release/tinywm-rust-xcb
```

## Usage

`tinywm-rust-xcb` has the following functionality:
  - focus follows mouse (X does this by default)
  - move windows with Alt + left button
  - resize windows with Alt + right button

## Limitations

Compared to [tinywm], it doesn't have the Alt + F1 shortcut to raise windows. I
want to add this on a future update, achieving functionality parity with
[tinywm].

## License

This project is licensed under the MIT License - see the
[LICENSE.md](LICENSE.md) file for details.

## Acknowledgements

- **Nick Welch** (@mackstann) who wrote [tinywm]
- **Remi Thebault** (@rtbo) for the [xcb crate]
- **Billie Thompson** (@PurpleBooth) for making a cool [README template]

[Rust]: https://www.rust-lang.org
[XCB]: https://xcb.freedesktop.org
[tinywm]: https://github.com/mackstann/tinywm
[cargo]: https://github.com/rust-lang/cargo/
[Xephyr]: https://www.freedesktop.org/wiki/Software/Xephyr/
[xcb crate]: https://github.com/rtbo/rust-xcb
[README template]: https://gist.github.com/PurpleBooth/109311bb0361f32d87a2
