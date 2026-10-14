# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
A change to the robot drawn for an existing seed is a breaking change.

## [Unreleased]

### Changed

- botsay exits with 1 when something goes wrong while it runs, like reading
  stdin, and keeps 2 for wrong options.

### Fixed

- Errors writing the output, like a full disk, are reported instead of
  ignored. A closed pipe (`botsay hi | head -1`) still isn't an error.

## [0.2.1] - 2026-10-13

### Added

- A surprise for one particular seed.

## [0.2.0] - 2026-10-13

### Added

- Robots are colored. The seed picks the color; robot shapes are unchanged.
- `-c`/`--color` sets the color as a hue, a hue name or a hex color.
- `-n`/`--no-color` turns colors off. botsay also respects `NO_COLOR` and
  `CLICOLOR_FORCE`, and uses 24-bit color when `COLORTERM` announces it.

## [0.1.0] - 2026-10-13

### Added

- `botsay` and `botthink` commands.
- Seeded robot generator: heads, antennas, eyes, mouths, arms and bases,
  always symmetric and 3 or 4 lines tall.
- `--seed` option and `BOTSAY_SEED` environment variable.
- Balloon styles `ascii`, `cowsay`, `linux-vt`, `round` (default) and `unicode`.
- `--bot-only`, `--width` and `--gallery` options.
