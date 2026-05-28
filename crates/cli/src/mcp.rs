use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, BufWriter, Write};

use krx_core::catalog::{ApiSchemaView, find_api, list_apis};
use krx_core::client::{parse_fields, parse_params};
use krx_core::error::{KrxCliError, Result};
use krx_core::runtime::{
    CallRequest, CallResponse, ResponseBody, ResponseFormat, execute_call, plan_call,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const JSON_RPC_VERSION: &str = "2.0";
const MCP_PROTOCOL_VERSION: &str = "2025-06-18";
const LEGACY_MCP_PROTOCOL_VERSION: &str = "2025-03-26";

pub fn serve() -> Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(stdout.lock());
    let mut line = String::new();

    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(response) = handle_message(trimmed) {
            writeln!(
                writer,
                "{}",
                serde_json::to_string(&response).expect("mcp response should serialize")
            )?;
            writer.flush()?;
        }
    }

    Ok(())
}

fn handle_message(line: &str) -> Option<Value> {
    let request = match serde_json::from_str::<JsonRpcRequest>(line) {
        Ok(request) => request,
        Err(error) => {
            return Some(json_rpc_error(
                Value::Null,
                -32700,
                format!("parse error: {error}"),
            ));
        }
    };

    if request.jsonrpc.as_deref() != Some(JSON_RPC_VERSION) {
        return Some(json_rpc_error(
            request.id.unwrap_or(Value::Null),
            -32600,
            "jsonrpc must be 2.0".to_string(),
        ));
    }

    handle_request(request)
}

fn handle_request(request: JsonRpcRequest) -> Option<Value> {
    match request.method.as_str() {
        "initialize" => Some(handle_initialize(request.id, request.params)),
        "notifications/initialized" => None,
        "tools/list" => Some(handle_tools_list(request.id)),
        "tools/call" => Some(handle_tools_call(request.id, request.params)),
        _ => Some(json_rpc_error(
            request.id.unwrap_or(Value::Null),
            -32601,
            format!("method not found: {}", request.method),
        )),
    }
}

fn handle_initialize(id: Option<Value>, params: Option<Value>) -> Value {
    let Some(id) = id else {
        return json_rpc_error(Value::Null, -32600, "initialize requires an id".to_string());
    };

    let requested = params
        .as_ref()
        .and_then(|value| serde_json::from_value::<InitializeParams>(value.clone()).ok())
        .and_then(|params| params.protocol_version);

    json_rpc_result(
        id,
        json!({
            "protocolVersion": negotiate_protocol_version(requested.as_deref()),
            "capabilities": {
                "tools": {
                    "listChanged": false
                }
            },
            "serverInfo": {
                "name": "krx",
                "version": env!("CARGO_PKG_VERSION")
            }
        }),
    )
}

fn handle_tools_list(id: Option<Value>) -> Value {
    let Some(id) = id else {
        return json_rpc_error(Value::Null, -32600, "tools/list requires an id".to_string());
    };

    json_rpc_result(
        id,
        json!({
            "tools": [
                tool_definition(
                    "krx_list_apis",
                    "List built-in KRX API metadata known to the CLI.",
                    json!({
                        "type": "object",
                        "properties": {},
                        "additionalProperties": false
                    }),
                ),
                tool_definition(
                    "krx_get_api_schema",
                    "Get the schema view for one built-in KRX API.",
                    json!({
                        "type": "object",
                        "properties": {
                            "api_id": {
                                "type": "string"
                            }
                        },
                        "required": ["api_id"],
                        "additionalProperties": false
                    }),
                ),
                tool_definition(
                    "krx_call_api",
                    "Plan or execute a read-only KRX API call with the same validation rules as the CLI.",
                    json!({
                        "type": "object",
                        "properties": {
                            "api_id": {
                                "type": "string"
                            },
                            "sample": {
                                "type": "boolean",
                                "default": false
                            },
                            "date": {
                                "type": "string",
                                "description": "YYYYMMDD shortcut for basDd"
                            },
                            "params": {
                                "type": "object",
                                "properties": {
                                    "basDd": {
                                        "type": "string"
                                    }
                                },
                                "additionalProperties": false
                            },
                            "format": {
                                "type": "string",
                                "enum": ["json", "xml"],
                                "default": "json"
                            },
                            "fields": {
                                "type": "array",
                                "items": {
                                    "type": "string"
                                }
                            },
                            "dry_run": {
                                "type": "boolean",
                                "default": false
                            }
                        },
                        "required": ["api_id"],
                        "additionalProperties": false
                    }),
                )
            ]
        }),
    )
}

