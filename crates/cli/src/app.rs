use std::{ffi::OsString, io::IsTerminal, process::ExitCode};

use clap::{Error as ClapError, Parser};
use krx_core::catalog::{ApiSchemaView, find_api, list_apis};
use krx_core::client::{RequestPlan, parse_fields, parse_params};
use krx_core::config::{
    AppConfig, ConfigPaths, clear_auth_key, config_paths, load_config, mask_secret, set_auth_key,
};
use krx_core::error::{KrxCliError, Result};
use krx_core::runtime::{CallRequest, CallResponse, ResponseBody, execute_call, plan_call};
use serde::Serialize;
use serde_json::Value;

use crate::cli::{
    CallArgs, Cli, Commands, ConfigCommands, McpCommands, OutputMode, ResponseFormatArg,
    SchemaCommands,
};
use crate::mcp;
use crate::output::{print_json, print_text};

#[derive(Debug, Serialize)]
struct SchemaListRow<'a> {
    category: &'a str,
    api_id: &'a str,
    name: &'a str,
    path: &'a str,
    output_fields: usize,
}

#[derive(Debug, Serialize)]
struct DryRunEnvelope {
    mode: &'static str,
    plan: RequestPlan,
}

#[derive(Debug, Serialize)]
struct CallEnvelope {
    api_id: String,
    sample: bool,
    format: String,
    status: u16,
    content_type: Option<String>,
    body: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct ConfigPathEnvelope {
    config_dir: String,
    config_file: String,
}

#[derive(Debug, Serialize)]
struct ConfigShowEnvelope {
    config_dir: String,
    config_file: String,
    auth_key_present: bool,
    masked_auth_key: Option<String>,
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope {
    error: ErrorPayload,
}

#[derive(Debug, Serialize)]
struct ErrorPayload {
    code: &'static str,
    message: String,
}

pub fn run() -> ExitCode {
    let raw_args: Vec<OsString> = std::env::args_os().collect();
    let requested_output_mode =
        requested_output_mode(&raw_args).unwrap_or_else(default_output_mode);
    let cli = match Cli::try_parse_from(&raw_args) {
        Ok(cli) => cli,
        Err(error) => return emit_clap_error(requested_output_mode, &error),
    };
    let is_mcp_command = matches!(&cli.command, Commands::Mcp { .. });
    let output_mode = cli.output.unwrap_or(requested_output_mode);

    match run_with_cli(cli, output_mode) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            if is_mcp_command {
                eprintln!("{error}");
            } else {
                emit_error(output_mode, &error);
            }
            ExitCode::from(1)
        }
    }
}

