use std::process::{Command, Output};

fn run(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tensor_tycoon"))
        .args(arguments)
        .output()
        .expect("tensor_tycoon should run")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

#[test]
fn help_aliases_print_help_to_stdout() {
    for argument in ["-h", "--help"] {
        let output = run(&[argument]);

        assert!(output.status.success());
        assert!(stderr(&output).is_empty());

        let stdout = stdout(&output);
        assert!(stdout.contains("Usage: tensor_tycoon [OPTIONS]"));
        assert!(stdout.contains("-h, --help"));
        assert!(stdout.contains("-v, --version"));
    }
}

#[test]
fn version_aliases_print_package_version_to_stdout() {
    let expected = format!("tensor_tycoon {}\n", env!("CARGO_PKG_VERSION"));

    for argument in ["-v", "--version"] {
        let output = run(&[argument]);

        assert!(output.status.success());
        assert_eq!(stdout(&output), expected);
        assert!(stderr(&output).is_empty());
    }
}

#[test]
fn unknown_and_extra_arguments_fail_strictly() {
    for arguments in [
        &["--unknown"][..],
        &["--help", "extra"][..],
        &["--version", "--help"][..],
    ] {
        let output = run(arguments);

        assert_eq!(output.status.code(), Some(2));
        assert!(stdout(&output).is_empty());

        let stderr = stderr(&output);
        assert!(stderr.contains("unexpected argument"));
        assert!(stderr.contains("tensor_tycoon --help"));
    }
}
