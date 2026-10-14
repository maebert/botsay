//! The robot generator.
//!
//! A bot is drawn on a small pixel grid and rendered with Unicode quadrant
//! block characters (`▘▝▀▖▌▞▛▗▚▐▜▄▙▟█`), each covering 2×2 pixels. Only the
//! left half is drawn; every pixel is mirrored, so bots are always symmetric.
//!
//! Each text line covers two pixel rows:
//!
//! | Line | Rows | Contents                                    |
//! |------|------|---------------------------------------------|
//! | 1    | 0–1  | antennas or ears, top of the head           |
//! | 2    | 2–3  | eyes, arms                                  |
//! | 3    | 4–5  | mouth, arms, bottom of the head             |
//! | 4    | 6–7  | legs, feet, wheels, treads… (optional)      |

use crate::color::{Color, Hsl, Rgb};
use crate::rng::Rng;

/// Quadrant characters, indexed by `TL | TR << 1 | BL << 2 | BR << 3`.
const QUAD: [char; 16] = [' ', '▘', '▝', '▀', '▖', '▌', '▞', '▛', '▗', '▚', '▐', '▜', '▄', '▙', '▟', '█'];
const ROWS: usize = 8;
const MAX_WIDTH: usize = 16;

// ╭─────────────────────────────────────────╮
// │ It looks like you're reading the source │
// │ code. Would you like help?              │
// ╰─────────────────────────────────────────╯
//     ╲
//      ╲
//       ╭──╮
//       │  │
//       @  @
//       ││ ││
//       ││ ││
//       │╰─╯│
//       ╰───╯
const CLIPPY: [&str; 7] = ["╭──╮", "│  │", "@  @", "││ ││", "││ ││", "│╰─╯│", "╰───╯"];
/// Brushed steel.
const CLIPPY_COLOR: Hsl = Hsl { hue: 210, saturation: 15, lightness: 70 };

pub struct Bot {
    /// 3 or 4 lines (Clippy gets 7), without trailing whitespace.
    pub lines: Vec<String>,
    pub color: Rgb,
}

impl Bot {
    /// The bot for `seed`, in the seed's color unless `color` overrides it.
    /// A hue override keeps the seed's saturation and lightness.
    pub fn new(seed: &str, color: Option<Color>) -> Self {
        let (lines, hsl) = if seed.eq_ignore_ascii_case("clippy") {
            (CLIPPY.map(String::from).to_vec(), CLIPPY_COLOR)
        } else {
            (draw(seed).render(), Hsl::from_seed(seed))
        };
        let color = match color {
            None => hsl.to_rgb(),
            Some(Color::Hue(hue)) => Hsl { hue, ..hsl }.to_rgb(),
            Some(Color::Rgb(rgb)) => rgb,
        };
        Bot { lines, color }
    }

    pub fn width(&self) -> usize {
        self.lines.iter().map(|line| line.chars().count()).max().unwrap_or(0)
    }
}

struct Canvas {
    /// Columns on each side of the center axis.
    half: i32,
    px: [[bool; MAX_WIDTH]; ROWS],
}

impl Canvas {
    fn new(head: i32) -> Self {
        let half = head + 3; // room for arms
        debug_assert!(2 * half as usize <= MAX_WIDTH);
        Canvas { half, px: [[false; MAX_WIDTH]; ROWS] }
    }

    /// Sets the pixel at distance `d` from the center axis (`d = 0` is the
    /// innermost column) on both sides. Out-of-range pixels are ignored.
    fn set(&mut self, d: i32, y: i32, on: bool) {
        if (0..self.half).contains(&d) && (0..ROWS as i32).contains(&y) {
            let row = &mut self.px[y as usize];
            row[(self.half - 1 - d) as usize] = on;
            row[(self.half + d) as usize] = on;
        }
    }

    fn hline(&mut self, y: i32, d0: i32, d1: i32, on: bool) {
        for d in d0..=d1 {
            self.set(d, y, on);
        }
    }

    fn vline(&mut self, d: i32, y0: i32, y1: i32, on: bool) {
        for y in y0..=y1 {
            self.set(d, y, on);
        }
    }

    fn render(&self) -> Vec<String> {
        let width = (2 * self.half) as usize;
        let has_base = self.px[6..].iter().any(|row| row.contains(&true));
        let rows = if has_base { 8 } else { 6 };
        let p = |y: usize, x: usize| usize::from(self.px[y][x]);

        let lines: Vec<String> = (0..rows)
            .step_by(2)
            .map(|y| {
                let line: String = (0..width)
                    .step_by(2)
                    .map(|x| QUAD[p(y, x) | p(y, x + 1) << 1 | p(y + 1, x) << 2 | p(y + 1, x + 1) << 3])
                    .collect();
                line.trim_end().to_string()
            })
            .collect();

        // Remove the margin left by unused arm columns.
        let pad = lines
            .iter()
            .filter(|l| !l.is_empty())
            .map(|l| l.chars().take_while(|&c| c == ' ').count())
            .min()
            .unwrap_or(0);
        lines.into_iter().map(|l| l.chars().skip(pad).collect()).collect()
    }
}

#[derive(Clone, Copy)]
enum Antenna {
    Center,
    Pair,
    Ears,
}

#[derive(Clone, Copy, PartialEq)]
enum Eyes {
    Dot,
    Wide,
    Tall,
    Cyclops,
}

#[derive(Clone, Copy)]
enum Mouth {
    Slot,
    Wide,
    Grill,
    Smile,
}

#[derive(Clone, Copy)]
enum Arms {
    Stub,
    Down,
    Up,
    Bolt,
}

#[derive(Clone, Copy)]
enum Base {
    Legs,
    Feet,
    Wheels,
    Treads,
    Hover,
    Pedestal,
}

