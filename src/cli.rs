use std::io::{self, IsTerminal, Read, Write};
use std::process::ExitCode;

use crate::balloon::{Mode, STYLES, Style};
use crate::bot::Bot;
use crate::color::{Color, ColorDepth, paint};

const DEFAULT_WIDTH: usize = 40;

fn usage(cmd: &str, mode: Mode) -> String {
    let (cow, balloon) = match mode {
        Mode::Say => ("cowsay", "Speech"),
        Mode::Think => ("cowthink", "Thought"),
    };
    let default = Style::DEFAULT.name;
    format!(
        "\
{cmd} - like {cow}, but with cute little robots

Usage: {cmd} [OPTIONS] [MESSAGE]...
       echo MESSAGE | {cmd} [OPTIONS]

Options:
  -s, --seed <SEED>      Bot seed; the same seed always draws the same bot
                         [default: $BOTSAY_SEED, else a random bot]
  -b, --balloon <STYLE>  {balloon} balloon style [default: {default}]
  -B, --balloons         List balloon styles
  -o, --bot-only         Draw only the bot, without a balloon
  -c, --color <COLOR>    Bot color: a hue (0-360), a hex color (ff8800), or a
                         name like blue [default: picked by the seed]
  -n, --no-color         Don't color the bot (also: NO_COLOR=1)
  -W, --width <N>        Wrap the message at N columns [default: {DEFAULT_WIDTH}]
      --gallery <N>      Show the bots for seeds 0 to N-1
  -h, --help             Print help
  -V, --version          Print version
"
    )
}

struct Options {
    seed: Option<String>,
    style: &'static Style,
    bot_only: bool,
    color: Option<Color>,
    no_color: bool,
    width: usize,
    gallery: Option<u32>,
    message: Vec<String>,
}

enum Command {
    Draw(Options),
    Print(String),
}

fn parse_args(cmd: &str, mode: Mode, mut parser: lexopt::Parser) -> Result<Command, lexopt::Error> {
    use lexopt::prelude::*;

    let mut opts = Options {
        seed: None,
        style: &Style::DEFAULT,
        bot_only: false,
        color: None,
        no_color: false,
        width: DEFAULT_WIDTH,
        gallery: None,
        message: Vec::new(),
    };
    while let Some(arg) = parser.next()? {
        match arg {
            Short('h') | Long("help") => return Ok(Command::Print(usage(cmd, mode))),
            Short('V') | Long("version") => {
                return Ok(Command::Print(format!("{cmd} {}\n", env!("CARGO_PKG_VERSION"))));
            }
            Short('B') | Long("balloons") => {
                return Ok(Command::Print(STYLES.iter().map(|style| format!("{}\n", style.name)).collect()));
            }
            Short('s') | Long("seed") => opts.seed = Some(parser.value()?.string()?),
            Short('b') | Long("balloon") => {
                let name = parser.value()?.string()?;
                opts.style = Style::by_name(&name)
                    .ok_or_else(|| format!("unknown balloon style '{name}' (see -B)"))?;
            }
            Short('o') | Long("bot-only") => opts.bot_only = true,
            Short('c') | Long("color") => opts.color = Some(parser.value()?.string()?.parse()?),
            Short('n') | Long("no-color") => opts.no_color = true,
            Short('W') | Long("width") => {
                let n = parser.value()?.string()?;
                opts.width =
                    n.parse().ok().filter(|&w| w > 0).ok_or_else(|| format!("invalid width '{n}'"))?;
            }
            Long("gallery") => {
                let n = parser.value()?.string()?;
                opts.gallery = Some(n.parse().map_err(|_| format!("invalid count '{n}'"))?);
            }
            Value(word) => opts.message.push(word.string()?),
            _ => return Err(arg.unexpected()),
        }
    }
    Ok(Command::Draw(opts))
}

/// A seed for when the user didn't pick one. It only needs to differ between
/// runs, not be unpredictable.
fn random_seed() -> String {
    let nanos =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos() as u64);
    (nanos ^ u64::from(std::process::id()).rotate_left(32)).to_string()
}

/// Color is off with `--no-color` or `NO_COLOR`, and otherwise on when
/// writing to a terminal or when `CLICOLOR_FORCE` is set. Terminals that
/// announce 24-bit color through `COLORTERM` get it; the rest get the
/// 256-color palette.
fn color_depth(
    no_color: bool,
    is_terminal: bool,
    env: impl Fn(&str) -> Option<String>,
) -> Option<ColorDepth> {
    let set = |name: &str| env(name).filter(|value| !value.is_empty());
    if no_color || set("NO_COLOR").is_some() {
        return None;
    }
    let forced = set("CLICOLOR_FORCE").is_some_and(|value| value != "0");
    if !is_terminal && !forced {
        return None;
    }
    match set("COLORTERM").as_deref() {
        Some("truecolor" | "24bit") => Some(ColorDepth::TrueColor),
        _ => Some(ColorDepth::Ansi256),
    }
}

