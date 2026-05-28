use std::io::Write;
use std::process::{Command, Output, Stdio};

use serde_json::{Value, json};

fn run_mcp_session(messages: &[Value]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_krx"))
        .args(["mcp", "serve"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("mcp server should start");

    {
        let stdin = child.stdin.as_mut().expect("stdin should be piped");
        for message in messages {
            writeln!(
                stdin,
                "{}",
                serde_json::to_string(message).expect("message should serialize")
            )
            .expect("message should write");
        }
    }

    child.wait_with_output().expect("mcp server should exit")
}

fn output_lines(output: &Output) -> Vec<Value> {
    String::from_utf8(output.stdout.clone())
        .expect("stdout should be utf-8")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("stdout line should be json"))
        .collect()
}

fn initialize_request(id: i64) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {
                "name": "krx-test",
                "version": "0.0.0"
            }
        }
    })
}

fn initialized_notification() -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    })
}

#[test]
fn initialize_negotiates_protocol_version() {
    let output = run_mcp_session(&[initialize_request(1), initialized_notification()]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let responses = output_lines(&output);
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0]["id"], 1);
    assert_eq!(responses[0]["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(responses[0]["result"]["serverInfo"]["name"], "krx");
    assert!(responses[0]["result"]["capabilities"]["tools"].is_object());
}

#[test]
fn tools_list_returns_read_only_tools() {
    let output = run_mcp_session(&[
        initialize_request(1),
        initialized_notification(),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        }),
    ]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let responses = output_lines(&output);
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[1]["id"], 2);

    let tools = responses[1]["result"]["tools"]
        .as_array()
        .expect("tools should be an array");

    let tool_names: Vec<_> = tools
        .iter()
        .map(|tool| tool["name"].as_str().expect("tool name should be a string"))
        .collect();

    assert_eq!(
        tool_names,
        vec!["krx_list_apis", "krx_get_api_schema", "krx_call_api"]
    );
    assert!(
        tools
            .iter()
            .all(|tool| tool["annotations"]["readOnlyHint"] == true)
    );
}

#[test]
fn tools_call_schema_show_returns_schema_payload() {
    let output = run_mcp_session(&[
        initialize_request(1),
        initialized_notification(),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "krx_get_api_schema",
                "arguments": {
                    "api_id": "krx_dd_trd"
                }
            }
        }),
    ]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let responses = output_lines(&output);
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[1]["id"], 2);

    let payload: Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("tool content should be text"),
    )
    .expect("tool content should be json");

    assert_eq!(payload["api_id"], "krx_dd_trd");
    assert_eq!(payload["query_parameters"][0]["name"], "basDd");
}

#[test]
fn tools_call_dry_run_returns_request_plan_payload() {
    let output = run_mcp_session(&[
        initialize_request(1),
        initialized_notification(),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "krx_call_api",
                "arguments": {
                    "api_id": "krx_dd_trd",
                    "sample": true,
                    "date": "20200414",
                    "dry_run": true
                }
            }
        }),
    ]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let responses = output_lines(&output);
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[1]["id"], 2);

    let payload: Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("tool content should be text"),
    )
    .expect("tool content should be json");

    assert_eq!(payload["mode"], "dry-run");
    assert_eq!(payload["plan"]["api_id"], "krx_dd_trd");
    assert_eq!(payload["plan"]["sample"], true);
}

#[test]
fn invalid_method_returns_json_rpc_error() {
    let output = run_mcp_session(&[
        initialize_request(1),
        initialized_notification(),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "ping"
        }),
    ]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let responses = output_lines(&output);
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[1]["id"], 2);
    assert_eq!(responses[1]["error"]["code"], -32601);
    assert!(
        responses[1]["error"]["message"]
            .as_str()
            .expect("error message should be a string")
            .contains("method not found")
    );
}

#[test]
fn invalid_call_request_returns_tool_error_and_server_stays_alive() {
    let output = run_mcp_session(&[
        initialize_request(1),
        initialized_notification(),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "krx_call_api",
                "arguments": {
                    "api_id": "krx_dd_trd"
                }
            }
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/list"
        }),
    ]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let responses = output_lines(&output);
    assert_eq!(responses.len(), 3);
    assert_eq!(responses[1]["id"], 2);
    assert_eq!(responses[1]["result"]["isError"], true);

    let payload: Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("tool content should be text"),
    )
    .expect("tool content should be json");

    assert_eq!(payload["error"]["code"], "invalid_input");
    assert!(
        payload["error"]["message"]
            .as_str()
            .expect("error message should be a string")
            .contains("missing query parameters")
    );
    assert_eq!(responses[2]["id"], 3);
    assert!(responses[2]["result"]["tools"].is_array());
}
