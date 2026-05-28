use std::io::Write;
use std::process::{Command, Stdio};

fn krx_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_krx"))
}

#[test]
fn json_output_writes_structured_parse_error_to_stdout() {
    let output = krx_command()
        .args(["--output", "json", "--bogus"])
        .output()
        .expect("command should run");

    assert!(!output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");

    assert_eq!(body["error"]["code"], "cli_parse_error");
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("--bogus")
    );
}

#[test]
fn json_output_writes_structured_error_to_stdout() {
    let output = krx_command()
        .args([
            "--output",
            "json",
            "call",
            "krx_dd_trd",
            "--date",
            "2024-01-31",
            "--sample",
        ])
        .output()
        .expect("command should run");

    assert!(!output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");

    assert_eq!(body["error"]["code"], "invalid_input");
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("basDd must be an 8-digit YYYYMMDD string")
    );
}

#[test]
fn text_output_writes_parse_error_to_stderr() {
    let output = krx_command()
        .args(["--output", "text", "--bogus"])
        .output()
        .expect("command should run");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("--bogus"));
}

#[test]
fn default_output_writes_parse_error_to_stdout_when_stdout_is_not_tty() {
    let output = krx_command()
        .args(["--bogus"])
        .output()
        .expect("command should run");

    assert!(!output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");

    assert_eq!(body["error"]["code"], "cli_parse_error");
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("--bogus")
    );
}

#[test]
fn text_output_writes_runtime_error_to_stderr() {
    let output = krx_command()
        .args([
            "--output",
            "text",
            "call",
            "krx_dd_trd",
            "--date",
            "2024-01-31",
            "--sample",
        ])
        .output()
        .expect("command should run");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("invalid input: basDd must be an 8-digit YYYYMMDD string"));
}

#[test]
fn mcp_serve_does_not_emit_cli_error_envelope_to_stdout() {
    let mut child = krx_command()
        .args(["--output", "json", "mcp", "serve"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("command should run");

    {
        let stdin = child.stdin.as_mut().expect("stdin should be piped");
        writeln!(stdin, "{{ not-json").expect("stdin write should succeed");
    }

    let output = child.wait_with_output().expect("command should finish");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let line = stdout
        .lines()
        .next()
        .expect("stdout should contain a response");
    let body: serde_json::Value = serde_json::from_str(line).expect("stdout should be json");

    assert_eq!(body["jsonrpc"], "2.0");
    assert!(body.get("error").is_some());
    assert!(body.get("result").is_none());
    assert!(body["error"].get("code").is_some());
    assert!(
        body.get("error")
            .and_then(|value| value.get("message"))
            .is_some()
    );
}
