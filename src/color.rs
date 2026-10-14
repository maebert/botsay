//! Bot colors.
//!
//! Every seed picks a color in HSL (hue, saturation, lightness). Saturation
//! and lightness stay in a range that reads well on dark and light terminals.
//! The color is printed as a 24-bit ANSI escape where the terminal supports
//! it, and as the closest of the 256 standard colors otherwise.

use std::str::FromStr;

use crate::rng::Rng;

const RESET: &str = "\x1b[0m";

/// See [`Rng::with_stream`].
const COLOR_STREAM: u64 = 1;

/// Hue names accepted by `--color`, in degrees.
const NAMES: &[(&str, u16)] = &[
    ("red", 0),
    ("orange", 30),
    ("yellow", 55),
    ("lime", 90),
    ("green", 130),
    ("teal", 170),
    ("cyan", 190),
    ("blue", 215),
    ("indigo", 245),
    ("purple", 275),
    ("magenta", 300),
    ("pink", 330),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Hue in degrees; saturation and lightness in percent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hsl {
    pub hue: u16,
    pub saturation: u8,
    pub lightness: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorDepth {
    TrueColor,
    Ansi256,
}

/// A color given to `--color`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Rgb(Rgb),
    /// Keeps the seed's saturation and lightness.
    Hue(u16),
}

/// Wraps `text` in `color` if `depth` is set.
pub fn paint(text: &str, color: Rgb, depth: Option<ColorDepth>) -> String {
    match depth {
        Some(depth) => format!("{}{text}{RESET}", color.escape(depth)),
        None => text.to_string(),
    }
}

impl Hsl {
    pub fn from_seed(seed: &str) -> Self {
        let mut rng = Rng::with_stream(seed, COLOR_STREAM);
        let hue = rng.range(0, 359) as u16;
        let saturation = rng.range(55, 85) as u8;
        let lightness = rng.range(55, 70) as u8;
        Hsl { hue, saturation, lightness }
    }

    pub fn to_rgb(self) -> Rgb {
        let s = f64::from(self.saturation.min(100)) / 100.0;
        let l = f64::from(self.lightness.min(100)) / 100.0;
        let h = f64::from(self.hue % 360) / 60.0;

        let chroma = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = chroma * (1.0 - (h % 2.0 - 1.0).abs());
        let (r, g, b) = match h as u8 {
            0 => (chroma, x, 0.0),
            1 => (x, chroma, 0.0),
            2 => (0.0, chroma, x),
            3 => (0.0, x, chroma),
            4 => (x, 0.0, chroma),
            _ => (chroma, 0.0, x),
        };
        let m = l - chroma / 2.0;
        let channel = |v: f64| ((v + m) * 255.0).round().clamp(0.0, 255.0) as u8;
        Rgb { r: channel(r), g: channel(g), b: channel(b) }
    }
}

impl Rgb {
    fn escape(self, depth: ColorDepth) -> String {
        match depth {
            ColorDepth::TrueColor => format!("\x1b[38;2;{};{};{}m", self.r, self.g, self.b),
            ColorDepth::Ansi256 => format!("\x1b[38;5;{}m", self.ansi256()),
        }
    }

    /// The closest color in the 256-color palette: either the 6×6×6 color
    /// cube (16–231) or the gray ramp (232–255).
    fn ansi256(self) -> u8 {
        const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
        let nearest = |v: u8| (0..6).min_by_key(|&i| LEVELS[i].abs_diff(v)).unwrap_or(0);
        let distance = |other: Rgb| {
            [(self.r, other.r), (self.g, other.g), (self.b, other.b)]
                .iter()
                .map(|&(a, b)| u32::from(a.abs_diff(b)).pow(2))
                .sum::<u32>()
        };

        let (r, g, b) = (nearest(self.r), nearest(self.g), nearest(self.b));
        let cube = Rgb { r: LEVELS[r], g: LEVELS[g], b: LEVELS[b] };
        let average = (u16::from(self.r) + u16::from(self.g) + u16::from(self.b)) / 3;
        let step = ((average.saturating_sub(3)) / 10).min(23) as u8;
        let gray = 8 + 10 * step;

        if distance(Rgb { r: gray, g: gray, b: gray }) < distance(cube) {
            232 + step
        } else {
            16 + 36 * r as u8 + 6 * g as u8 + b as u8
        }
    }
}

impl FromStr for Color {
    type Err = String;

    /// Parses a hue name (`blue`), a hue in degrees (`200`), or a hex color
    /// (`ff8800`, `#ff8800`, `#f80`).
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let s = input.trim().to_ascii_lowercase();
        if let Some(&(_, hue)) = NAMES.iter().find(|(name, _)| *name == s) {
            return Ok(Color::Hue(hue));
        }
        // Up to three digits is a hue, so "200" isn't read as #220000.
        if (1..=3).contains(&s.len()) && s.chars().all(|c| c.is_ascii_digit()) {
            return match s.parse::<u16>() {
                Ok(hue) if hue <= 360 => Ok(Color::Hue(hue % 360)),
                _ => Err(format!("invalid hue '{input}' (use 0 to 360)")),
            };
        }
        let hex = s.strip_prefix('#').unwrap_or(&s);
        if hex.chars().all(|c| c.is_ascii_hexdigit()) {
            let digit = |i: usize| u8::from_str_radix(&hex[i..=i], 16).unwrap_or(0);
            let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0);
            match hex.len() {
                3 => return Ok(Color::Rgb(Rgb { r: digit(0) * 17, g: digit(1) * 17, b: digit(2) * 17 })),
                6 => return Ok(Color::Rgb(Rgb { r: byte(0), g: byte(2), b: byte(4) })),
                _ => {}
            }
        }
        Err(format!(
            "invalid color '{input}' (use a hue from 0 to 360, a hex color like ff8800, or a name like blue)"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb(r: u8, g: u8, b: u8) -> Rgb {
        Rgb { r, g, b }
    }

    #[test]
    fn hsl_to_rgb() {
        let hsl = |hue, saturation, lightness| Hsl { hue, saturation, lightness }.to_rgb();
        assert_eq!(hsl(0, 100, 50), rgb(255, 0, 0));
        assert_eq!(hsl(120, 100, 50), rgb(0, 255, 0));
        assert_eq!(hsl(240, 100, 25), rgb(0, 0, 128));
        assert_eq!(hsl(300, 50, 50), rgb(191, 64, 191));
        assert_eq!(hsl(0, 0, 50), rgb(128, 128, 128));
    }

    #[test]
    fn ansi256() {
        assert_eq!(rgb(0, 0, 0).ansi256(), 16);
        assert_eq!(rgb(255, 255, 255).ansi256(), 231);
        assert_eq!(rgb(0, 0, 255).ansi256(), 21);
        assert_eq!(rgb(128, 128, 128).ansi256(), 244);
        assert_eq!(rgb(95, 135, 175).ansi256(), 67);
    }

    #[test]
    fn parse() {
        assert_eq!("blue".parse(), Ok(Color::Hue(215)));
        assert_eq!(" Red ".parse(), Ok(Color::Hue(0)));
        assert_eq!("200".parse(), Ok(Color::Hue(200)));
        assert_eq!("360".parse(), Ok(Color::Hue(0)));
        assert_eq!("ff8800".parse(), Ok(Color::Rgb(rgb(255, 136, 0))));
        assert_eq!("#FF8800".parse(), Ok(Color::Rgb(rgb(255, 136, 0))));
        assert_eq!("#f80".parse(), Ok(Color::Rgb(rgb(255, 136, 0))));
        assert_eq!("123456".parse(), Ok(Color::Rgb(rgb(0x12, 0x34, 0x56))));
        for bad in ["", "nope", "361", "#ff88", "#gggggg", "-5"] {
            assert!(bad.parse::<Color>().is_err(), "{bad:?} should not parse");
        }
    }

    /// Pins every seed's color, like `tests/gallery.txt` pins their shapes.
    /// Run with `UPDATE_SNAPSHOTS=1` to accept a change.
    #[test]
    fn seed_colors_match_snapshot() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/colors.txt");
        let actual: String = (0..180)
            .map(|seed| {
                let Hsl { hue, saturation, lightness } = Hsl::from_seed(&seed.to_string());
                format!("{seed:>3}  hsl({hue}, {saturation}%, {lightness}%)\n")
            })
            .collect();
        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            std::fs::write(path, &actual).unwrap();
        }
        let expected = std::fs::read_to_string(path).unwrap_or_default();
        assert!(actual == expected, "seed colors changed; rerun with UPDATE_SNAPSHOTS=1 if that's intended");
    }
}