fn gallery(count: u32, depth: Option<ColorDepth>) -> String {
    let seeds: Vec<String> = (0..count).map(|seed| seed.to_string()).collect();
    let mut out = String::new();
    for seeds in seeds.chunks(9) {
        let bots: Vec<Bot> = seeds.iter().map(|seed| Bot::new(seed, None)).collect();
        let widest_seed = seeds.iter().map(String::len).max().unwrap_or(0);
        let column = bots.iter().map(|bot| bot.width() + 4).max().unwrap_or(0).max(widest_seed + 2);
        let mut row = |cells: Vec<String>| {
            out.push_str(cells.concat().trim_end());
            out.push('\n');
        };
        row(seeds.iter().map(|seed| format!("{seed:<column$}")).collect());
        for y in 0..4 {
            row(bots
                .iter()
                .map(|bot| {
                    let line = bot.lines.get(y).map_or("", String::as_str);
                    paint(&format!("{line:<column$}"), bot.color, depth)
                })
                .collect());
        }
        out.push('\n');
    }
    out
}

fn read_message(cmd: &str, words: &[String]) -> Result<String, String> {
    if !words.is_empty() {
        return Ok(words.join(" "));
    }
    let mut stdin = io::stdin();
    if stdin.is_terminal() {
        return Err(format!("no message given (try: {cmd} hello)"));
    }
    let mut message = String::new();
    stdin.read_to_string(&mut message).map_err(|e| format!("reading stdin: {e}"))?;
    Ok(message.trim_end().to_string())
}

fn draw(cmd: &str, mode: Mode, opts: Options) -> Result<String, String> {
    let depth = color_depth(opts.no_color, io::stdout().is_terminal(), |name| std::env::var(name).ok());
    if let Some(count) = opts.gallery {
        return Ok(gallery(count, depth));
    }

    let seed = opts
        .seed
        .or_else(|| std::env::var("BOTSAY_SEED").ok().filter(|seed| !seed.is_empty()))
        .unwrap_or_else(random_seed);
    let bot = Bot::new(&seed, opts.color);

    let mut out = String::new();
    let indent = if opts.bot_only {
        ""
    } else {
        let message = read_message(cmd, &opts.message)?;
        for line in opts.style.render(&message, opts.width, mode) {
            out.push_str(&line);
            out.push('\n');
        }
        let trail = opts.style.trail(mode);
        out.push_str(&format!("    {trail}\n     {trail}\n"));
        "      "
    };
    for line in &bot.lines {
        out.push_str(indent);
        out.push_str(&paint(line, bot.color, depth));
        out.push('\n');
    }
    Ok(out)
}

pub fn run(mode: Mode) -> ExitCode {
    let cmd = match mode {
        Mode::Say => "botsay",
        Mode::Think => "botthink",
    };
    let output =
        parse_args(cmd, mode, lexopt::Parser::from_env()).map_err(|e| e.to_string()).and_then(|command| {
            match command {
                Command::Print(text) => Ok(text),
                Command::Draw(opts) => draw(cmd, mode, opts),
            }
        });
    match output {
        Ok(text) => {
            //  ______________________________________________________
            // < A closed pipe is not my problem: botsay hi | head -1 >
            //  ------------------------------------------------------
            //     \
            //      \
            //       ▝▄▄▘
            //       ▐▛▜▌
            //       ▝██▘
            //       ▗▌▐▖
            let mut stdout = io::stdout().lock();
            match stdout.write_all(text.as_bytes()).and_then(|()| stdout.flush()) {
                Err(error) if error.kind() != io::ErrorKind::BrokenPipe => {
                    eprintln!("{cmd}: {error}");
                    ExitCode::FAILURE
                }
                _ => ExitCode::SUCCESS,
            }
        }
        Err(error) => {
            eprintln!("{cmd}: {error}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_depth() {
        let depth = |no_color, is_terminal, env: &[(&str, &str)]| {
            super::color_depth(no_color, is_terminal, |name| {
                env.iter().find(|(key, _)| *key == name).map(|(_, value)| value.to_string())
            })
        };
        let (ansi, truecolor) = (Some(ColorDepth::Ansi256), Some(ColorDepth::TrueColor));

        assert_eq!(depth(false, true, &[]), ansi);
        assert_eq!(depth(false, false, &[]), None);
        assert_eq!(depth(true, true, &[]), None);
        assert_eq!(depth(false, true, &[("NO_COLOR", "1")]), None);
        assert_eq!(depth(false, true, &[("NO_COLOR", "")]), ansi);
        assert_eq!(depth(false, false, &[("CLICOLOR_FORCE", "1")]), ansi);
        assert_eq!(depth(false, false, &[("CLICOLOR_FORCE", "0")]), None);
        assert_eq!(depth(true, false, &[("CLICOLOR_FORCE", "1")]), None);
        assert_eq!(depth(false, true, &[("COLORTERM", "truecolor")]), truecolor);
        assert_eq!(depth(false, true, &[("COLORTERM", "24bit")]), truecolor);
        assert_eq!(depth(false, true, &[("COLORTERM", "yes")]), ansi);
    }
}
