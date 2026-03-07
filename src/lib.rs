pub mod catalog;
pub mod cli;
pub mod client;
pub mod config;
pub mod error;
pub mod output;

use std::io::IsTerminal;

use clap::Parser;
use serde::Serialize;
use serde_json::Value;

use crate::catalog::{ApiSchemaView, find_api, list_apis};
use crate::cli::{Cli, Commands, ConfigCommands, OutputMode, SchemaCommands};
use crate::client::{build_request_plan, execute_request, parse_params};
use crate::config::{AppConfig, config_paths, load_config, mask_secret, set_auth_key};
use crate::error::Result;
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
    plan: client::RequestPlan,
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

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let output_mode = cli.output.unwrap_or_else(default_output_mode);

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
                let paths = config::clear_auth_key()?;
                let config = load_config()?;
                let envelope = config_show_envelope(paths, config);

                match output_mode {
                    OutputMode::Json => print_json(&envelope)?,
                    OutputMode::Text => print_text(&render_config_cleared(&envelope))?,
                }
            }
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
            let plan = build_request_plan(api, &args, params)?;

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

            let response = execute_request(&plan)?;

            match output_mode {
                OutputMode::Json => {
                    let body = match args.format {
                        cli::ResponseFormat::Json => serde_json::from_str(&response.body)
                            .unwrap_or_else(|_| serde_json::Value::String(response.body.clone())),
                        cli::ResponseFormat::Xml => {
                            serde_json::Value::String(response.body.clone())
                        }
                    };

                    print_json(&call_json_output(
                        api.api_id,
                        &args,
                        response.status,
                        response.content_type,
                        body,
                    ))?;
                }
                OutputMode::Text => print_text(&response.body)?,
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

fn render_dry_run(plan: &client::RequestPlan) -> String {
    format!(
        "GET {url}\nAUTH_KEY: {auth_key}\nparams: {params}",
        url = plan.url,
        auth_key = plan.masked_auth_key,
        params = serde_json::to_string_pretty(&plan.query).unwrap_or_else(|_| "{}".to_string())
    )
}

fn call_json_output(
    api_id: &str,
    args: &cli::CallArgs,
    status: u16,
    content_type: Option<String>,
    body: Value,
) -> Value {
    if args.body_only {
        return body;
    }

    serde_json::to_value(CallEnvelope {
        api_id: api_id.to_string(),
        sample: args.sample,
        format: args.format.to_string(),
        status,
        content_type,
        body,
    })
    .expect("call envelope should serialize")
}

fn config_show_envelope(paths: config::ConfigPaths, config: AppConfig) -> ConfigShowEnvelope {
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
    use crate::cli::{CallArgs, ResponseFormat};

    fn sample_call_args(body_only: bool) -> CallArgs {
        CallArgs {
            api_id: "krx_dd_trd".to_string(),
            sample: true,
            date: Some("20200414".to_string()),
            params: None,
            format: ResponseFormat::Json,
            auth_key: None,
            dry_run: false,
            body_only,
        }
    }

    #[test]
    fn call_json_output_returns_body_only_when_requested() {
        let output = call_json_output(
            "krx_dd_trd",
            &sample_call_args(true),
            200,
            Some("application/json".to_string()),
            serde_json::json!({"OutBlock_1":[{"BAS_DD":"20200414"}]}),
        );

        assert_eq!(
            output,
            serde_json::json!({"OutBlock_1":[{"BAS_DD":"20200414"}]})
        );
    }

    #[test]
    fn call_json_output_wraps_envelope_by_default() {
        let output = call_json_output(
            "krx_dd_trd",
            &sample_call_args(false),
            200,
            Some("application/json".to_string()),
            serde_json::json!({"OutBlock_1":[{"BAS_DD":"20200414"}]}),
        );

        assert_eq!(output["api_id"], "krx_dd_trd");
        assert_eq!(output["body"]["OutBlock_1"][0]["BAS_DD"], "20200414");
    }
}
