# botsay

Like `cowsay`, but with cute little robots.

```
$ botsay -s 7 "Hello! I'm a little robot."
╭────────────────────────────╮
│ Hello! I'm a little robot. │
╰────────────────────────────╯
    ╲
     ╲
       ▗▙▟▖
       █▐▌█
      ▐▐██▌▌
       ▗▌▐▖
```

Every robot is generated from a seed, and the same seed always draws the same
robot. There are tens of thousands of them:

```
0           1           2           3           4           5           6           7           8
  ▙▟         ▗▟▄▙▖       ▗▟▄▙▖        ▄▙▟▄       ▗██▖       ▐████▌       ▗▄▟▙▄▖      ▗▙▟▖       ▗▟▄▙▖
▗▟▐▌▙▖       █▛█▜█      ▗▟▟█▙▙▖     ▗▟▜██▛▙▖    ▗▟▜▛▙▖      ▟▀██▀▙       █▙▟▙▟█      █▐▌█       ▐█▀█▌
▐▝██▘▌      ▐▐███▌▌     ▐▐▙▄▟▌▌     ▐▝████▘▌    ▐▝██▘▌      ▜████▛      ▐▐████▌▌    ▐▐██▌▌      ▐███▌
  ▌▐         ▐▜▀▛▌        ▐ ▌                                            ▝▛▛▜▜▘      ▗▌▐▖        ▟ ▙
```

## Installation

You need a [Rust toolchain](https://rustup.rs) (1.85 or newer).

```sh
cargo install --git https://github.com/maebert/botsay
```

This installs `botsay`. For thought balloons, link it as `botthink`:
`ln -s ~/.cargo/bin/botsay ~/.cargo/bin/botthink`.

## Usage

```sh
botsay hello                  # a random robot says hello
botsay -s manu hello          # the robot for seed "manu" says hello
fortune | botsay              # read the message from stdin
botthink "hmm..."             # a thought balloon instead
botsay -o -s manu             # just the robot
botsay --gallery 27           # browse the robots for seeds 0 to 26
```

| Option                  | Description                                                    |
|-------------------------|----------------------------------------------------------------|
| `-s`, `--seed <SEED>`   | Robot seed; any string works. Defaults to `$BOTSAY_SEED`, else random. |
| `-b`, `--balloon <STYLE>` | Balloon style (default: `round`)                             |
| `-B`, `--balloons`      | List balloon styles                                            |
| `-o`, `--bot-only`      | Draw only the robot, without a balloon                         |
| `-W`, `--width <N>`     | Wrap the message at N columns (default: 40)                    |
| `--gallery <N>`         | Show the robots for seeds 0 to N-1                             |
| `-h`, `--help`          | Print help                                                     |
| `-V`, `--version`       | Print version                                                  |

### Your own robot

Set `BOTSAY_SEED` in your shell profile to always get the same robot:

```sh
export BOTSAY_SEED="$USER"
```

### Balloon styles

The style names follow [ponysay](https://github.com/erkin/ponysay)'s:

```
ascii         cowsay        linux-vt       round           unicode
+------+       ______       ╔══════╗       ╭──────╮        ┌──────┐
| beep |      < beep >      ║ beep ║       │ beep │        │ beep │
+------+       ------       ╚══════╝       ╰──────╯        └──────┘
    \             \             \              ╲               ╲
```

`botthink` uses `o` or `○` for the trail, and `( )` sides for the cowsay style.

## How the robots are made

A robot is drawn on a pixel grid at most 16 pixels wide and 8 tall. It is
printed with Unicode quadrant block characters (`▘▝▀▖▌▞▛▗▚▐▜▄▙▟█`), each of
which covers 2×2 pixels. Only the left half is drawn and every pixel is
mirrored, so robots are always symmetric.

The seed is hashed into a small random number generator (FNV-1a and SplitMix64),
which then picks the parts one line at a time:

1. **Head and antennas:** head width, square or rounded corners, a tall head,
   or a short one with a center antenna, a pair of antennas, or ears.
2. **Eyes and arms:** dots, wide eyes, tall eyes or a cyclops eye, plus
   stubby, lowered, raised or bent arms.
3. **Mouth:** a slot, a wide mouth, a grill or a smile, unless the eyes sit too
   low to leave room for one.
4. **Base (optional):** legs, feet, wheels, treads, a hover jet or a pedestal.

The generator does not depend on any external crate, so a seed draws the same
robot on every platform. That is a promise within a major version: if a
release changes what a seed draws, it gets a new major version.

## Development

```sh
cargo test                                   # UPDATE_SNAPSHOTS=1 to accept changes
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

`tests/gallery.txt` pins the robot for every seed in the gallery. If you change
the generator on purpose, rerun the tests with `UPDATE_SNAPSHOTS=1` and note in
the changelog that seeds draw different robots.

## Known limitations

- Message widths are counted in characters, so wide characters like CJK or
  emoji make the right edge of the balloon uneven.
- The robots need a font with the Unicode quadrant block characters. Almost
  every modern terminal font has them.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT), at your option.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in this crate by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