fn handle_tools_call(id: Option<Value>, params: Option<Value>) -> Value {
    let Some(id) = id else {
        return json_rpc_error(Value::Null, -32600, "tools/call requires an id".to_string());
    };

    let params = match params {
        Some(params) => params,
        None => {
            return json_rpc_result(
                id,
                tool_error_result(
                    "invalid_tool_arguments",
                    "tools/call requires params".to_string(),
                ),
            );
        }
    };

    let call = match serde_json::from_value::<ToolCallParams>(params) {
        Ok(call) => call,
        Err(error) => {
            return json_rpc_result(
                id,
                tool_error_result(
                    "invalid_tool_arguments",
                    format!("invalid tools/call params: {error}"),
                ),
            );
        }
    };

    json_rpc_result(id, run_tool(call))
}

fn run_tool(call: ToolCallParams) -> Value {
    match call.name.as_str() {
        "krx_list_apis" => tool_success_result(json!({
            "apis": list_apis()
                .iter()
                .map(|api| SchemaListRow {
                    category: api.category,
                    api_id: api.api_id,
                    name: api.name,
                    path: api.path,
                    output_fields: api.output_fields,
                })
                .collect::<Vec<_>>()
        })),
        "krx_get_api_schema" => {
            let args = match serde_json::from_value::<GetApiSchemaArgs>(call.arguments) {
                Ok(args) => args,
                Err(error) => {
                    return tool_error_result(
                        "invalid_tool_arguments",
                        format!("invalid krx_get_api_schema arguments: {error}"),
                    );
                }
            };

            match find_api(&args.api_id) {
                Ok(api) => tool_success_result(
                    serde_json::to_value(ApiSchemaView::from(api))
                        .expect("schema view should serialize"),
                ),
                Err(error) => tool_error_result(error.code(), error.to_string()),
            }
        }
        "krx_call_api" => {
            let args = match serde_json::from_value::<CallApiArgs>(call.arguments) {
                Ok(args) => args,
                Err(error) => {
                    return tool_error_result(
                        "invalid_tool_arguments",
                        format!("invalid krx_call_api arguments: {error}"),
                    );
                }
            };

            match run_call_tool(args) {
                Ok(value) => tool_success_result(value),
                Err(error) => tool_error_result(error.code(), error.to_string()),
            }
        }
        _ => tool_error_result("unknown_tool", format!("unknown tool: {}", call.name)),
    }
}

fn run_call_tool(args: CallApiArgs) -> Result<Value> {
    let api = find_api(&args.api_id)?;
    let params_json = args
        .params
        .as_ref()
        .map(serde_json::to_string)
        .transpose()?;
    let query = parse_params(params_json.as_deref(), args.date.as_deref(), api)?;
    let selected_fields = parse_fields(args.fields.as_deref(), api)?;
    validate_call_fields_usage(args.format, args.dry_run, selected_fields.as_deref())?;

    let request = CallRequest {
        api_id: args.api_id,
        sample: args.sample,
        format: args.format.into(),
        auth_key: None,
        query,
        selected_fields,
    };
    let plan = plan_call(&request)?;

    if args.dry_run {
        return Ok(json!({
            "mode": "dry-run",
            "plan": plan
        }));
    }

    let response = execute_call(&request)?;
    Ok(call_response_json(response))
}

