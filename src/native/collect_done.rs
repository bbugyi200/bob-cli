use std::ffi::OsString;

use super::env as bob_env;

const COMMAND_NAME: &str = "bob collect-done";
const DEFAULT_THRESHOLD: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Args {
    threshold: usize,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            threshold: DEFAULT_THRESHOLD,
        }
    }
}

pub(crate) fn run(args: Vec<OsString>) -> i32 {
    match parse_args(args) {
        ParseResult::Run(args) => run_collect_done(args),
        ParseResult::Help => {
            print_help();
            0
        }
        ParseResult::Error(message) => {
            eprintln!("{COMMAND_NAME}: {message}");
            eprintln!("Try '{COMMAND_NAME} --help' for more information.");
            2
        }
    }
}

fn run_collect_done(args: Args) -> i32 {
    println!("Collect done tasks");
    println!("threshold: {}", args.threshold);
    println!("No vault changes made; collection runs in a later phase.");
    0
}

fn parse_args(args: Vec<OsString>) -> ParseResult {
    let mut parsed = Args::default();
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        let text = bob_env::os_to_string(&arg);
        match text.as_str() {
            "-h" | "--help" => return ParseResult::Help,
            "--threshold" => {
                let Some(value) = args.next() else {
                    return ParseResult::Error(
                        "option --threshold requires a value".to_string(),
                    );
                };
                parsed.threshold = match parse_threshold(&value) {
                    Ok(threshold) => threshold,
                    Err(message) => return ParseResult::Error(message),
                };
            }
            "--" => {
                if let Some(extra) = args.next() {
                    return ParseResult::Error(format!(
                        "unexpected positional argument: {}",
                        bob_env::os_to_string(&extra)
                    ));
                }
            }
            _ if let Some(value) = text.strip_prefix("--threshold=") => {
                parsed.threshold = match parse_threshold_text(value) {
                    Ok(threshold) => threshold,
                    Err(message) => return ParseResult::Error(message),
                };
            }
            _ if text.starts_with('-') => {
                return ParseResult::Error(format!(
                    "unrecognized argument: {text}"
                ));
            }
            _ => {
                return ParseResult::Error(format!(
                    "unexpected positional argument: {text}"
                ));
            }
        }
    }

    ParseResult::Run(parsed)
}

enum ParseResult {
    Run(Args),
    Help,
    Error(String),
}

fn parse_threshold(value: &OsString) -> Result<usize, String> {
    parse_threshold_text(&bob_env::os_to_string(value))
}

fn parse_threshold_text(value: &str) -> Result<usize, String> {
    let threshold = value
        .parse::<usize>()
        .map_err(|_| format!("invalid --threshold value: {value}"))?;
    if threshold == 0 {
        return Err("--threshold must be at least 1".to_string());
    }

    Ok(threshold)
}

fn print_help() {
    println!(
        "\
usage: {COMMAND_NAME} [--threshold N]

Collect done and canceled Bob task blocks into archive notes.

options:
  -h, --help       show this help message and exit
  --threshold N    minimum completed/canceled task count per source note \
(default: {DEFAULT_THRESHOLD})"
    );
}

#[cfg(test)]
mod tests {
    use super::{parse_args, Args, ParseResult, DEFAULT_THRESHOLD};
    use std::ffi::OsString;

    #[test]
    fn parses_default_threshold() {
        match parse_args(vec![]) {
            ParseResult::Run(args) => {
                assert_eq!(args.threshold, DEFAULT_THRESHOLD);
            }
            _ => panic!("expected runnable args"),
        }
    }

    #[test]
    fn parses_threshold_option() {
        match parse_args(os_args(["--threshold", "15"])) {
            ParseResult::Run(args) => assert_eq!(args, Args { threshold: 15 }),
            _ => panic!("expected runnable args"),
        }
    }

    #[test]
    fn parses_threshold_equals_option() {
        match parse_args(os_args(["--threshold=3"])) {
            ParseResult::Run(args) => assert_eq!(args, Args { threshold: 3 }),
            _ => panic!("expected runnable args"),
        }
    }

    #[test]
    fn rejects_zero_threshold() {
        match parse_args(os_args(["--threshold", "0"])) {
            ParseResult::Error(message) => {
                assert!(message.contains("at least 1"));
            }
            _ => panic!("expected parse error"),
        }
    }

    fn os_args<const N: usize>(args: [&str; N]) -> Vec<OsString> {
        args.into_iter().map(OsString::from).collect()
    }
}
