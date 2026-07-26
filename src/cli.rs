use std::ffi::{OsStr, OsString};
use std::fmt;

pub(crate) const HELP: &str = "\
Tensor Tycoon - A bilingual terminal AI model strategy game

Usage: tensor_tycoon [OPTIONS]

Options:
  -h, --help       Print help
  -v, --version    Print version";

pub(crate) const VERSION: &str = concat!("tensor_tycoon ", env!("CARGO_PKG_VERSION"));

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum StartupAction {
    Run,
    Help,
    Version,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ParseError {
    argument: OsString,
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unexpected argument '{}'",
            self.argument.to_string_lossy()
        )
    }
}

pub(crate) fn parse(
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<StartupAction, ParseError> {
    let mut arguments = arguments.into_iter();
    let Some(first) = arguments.next() else {
        return Ok(StartupAction::Run);
    };

    let action = match first.as_os_str() {
        argument if argument == OsStr::new("-h") || argument == OsStr::new("--help") => {
            StartupAction::Help
        }
        argument if argument == OsStr::new("-v") || argument == OsStr::new("--version") => {
            StartupAction::Version
        }
        _ => return Err(ParseError { argument: first }),
    };

    if let Some(argument) = arguments.next() {
        return Err(ParseError { argument });
    }

    Ok(action)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_strs(arguments: &[&str]) -> Result<StartupAction, ParseError> {
        parse(arguments.iter().map(OsString::from))
    }

    #[test]
    fn no_arguments_runs_the_game() {
        assert_eq!(parse_strs(&[]), Ok(StartupAction::Run));
    }

    #[test]
    fn recognizes_help_aliases() {
        assert_eq!(parse_strs(&["-h"]), Ok(StartupAction::Help));
        assert_eq!(parse_strs(&["--help"]), Ok(StartupAction::Help));
    }

    #[test]
    fn recognizes_version_aliases() {
        assert_eq!(parse_strs(&["-v"]), Ok(StartupAction::Version));
        assert_eq!(parse_strs(&["--version"]), Ok(StartupAction::Version));
    }

    #[test]
    fn rejects_unknown_and_extra_arguments() {
        assert!(parse_strs(&["--unknown"]).is_err());
        assert!(parse_strs(&["--help", "extra"]).is_err());
        assert!(parse_strs(&["--version", "--help"]).is_err());
    }
}