fn validate_call_fields_usage(
    format: McpResponseFormat,
    dry_run: bool,
    selected_fields: Option<&[String]>,
) -> Result<()> {
    if selected_fields.is_none() {
        return Ok(());
    }

    if format != McpResponseFormat::Json {
        return Err(KrxCliError::InvalidInput(
            "fields requires format=json".to_string(),
        ));
    }

    if dry_run {
        return Err(KrxCliError::InvalidInput(
            "fields cannot be used with dry_run=true".to_string(),
        ));
    }

    Ok(())
}

fn call_response_json(response: CallResponse) -> Value {
    let CallResponse {
        api_id,
        sample,
        format,
        status,
        content_type,
        body,
    } = response;

    json!({
        "api_id": api_id,
        "sample": sample,
        "format": format.to_string(),
        "status": status,
        "content_type": content_type,
        "body": response_body_value(body)
    })
}

fn response_body_value(body: ResponseBody) -> Value {
    match body {
        ResponseBody::Json(body) => body,
        ResponseBody::Text(body) => Value::String(body),
    }
}

fn tool_definition(name: &str, description: &str, input_schema: Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema,
        "annotations": {
            "readOnlyHint": true
        }
    })
}

fn tool_success_result(payload: Value) -> Value {
    let text = serde_json::to_string(&payload).expect("tool payload should serialize");

    json!({
        "content": [
            {
                "type": "text",
                "text": text
            }
        ],
        "structuredContent": payload
    })
}

fn tool_error_result(code: &str, message: String) -> Value {
    let payload = json!({
        "error": {
            "code": code,
            "message": message
        }
    });
    let text = serde_json::to_string(&payload).expect("tool error payload should serialize");

    json!({
        "content": [
            {
                "type": "text",
                "text": text
            }
        ],
        "structuredContent": payload,
        "isError": true
    })
}

fn json_rpc_result(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": JSON_RPC_VERSION,
        "id": id,
        "result": result
    })
}

fn json_rpc_error(id: Value, code: i32, message: String) -> Value {
    json!({
        "jsonrpc": JSON_RPC_VERSION,
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

fn negotiate_protocol_version(requested: Option<&str>) -> &'static str {
    match requested {
        Some(LEGACY_MCP_PROTOCOL_VERSION) => LEGACY_MCP_PROTOCOL_VERSION,
        _ => MCP_PROTOCOL_VERSION,
    }
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: Option<String>,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InitializeParams {
    protocol_version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ToolCallParams {
    name: String,
    #[serde(default)]
    arguments: Value,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GetApiSchemaArgs {
    api_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
enum McpResponseFormat {
    #[default]
    Json,
    Xml,
}

impl From<McpResponseFormat> for ResponseFormat {
    fn from(value: McpResponseFormat) -> Self {
        match value {
            McpResponseFormat::Json => Self::Json,
            McpResponseFormat::Xml => Self::Xml,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CallApiArgs {
    api_id: String,
    #[serde(default)]
    sample: bool,
    date: Option<String>,
    params: Option<BTreeMap<String, String>>,
    #[serde(default)]
    format: McpResponseFormat,
    fields: Option<Vec<String>>,
    #[serde(default)]
    dry_run: bool,
}

#[derive(Debug, Serialize)]
struct SchemaListRow<'a> {
    category: &'a str,
    api_id: &'a str,
    name: &'a str,
    path: &'a str,
    output_fields: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negotiate_protocol_version_accepts_legacy_version() {
        assert_eq!(negotiate_protocol_version(Some("2025-03-26")), "2025-03-26");
    }

    #[test]
    fn tool_error_result_sets_is_error_flag() {
        let result = tool_error_result("unknown_tool", "missing".to_string());

        assert_eq!(result["isError"], true);
        assert_eq!(result["structuredContent"]["error"]["code"], "unknown_tool");
    }

    #[test]
    fn call_tool_rejects_fields_with_xml_format() {
        let error = validate_call_fields_usage(
            McpResponseFormat::Xml,
            false,
            Some(&["BAS_DD".to_string()]),
        )
        .unwrap_err();

        assert!(error.to_string().contains("format=json"));
    }
}
