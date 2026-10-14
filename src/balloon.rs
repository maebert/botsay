//! Speech and thought balloons.

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Say,
    Think,
}

pub struct Style {
    pub name: &'static str,
    /// Left corner, fill, right corner.
    top: [char; 3],
    bottom: [char; 3],
    sides: Sides,
    trail_say: char,
    trail_think: char,
}

enum Sides {
    /// The same edges on every line.
    Box(char, char),
    /// cowsay: `< >` for a single line, `/ \`, `| |`, `\ /` for several,
    /// and `( )` when thinking.
    Cowsay,
}

// ┌─────────────────────────────────────────────┐
// │ Keep these sorted by name: -B lists them in │
// │ this order.                                 │
// └─────────────────────────────────────────────┘
//     ╲
//      ╲
//        ▗▄▟▙▄▖
//        █▙▟▙▟█
//       ▐▐████▌▌
//        ▝▛▛▜▜▘
/// The names follow ponysay's.
pub const STYLES: &[Style] = &[
    Style {
        name: "ascii",
        top: ['+', '-', '+'],
        bottom: ['+', '-', '+'],
        sides: Sides::Box('|', '|'),
        trail_say: '\\',
        trail_think: 'o',
    },
    Style {
        name: "cowsay",
        top: [' ', '_', ' '],
        bottom: [' ', '-', ' '],
        sides: Sides::Cowsay,
        trail_say: '\\',
        trail_think: 'o',
    },
    Style {
        // Double lines are part of CP437, so the Linux console can draw them.
        name: "linux-vt",
        top: ['╔', '═', '╗'],
        bottom: ['╚', '═', '╝'],
        sides: Sides::Box('║', '║'),
        trail_say: '\\',
        trail_think: 'o',
    },
    Style::DEFAULT,
    Style {
        name: "unicode",
        top: ['┌', '─', '┐'],
        bottom: ['└', '─', '┘'],
        sides: Sides::Box('│', '│'),
        trail_say: '╲',
        trail_think: '○',
    },
];

impl Style {
    pub const DEFAULT: Style = Style {
        name: "round",
        top: ['╭', '─', '╮'],
        bottom: ['╰', '─', '╯'],
        sides: Sides::Box('│', '│'),
        trail_say: '╲',
        trail_think: '○',
    };

    pub fn by_name(name: &str) -> Option<&'static Style> {
        STYLES.iter().find(|style| style.name == name)
    }

    /// The character that trails from the balloon down to the bot.
    pub fn trail(&self, mode: Mode) -> char {
        match mode {
            Mode::Say => self.trail_say,
            Mode::Think => self.trail_think,
        }
    }

    /// Wraps `text` at `width` columns and draws the balloon around it.
    pub fn render(&self, text: &str, width: usize, mode: Mode) -> Vec<String> {
        let lines = wrap(text, width.max(1));
        let w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let n = lines.len();
        let edge = |[left, fill, right]: [char; 3]| {
            let mut s = String::from(left);
            s.extend(std::iter::repeat_n(fill, w + 2));
            s.push(right);
            s.trim_end().to_string()
        };

        let mut out = Vec::with_capacity(n + 2);
        out.push(edge(self.top));
        for (i, line) in lines.iter().enumerate() {
            let (left, right) = match self.sides {
                Sides::Box(left, right) => (left, right),
                Sides::Cowsay if mode == Mode::Think => ('(', ')'),
                Sides::Cowsay => match (i, n) {
                    (_, 1) => ('<', '>'),
                    (0, _) => ('/', '\\'),
                    (i, n) if i == n - 1 => ('\\', '/'),
                    _ => ('|', '|'),
                },
            };
            out.push(format!("{left} {line:<w$} {right}"));
        }
        out.push(edge(self.bottom));
        out
    }
}

/// Greedy word wrap that keeps explicit line breaks and splits words longer
/// than a line. Widths are counted in `char`s, so wide characters such as CJK
/// or emoji make the balloon's right edge uneven.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.replace('\t', "    ").lines() {
        let mut line = String::new();
        let mut len = 0;
        for word in paragraph.split_whitespace() {
            let mut word: Vec<char> = word.chars().collect();
            while !word.is_empty() {
                let sep = usize::from(len > 0);
                if len + sep + word.len() <= width {
                    if sep == 1 {
                        line.push(' ');
                    }
                    line.extend(&word);
                    len += sep + word.len();
                    break;
                }
                if len > 0 {
                    lines.push(std::mem::take(&mut line));
                    len = 0;
                    continue;
                }
                let rest = word.split_off(width.min(word.len()));
                lines.push(word.into_iter().collect());
                word = rest;
            }
        }
        lines.push(line);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap() {
        assert_eq!(super::wrap("the quick brown fox", 10), ["the quick", "brown fox"]);
        assert_eq!(super::wrap("  a   b  ", 10), ["a b"]);
        assert_eq!(super::wrap("one\ntwo\n\nthree", 40), ["one", "two", "", "three"]);
        assert_eq!(super::wrap("abcdefghij xy", 4), ["abcd", "efgh", "ij", "xy"]);
        assert_eq!(super::wrap("héllo wörld", 5), ["héllo", "wörld"]);
        assert_eq!(super::wrap("", 40), [""]);
    }

    #[test]
    fn cowsay_balloons() {
        let cowsay = Style::by_name("cowsay").unwrap();
        assert_eq!(cowsay.render("moo", 40, Mode::Say), [" _____", "< moo >", " -----"]);
        assert_eq!(cowsay.render("a b c", 1, Mode::Say), [" ___", "/ a \\", "| b |", "\\ c /", " ---"]);
        assert_eq!(cowsay.render("a b", 1, Mode::Think), [" ___", "( a )", "( b )", " ---"]);
    }

    #[test]
    fn balloons_are_rectangular() {
        for style in STYLES {
            for mode in [Mode::Say, Mode::Think] {
                let lines = style.render("one two three four five six", 9, mode);
                let widths: Vec<_> = lines[1..lines.len() - 1].iter().map(|l| l.chars().count()).collect();
                assert!(widths.windows(2).all(|w| w[0] == w[1]), "{}: {lines:?}", style.name);
            }
        }
    }
}