fn run_with_cli(cli: Cli, output_mode: OutputMode) -> Result<()> {
    match cli.command {
        Commands::Config { command } => match command {
            ConfigCommands::Path => {
                let paths = config_paths()?;
                let envelope = ConfigPathEnvelope {
                    config_dir: paths.dir.display().to_string(),
                    config_file: paths.file.display().to_string(),
                };

                match output_mode {
                    OutputMode::Json => print_json(&envelope)?,
                    OutputMode::Text => print_text(&render_config_path(&envelope))?,
                }
            }
            ConfigCommands::Show => {
                let paths = config_paths()?;
                let config = load_config()?;
                let envelope = ConfigShowEnvelope {
                    config_dir: paths.dir.display().to_string(),
                    config_file: paths.file.display().to_string(),
                    auth_key_present: config.auth_key.is_some(),
                    masked_auth_key: config.auth_key.as_deref().map(mask_secret),
                };

                match output_mode {
                    OutputMode::Json => print_json(&envelope)?,
                    OutputMode::Text => print_text(&render_config_show(&envelope))?,
                }
            }
            ConfigCommands::SetAuthKey { auth_key } => {
                let paths = set_auth_key(&auth_key)?;
                let config = load_config()?;
                let envelope = config_show_envelope(paths, config);

                match output_mode {
                    OutputMode::Json => print_json(&envelope)?,
                    OutputMode::Text => print_text(&render_config_saved(&envelope))?,
                }
            }
            ConfigCommands::ClearAuthKey => {
                let paths = clear_auth_key()?;
                let config = load_config()?;
                let envelope = config_show_envelope(paths, config);

                match output_mode {
                    OutputMode::Json => print_json(&envelope)?,
                    OutputMode::Text => print_text(&render_config_cleared(&envelope))?,
                }
            }
        },
        Commands::Mcp { command } => match command {
            McpCommands::Serve => mcp::serve()?,
        },
        Commands::Schema { command } => match command {
            SchemaCommands::List => {
                let rows: Vec<_> = list_apis()
                    .iter()
                    .map(|api| SchemaListRow {
                        category: api.category,
                        api_id: api.api_id,
                        name: api.name,
                        path: api.path,
                        output_fields: api.output_fields,
                    })
                    .collect();

                match output_mode {
                    OutputMode::Json => print_json(&rows)?,
                    OutputMode::Text => print_text(&render_schema_list(&rows))?,
                }
            }
            SchemaCommands::Show { api_id } => {
                let api = find_api(&api_id)?;
                let view = ApiSchemaView::from(api);

                match output_mode {
                    OutputMode::Json => print_json(&view)?,
                    OutputMode::Text => print_text(&render_schema_view(&view))?,
                }
            }
        },
        Commands::Call(args) => {
            let api = find_api(&args.api_id)?;
            let params = parse_params(args.params.as_deref(), args.date.as_deref(), api)?;
            let selected_fields = parse_fields(args.fields.as_deref(), api)?;
            validate_fields_usage(output_mode, &args, selected_fields.as_deref())?;
            let request = CallRequest {
                api_id: args.api_id.clone(),
                sample: args.sample,
                format: args.format.into(),
                auth_key: args.auth_key.clone(),
                query: params,
                selected_fields,
            };
            let plan = plan_call(&request)?;

            if args.dry_run {
                let envelope = DryRunEnvelope {
                    mode: "dry-run",
                    plan,
                };

                match output_mode {
                    OutputMode::Json => print_json(&envelope)?,
                    OutputMode::Text => print_text(&render_dry_run(&envelope.plan))?,
                }

                return Ok(());
            }

            let response = execute_call(&request)?;

            match output_mode {
                OutputMode::Json => print_json(&call_json_output(response, args.body_only))?,
                OutputMode::Text => print_text(&render_call_text(response))?,
            }
        }
    }

    Ok(())
}

fn default_output_mode() -> OutputMode {
    if std::io::stdout().is_terminal() {
        OutputMode::Text
    } else {
        OutputMode::Json
    }
}

fn requested_output_mode(args: &[OsString]) -> Option<OutputMode> {
    for (index, arg) in args.iter().enumerate() {
        let Some(arg) = arg.to_str() else {
            continue;
        };

        if let Some(value) = arg.strip_prefix("--output=") {
            return parse_output_mode(value);
        }

        if arg == "--output" {
            let value = args.get(index + 1)?.to_str()?;
            return parse_output_mode(value);
        }
    }

    std::env::var("KRX_OUTPUT")
        .ok()
        .as_deref()
        .and_then(parse_output_mode)
}

fn parse_output_mode(value: &str) -> Option<OutputMode> {
    match value {
        "text" => Some(OutputMode::Text),
        "json" => Some(OutputMode::Json),
        _ => None,
    }
}