// ╭───────────────────────────────────────────╮
// │ The order of rng calls is part of the     │
// │ output. Reorder or add a choice and every │
// │ seed draws a different bot.               │
// ╰───────────────────────────────────────────╯
//     ╲
//      ╲
//        ▗▟▄▙▖
//        █▛█▜█
//       ▐▐███▌▌
//        ▐▜▀▛▌
fn draw(seed: &str) -> Canvas {
    let mut rng = Rng::from_seed(seed);

    let hw = rng.pick(&[3, 4, 4, 5]); // head half-width in pixels
    let arms = rng.pick(&[None, None, Some(Arms::Stub), Some(Arms::Down), Some(Arms::Up), Some(Arms::Bolt)]);
    let mut c = Canvas::new(hw);

    // Head.
    let top = rng.pick(&[0, 1, 1]); // a tall head starts at row 0 and has no antenna
    for y in top..6 {
        c.hline(y, 0, hw - 1, true);
    }
    if rng.pick(&[true, false]) {
        // rounded corners
        c.set(hw - 1, top, false);
        c.set(hw - 1, 5, false);
    }

    // Line 1: antennas.
    if top == 1 {
        let antenna =
            [None, Some(Antenna::Center), Some(Antenna::Pair), Some(Antenna::Ears), Some(Antenna::Pair)];
        match rng.pick(&antenna) {
            Some(Antenna::Center) => c.set(0, 0, true),
            Some(Antenna::Pair) => {
                let d = rng.range(1, hw - 2);
                c.set(d, 0, true);
            }
            Some(Antenna::Ears) => c.set(hw - 1, 0, true),
            None => {}
        }
    }

    // Line 2: eyes. Every bot has eyes: two, or one centered cyclops eye.
    // Each eye is a hole surrounded by head pixels: never in the outer column,
    // and never directly above the mouth.
    let mut eyes = rng.pick(&[Eyes::Dot, Eyes::Dot, Eyes::Dot, Eyes::Wide, Eyes::Tall, Eyes::Cyclops]);
    if eyes == Eyes::Wide && hw < 4 {
        eyes = Eyes::Dot;
    }
    let ex = if eyes == Eyes::Wide { rng.range(1, hw - 3) } else { rng.range(1, hw - 2) };
    let ey = rng.pick(&[2, 3]);
    match eyes {
        Eyes::Dot => c.set(ex, ey, false),
        Eyes::Wide => c.hline(ey, ex, ex + 1, false),
        Eyes::Tall => c.vline(ex, 2, 3, false),
        Eyes::Cyclops => c.set(0, ey, false),
    }
    let eyes_low = eyes == Eyes::Tall || ey == 3;

    // Line 3: mouth, unless the eyes sit right above it.
    let mouth = if eyes_low {
        None
    } else {
        rng.pick(&[None, Some(Mouth::Slot), Some(Mouth::Wide), Some(Mouth::Grill), Some(Mouth::Smile)])
    };
    match mouth {
        Some(Mouth::Slot) => c.set(0, 4, false),
        Some(Mouth::Wide) => {
            let w = rng.range(1, hw - 2);
            c.hline(4, 0, w, false);
        }
        Some(Mouth::Grill) => {
            for d in (0..hw - 1).step_by(2) {
                c.set(d, 4, false);
            }
        }
        Some(Mouth::Smile) => c.hline(4, 0, hw - 3, false),
        None => {}
    }

    // Lines 2–3: arms.
    let a = hw; // first column outside the head
    match arms {
        Some(Arms::Stub) => c.vline(a, 3, 4, true),
        Some(Arms::Down) => {
            c.set(a, 3, true);
            c.vline(a + 1, 3, 5, true);
        }
        Some(Arms::Up) => {
            c.set(a, 4, true);
            c.vline(a + 1, 1, 4, true);
        }
        Some(Arms::Bolt) => {
            c.vline(a, 2, 3, true);
            c.vline(a + 1, 4, 5, true);
        }
        None => {}
    }

    // Line 4: base (optional).
    let base = rng.pick(&[
        None,
        Some(Base::Legs),
        Some(Base::Legs),
        Some(Base::Feet),
        Some(Base::Wheels),
        Some(Base::Treads),
        Some(Base::Hover),
        Some(Base::Pedestal),
    ]);
    let lx = rng.range(1, hw - 2);
    match base {
        Some(Base::Legs) => c.vline(lx, 6, 7, true),
        Some(Base::Feet) => {
            c.vline(lx, 6, 7, true);
            c.set(lx + 1, 7, true);
        }
        Some(Base::Wheels) => {
            c.hline(6, lx, lx + 1, true);
            c.set(lx, 7, true);
        }
        Some(Base::Treads) => {
            c.hline(6, 0, hw - 1, true);
            for d in (1..hw).step_by(2) {
                c.set(d, 7, true);
            }
        }
        Some(Base::Hover) => c.set(lx, 6, true),
        Some(Base::Pedestal) => {
            c.hline(6, 0, (hw - 3).max(0), true);
            c.hline(7, 0, hw - 1, true);
        }
        None => {}
    }

    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bots_are_symmetric_and_three_or_four_lines() {
        for seed in (0..2000).map(|i| i.to_string()) {
            let canvas = draw(&seed);
            let width = 2 * canvas.half as usize;
            for row in &canvas.px {
                assert!(row[..width].iter().eq(row[..width].iter().rev()), "seed {seed} is lopsided");
            }
            let lines = canvas.render();
            assert!((3..=4).contains(&lines.len()), "seed {seed}: {lines:?}");
            assert!(lines.iter().all(|line| !line.is_empty()), "seed {seed}: {lines:?}");
        }
    }
}
