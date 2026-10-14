use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::{env, fs};

const BOT_7: &str = " ▗▙▟▖\n █▐▌█\n▐▐██▌▌\n ▗▌▐▖\n";

/// `botsay`, or `botthink` when `bin` says so, without picking up settings
/// from the environment running the tests.
fn command(bin: &str, args: &[&str]) -> Command {
    let mut command = Command::new(bin);
    command.args(args);
    for var in ["BOTSAY_SEED", "NO_COLOR", "CLICOLOR_FORCE", "COLORTERM"] {
        command.env_remove(var);
    }
    command
}

fn botsay(args: &[&str]) -> Command {
    command(env!("CARGO_BIN_EXE_botsay"), args)
}

fn run(mut command: Command, stdin: &[u8]) -> Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start botsay");
    child.stdin.take().unwrap().write_all(stdin).unwrap();
    child.wait_with_output().unwrap()
}

fn stdout(command: Command) -> String {
    let out = run(command, b"");
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
    let out = run(botsay(&["-s", "7"]), b"from a pipe\n");
    assert!(String::from_utf8(out.stdout).unwrap().contains("│ from a pipe │"));
}

#[test]
fn thinks_when_called_botthink() {
    let botthink =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("botthink{}", env::consts::EXE_SUFFIX));
    // A hard link rather than a copy: running a freshly written executable can
    // fail with "Text file busy" on Linux while other tests spawn processes.
    let _ = fs::remove_file(&botthink);
    fs::hard_link(env!("CARGO_BIN_EXE_botsay"), &botthink).unwrap();
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
fn colors_only_the_bot() {
    let colored = |args: &[&str], env: &[(&str, &str)]| {
        let mut command = botsay(args);
        command.envs(env.iter().copied());
        stdout(command)
    };
    let forced = [("CLICOLOR_FORCE", "1")];

    assert!(!colored(&["-s", "7", "hi"], &[]).contains('\x1b'), "colored a pipe");

    let out = colored(&["-s", "7", "hi"], &forced);
    let (balloon, bot) = out.split_at(out.find("     ╲\n").unwrap() + "     ╲\n".len());
    assert!(!balloon.contains('\x1b'), "colored the balloon:\n{out}");
    assert!(bot.lines().all(|line| line.starts_with("      \x1b[38;5;") && line.ends_with("\x1b[0m")));

    let truecolor = [("CLICOLOR_FORCE", "1"), ("COLORTERM", "truecolor")];
    let out = colored(&["-s", "7", "-o", "-c", "ff8800"], &truecolor);
    assert_eq!(out.lines().next().unwrap(), "\x1b[38;2;255;136;0m ▗▙▟▖\x1b[0m");

    assert_eq!(colored(&["-s", "7", "-o", "-n"], &forced), BOT_7);
    assert_eq!(colored(&["-s", "7", "-o"], &[("CLICOLOR_FORCE", "1"), ("NO_COLOR", "1")]), BOT_7);
}

#[test]
fn stops_quietly_when_the_reader_does() {
    let mut child =
        botsay(&["--gallery", "10000"]).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let mut first_bytes = [0; 16];
    child.stdout.take().unwrap().read_exact(&mut first_bytes).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stderr), "");
}

#[test]
#[cfg(target_os = "linux")]
fn reports_a_full_disk() {
    let full = fs::OpenOptions::new().write(true).open("/dev/full").unwrap();
    let out = botsay(&["hi"]).stdout(full).stderr(Stdio::piped()).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).starts_with("botsay: No space left on device"));
}

#[test]
fn usage_errors_exit_with_2() {
    for (args, error) in [
        (&["-b", "nope", "hi"][..], "unknown balloon style 'nope' (see -B)"),
        (&["-c", "nope", "hi"], "invalid color 'nope'"),
        (&["-W", "0", "hi"], "invalid width '0'"),
        (&["--seed"], "missing argument for option '--seed'"),
        (&["-x"], "invalid option '-x'"),
    ] {
        let out = run(botsay(args), b"");
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(stderr.starts_with(&format!("botsay: {error}")), "{args:?}: {stderr}");
        assert!(out.stdout.is_empty());
    }
}

#[test]
fn runtime_errors_exit_with_1() {
    let out = run(botsay(&["-s", "7"]), b"not utf-8: \xff\n");
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8(out.stderr).unwrap().starts_with("botsay: reading stdin:"));
}

#[test]
fn it_looks_like_youre_writing_a_test() {
    let out = stdout(botsay(&["-s", "clippy", "-o"]));
    assert_eq!(out, "╭──╮\n│  │\n@  @\n││ ││\n││ ││\n│╰─╯│\n╰───╯\n");
}
