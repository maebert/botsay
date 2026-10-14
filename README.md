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
robot in the same color. Here are the first nine:

```
0           1           2           3           4           5           6           7           8
  ▙▟         ▗▟▄▙▖       ▗▟▄▙▖        ▄▙▟▄       ▗██▖       ▐████▌       ▗▄▟▙▄▖      ▗▙▟▖       ▗▟▄▙▖
▗▟▐▌▙▖       █▛█▜█      ▗▟▟█▙▙▖     ▗▟▜██▛▙▖    ▗▟▜▛▙▖      ▟▀██▀▙       █▙▟▙▟█      █▐▌█       ▐█▀█▌
▐▝██▘▌      ▐▐███▌▌     ▐▐▙▄▟▌▌     ▐▝████▘▌    ▐▝██▘▌      ▜████▛      ▐▐████▌▌    ▐▐██▌▌      ▐███▌
  ▌▐         ▐▜▀▛▌        ▐ ▌                                            ▝▛▛▜▜▘      ▗▌▐▖        ▟ ▙
```

## Installation

With [Homebrew](https://brew.sh) on macOS or Linux:

```sh
brew install maebert/tap/botsay
```

Or from source, with a [Rust toolchain](https://rustup.rs) (1.85 or newer):

```sh
cargo install --git https://github.com/maebert/botsay
```

Homebrew also installs `botthink`, which draws thought balloons. With Cargo,
make it yourself: `ln -s ~/.cargo/bin/botsay ~/.cargo/bin/botthink`.

## Usage

```sh
botsay hello                  # a random robot says hello
botsay -s manu hello          # the robot for seed "manu" says hello
fortune | botsay              # read the message from stdin
botthink "hmm..."             # a thought balloon instead
botsay -o -s manu             # just the robot
botsay -c blue hello          # a blue robot
botsay --gallery 27           # browse the robots for seeds 0 to 26
```

| Option                  | Description                                                    |
|-------------------------|----------------------------------------------------------------|
| `-s`, `--seed <SEED>`   | Robot seed; any string works. Defaults to `$BOTSAY_SEED`, else random. |
| `-b`, `--balloon <STYLE>` | Balloon style (default: `round`)                             |
| `-B`, `--balloons`      | List balloon styles                                            |
| `-o`, `--bot-only`      | Draw only the robot, without a balloon                         |
| `-c`, `--color <COLOR>` | Robot color: a hue (`0`–`360`), a hex color (`ff8800`), or a name. Defaults to the seed's color. |
| `-n`, `--no-color`      | Don't color the robot                                          |
| `-W`, `--width <N>`     | Wrap the message at N columns (default: 40)                    |
| `--gallery <N>`         | Show the robots for seeds 0 to N-1                             |
| `-h`, `--help`          | Print help                                                     |
| `-V`, `--version`       | Print version                                                  |

### Your own robot

Set `BOTSAY_SEED` in your shell profile to always get the same robot:

```sh
export BOTSAY_SEED="$USER"
```

### Colors

The seed picks each robot's color: a hue anywhere on the color wheel, with a
saturation and lightness picked for dark terminals. On a light background some
of the paler robots are hard to see; `-c` or `-n` helps there. Only the robot
is colored, not the balloon.

`-c` overrides it with:

- a hue in degrees, like `-c 200`, which keeps the seed's saturation and lightness;
- a name for a hue: `red`, `orange`, `yellow`, `lime`, `green`, `teal`, `cyan`,
  `blue`, `indigo`, `purple`, `magenta` or `pink`;
- an exact hex color, like `-c ff8800` or `-c '#f80'` (quote it, since `#`
  starts a comment in most shells).

Colors are only printed to a terminal, so `botsay hi > file.txt` stays plain.
They use 24-bit color if the terminal sets `COLORTERM=truecolor`, and the
256-color palette otherwise. botsay also follows the usual conventions:
[`NO_COLOR`](https://no-color.org) turns colors off, and `CLICOLOR_FORCE=1`
turns them on even when the output isn't a terminal.

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

## Development

```sh
cargo test                                   # UPDATE_SNAPSHOTS=1 to accept changes
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

### Releasing

1. Bump `version` in `Cargo.toml`, move the changelog entries under a new
   heading for that version, and add its link at the bottom of the changelog.
2. Commit, then tag and push: `git tag v0.2.0 && git push origin v0.2.0`.

The [release workflow](.github/workflows/release.yml) runs the tests, checks
that the tag matches `Cargo.toml`, builds binaries for macOS and Linux,
publishes a GitHub Release, and updates the formula in
[maebert/homebrew-tap](https://github.com/maebert/homebrew-tap) from
[`.github/formula.rb.in`](.github/formula.rb.in).

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
