use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::{env, fs};

const BOT_7: &str = " ▗▙▟▖\n █▐▌█\n▐▐██▌▌\n ▗▌▐▖\n";

/// `botsay`, or `botthink` when `bin` says so, without picking up settings
/// from the environment running the tests.
fn command(bin: &str, args: &[&str]) -> Command {
    let mut command = Command::new(bin);
    command.args(args);
    command.env_remove("BOTSAY_SEED");
    command
}

fn botsay(args: &[&str]) -> Command {
    command(env!("CARGO_BIN_EXE_botsay"), args)
}

fn run(mut command: Command, stdin: &str) -> Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start botsay");
    child.stdin.take().unwrap().write_all(stdin.as_bytes()).unwrap();
    child.wait_with_output().unwrap()
}

fn stdout(command: Command) -> String {
    let out = run(command, "");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).unwrap()
}

fn indented(bot: &str) -> String {
    bot.lines().map(|line| format!("      {line}\n")).collect()
}

// +----------------------------------------------+
// | If I look different, every seed changed. Was |
// | that on purpose?                             |
// +----------------------------------------------+
//     \
//      \
//        ▗▙▟▖
//        █▐▌█
//       ▐▐██▌▌
//        ▗▌▐▖
/// Pins the shape of every seed in the gallery. Run with `UPDATE_SNAPSHOTS=1`
/// to accept a change.
#[test]
fn gallery_matches_snapshot() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/gallery.txt");
    let actual = stdout(botsay(&["--gallery", "180"]));
    if env::var_os("UPDATE_SNAPSHOTS").is_some() {
        fs::write(&path, &actual).unwrap();
    }
    let expected = fs::read_to_string(&path).unwrap_or_default();
    assert!(actual == expected, "the gallery changed; rerun with UPDATE_SNAPSHOTS=1 if that's intended");
}

#[test]
fn says_a_message() {
    let expected = format!("╭───────╮\n│ hello │\n╰───────╯\n    ╲\n     ╲\n{}", indented(BOT_7));
    assert_eq!(stdout(botsay(&["-s", "7", "hello"])), expected);
    assert!(stdout(botsay(&["-s7", "-b", "cowsay", "moo"])).starts_with(" _____\n< moo >\n -----\n    \\\n"));
}

#[test]
fn reads_message_from_stdin() {
    let out = run(botsay(&["-s", "7"]), "from a pipe\n");
    assert!(String::from_utf8(out.stdout).unwrap().contains("│ from a pipe │"));
}

#[test]
fn thinks_when_called_botthink() {
    let botthink =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("botthink{}", env::consts::EXE_SUFFIX));
    fs::copy(env!("CARGO_BIN_EXE_botsay"), &botthink).unwrap();
    let botthink = botthink.to_str().unwrap();

    let expected = format!("╭─────╮\n│ hmm │\n╰─────╯\n    ○\n     ○\n{}", indented(BOT_7));
    assert_eq!(stdout(command(botthink, &["-s", "7", "hmm"])), expected);
    assert!(stdout(command(botthink, &["-s", "7", "-b", "cowsay", "hmm"])).starts_with(" _____\n( hmm )\n"));
}

#[test]
fn seed_comes_from_flag_or_environment() {
    assert_eq!(stdout(botsay(&["-os7"])), BOT_7);

    let mut from_env = botsay(&["-o"]);
    from_env.env("BOTSAY_SEED", "7");
    assert_eq!(stdout(from_env), BOT_7);

    let mut flag_wins = botsay(&["-o", "-s", "7"]);
    flag_wins.env("BOTSAY_SEED", "something else");
    assert_eq!(stdout(flag_wins), BOT_7);
}

#[test]
fn usage_errors_exit_with_2() {
    for (args, error) in [
        (&["-b", "nope", "hi"][..], "unknown balloon style 'nope' (see -B)"),
        (&["-W", "0", "hi"], "invalid width '0'"),
        (&["--seed"], "missing argument for option '--seed'"),
        (&["-x"], "invalid option '-x'"),
    ] {
        let out = run(botsay(args), "");
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(stderr.starts_with(&format!("botsay: {error}")), "{args:?}: {stderr}");
        assert!(out.stdout.is_empty());
    }
}