fn render_schema_list(rows: &[SchemaListRow<'_>]) -> String {
    let mut lines = vec!["Supported APIs".to_string()];
    for row in rows {
        lines.push(format!(
            "- {:<20} {:<24} {}",
            row.api_id, row.category, row.name
        ));
    }
    lines.join("\n")
}

fn render_schema_view(view: &ApiSchemaView<'_>) -> String {
    format!(
        "API: {name} ({api_id})\nCategory: {category}\nPath: {path}\nDescription: {description}\nSample endpoint: {sample}\nReal endpoint: {real}\nQuery params: basDd (required, YYYYMMDD)\nOutput fields: {outputs}",
        name = view.name,
        api_id = view.api_id,
        category = view.category,
        path = view.path,
        description = view.description,
        sample = view.sample_endpoint,
        real = view.real_endpoint,
        outputs = view.output_fields
    )
}

fn render_dry_run(plan: &RequestPlan) -> String {
    format!(
        "GET {url}\nAUTH_KEY: {auth_key}\nparams: {params}",
        url = plan.url,
        auth_key = plan.masked_auth_key,
        params = serde_json::to_string_pretty(&plan.query).unwrap_or_else(|_| "{}".to_string())
    )
}

fn call_json_output(response: CallResponse, body_only: bool) -> Value {
    let CallResponse {
        api_id,
        sample,
        format,
        status,
        content_type,
        body,
    } = response;
    let body = response_body_value(body);

    if body_only {
        return body;
    }

    serde_json::to_value(CallEnvelope {
        api_id,
        sample,
        format: format.to_string(),
        status,
        content_type,
        body,
    })
    .expect("call envelope should serialize")
}

fn validate_fields_usage(
    output_mode: OutputMode,
    args: &CallArgs,
    selected_fields: Option<&[String]>,
) -> Result<()> {
    if selected_fields.is_none() {
        return Ok(());
    }

    if output_mode != OutputMode::Json {
        return Err(KrxCliError::InvalidInput(
            "--fields requires --output json".to_string(),
        ));
    }

    if args.format != ResponseFormatArg::Json {
        return Err(KrxCliError::InvalidInput(
            "--fields requires --format json".to_string(),
        ));
    }

    if args.dry_run {
        return Err(KrxCliError::InvalidInput(
            "--fields cannot be used with --dry-run".to_string(),
        ));
    }

    Ok(())
}

fn render_call_text(response: CallResponse) -> String {
    match response.body {
        ResponseBody::Json(body) => body.to_string(),
        ResponseBody::Text(body) => body,
    }
}

fn response_body_value(body: ResponseBody) -> Value {
    match body {
        ResponseBody::Json(body) => body,
        ResponseBody::Text(body) => Value::String(body),
    }
}

fn emit_error(output_mode: OutputMode, error: &KrxCliError) {
    match output_mode {
        OutputMode::Json => {
            let envelope = ErrorEnvelope {
                error: ErrorPayload {
                    code: error.code(),
                    message: error.to_string(),
                },
            };

            if print_json(&envelope).is_err() {
                eprintln!("{error}");
            }
        }
        OutputMode::Text => eprintln!("{error}"),
    }
}

fn emit_clap_error(output_mode: OutputMode, error: &ClapError) -> ExitCode {
    let exit_code = u8::try_from(error.exit_code()).unwrap_or(1);

    if output_mode == OutputMode::Json && error.use_stderr() {
        let envelope = ErrorEnvelope {
            error: ErrorPayload {
                code: "cli_parse_error",
                message: error.to_string(),
            },
        };

        if print_json(&envelope).is_err() {
            let _ = error.print();
        }
    } else {
        let _ = error.print();
    }

    ExitCode::from(exit_code)
}

fn config_show_envelope(paths: ConfigPaths, config: AppConfig) -> ConfigShowEnvelope {
    ConfigShowEnvelope {
        config_dir: paths.dir.display().to_string(),
        config_file: paths.file.display().to_string(),
        auth_key_present: config.auth_key.is_some(),
        masked_auth_key: config.auth_key.as_deref().map(mask_secret),
    }
}

fn render_config_path(envelope: &ConfigPathEnvelope) -> String {
    format!(
        "Config dir: {}\nConfig file: {}",
        envelope.config_dir, envelope.config_file
    )
}

fn render_config_show(envelope: &ConfigShowEnvelope) -> String {
    format!(
        "Config dir: {}\nConfig file: {}\nAuth key present: {}\nMasked auth key: {}",
        envelope.config_dir,
        envelope.config_file,
        envelope.auth_key_present,
        envelope.masked_auth_key.as_deref().unwrap_or("not set")
    )
}

fn render_config_saved(envelope: &ConfigShowEnvelope) -> String {
    format!(
        "Saved config to {}\nAuth key: {}",
        envelope.config_file,
        envelope.masked_auth_key.as_deref().unwrap_or("not set")
    )
}

fn render_config_cleared(envelope: &ConfigShowEnvelope) -> String {
    format!("Cleared auth key from {}", envelope.config_file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{CallArgs, OutputMode, ResponseFormatArg};
    use krx_core::runtime::ResponseFormat;

    fn sample_call_args(body_only: bool) -> CallArgs {
        CallArgs {
            api_id: "krx_dd_trd".to_string(),
            sample: true,
            date: Some("20200414".to_string()),
            params: None,
            format: ResponseFormatArg::Json,
            auth_key: None,
            dry_run: false,
            body_only,
            fields: None,
        }
    }

    fn sample_call_response(body: ResponseBody) -> CallResponse {
        CallResponse {
            api_id: "krx_dd_trd".to_string(),
            sample: true,
            format: ResponseFormat::Json,
            status: 200,
            content_type: Some("application/json".to_string()),
            body,
        }
    }

    #[test]
    fn call_json_output_returns_body_only_when_requested() {
        let output = call_json_output(
            sample_call_response(ResponseBody::Json(
                serde_json::json!({"OutBlock_1":[{"BAS_DD":"20200414"}]}),
            )),
            true,
        );

        assert_eq!(
            output,
            serde_json::json!({"OutBlock_1":[{"BAS_DD":"20200414"}]})
        );
    }

    #[test]
    fn call_json_output_wraps_envelope_by_default() {
        let output = call_json_output(
            sample_call_response(ResponseBody::Json(
                serde_json::json!({"OutBlock_1":[{"BAS_DD":"20200414"}]}),
            )),
            false,
        );

        assert_eq!(output["api_id"], "krx_dd_trd");
        assert_eq!(output["body"]["OutBlock_1"][0]["BAS_DD"], "20200414");
    }

    #[test]
    fn validate_fields_usage_rejects_text_output() {
        let error = validate_fields_usage(
            OutputMode::Text,
            &CallArgs {
                fields: Some(vec!["BAS_DD".to_string()]),
                ..sample_call_args(false)
            },
            Some(&["BAS_DD".to_string()]),
        )
        .unwrap_err();

        assert!(error.to_string().contains("--output json"));
    }

    #[test]
    fn validate_fields_usage_rejects_dry_run() {
        let error = validate_fields_usage(
            OutputMode::Json,
            &CallArgs {
                dry_run: true,
                fields: Some(vec!["BAS_DD".to_string()]),
                ..sample_call_args(true)
            },
            Some(&["BAS_DD".to_string()]),
        )
        .unwrap_err();

        assert!(error.to_string().contains("--dry-run"));
    }

    #[test]
    fn validate_fields_usage_rejects_xml_format() {
        let error = validate_fields_usage(
            OutputMode::Json,
            &CallArgs {
                format: ResponseFormatArg::Xml,
                fields: Some(vec!["BAS_DD".to_string()]),
                ..sample_call_args(true)
            },
            Some(&["BAS_DD".to_string()]),
        )
        .unwrap_err();

        assert!(error.to_string().contains("--format json"));
    }

    #[test]
    fn validate_fields_usage_allows_envelope_mode() {
        validate_fields_usage(
            OutputMode::Json,
            &CallArgs {
                fields: Some(vec!["BAS_DD".to_string(), "IDX_NM".to_string()]),
                ..sample_call_args(false)
            },
            Some(&["BAS_DD".to_string(), "IDX_NM".to_string()]),
        )
        .unwrap();
    }

    #[test]
    fn validate_fields_usage_allows_body_only_json_output() {
        validate_fields_usage(
            OutputMode::Json,
            &CallArgs {
                fields: Some(vec!["BAS_DD".to_string(), "IDX_NM".to_string()]),
                ..sample_call_args(true)
            },
            Some(&["BAS_DD".to_string(), "IDX_NM".to_string()]),
        )
        .unwrap();
    }

    #[test]
    fn call_json_output_wraps_text_body_as_json_string() {
        let output = call_json_output(
            sample_call_response(ResponseBody::Text("<xml />".to_string())),
            false,
        );

        assert_eq!(output["body"], "<xml />");
    }

    #[test]
    fn render_call_text_serializes_json_body_without_envelope() {
        let output = render_call_text(sample_call_response(ResponseBody::Json(
            serde_json::json!({"OutBlock_1":[{"BAS_DD":"20200414"}]}),
        )));

        assert_eq!(output, r#"{"OutBlock_1":[{"BAS_DD":"20200414"}]}"#);
    }

    #[test]
    fn render_call_text_returns_text_body_as_is() {
        let output = render_call_text(sample_call_response(ResponseBody::Text(
            "<xml />".to_string(),
        )));

        assert_eq!(output, "<xml />");
    }

    #[test]
    fn response_body_value_maps_text_to_json_string() {
        let output = response_body_value(ResponseBody::Text("<xml />".to_string()));

        assert_eq!(output, Value::String("<xml />".to_string()));
    }

    #[test]
    fn response_body_value_returns_json_body_without_changes() {
        let output = response_body_value(ResponseBody::Json(
            serde_json::json!({"OutBlock_1":[{"BAS_DD":"20200414"}]}),
        ));

        assert_eq!(output["OutBlock_1"][0]["BAS_DD"], "20200414");
    }
}
