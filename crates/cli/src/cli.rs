use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormatArg {
    Json,
    Xml,
}

impl From<ResponseFormatArg> for krx_core::runtime::ResponseFormat {
    fn from(value: ResponseFormatArg) -> Self {
        match value {
            ResponseFormatArg::Json => Self::Json,
            ResponseFormatArg::Xml => Self::Xml,
        }
    }
}

impl std::fmt::Display for ResponseFormatArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => f.write_str("json"),
            Self::Xml => f.write_str("xml"),
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "krw", version, about = "Agent-friendly KRX Open API CLI")]
pub struct Cli {
    #[arg(long, global = true, env = "KRX_OUTPUT")]
    pub output: Option<OutputMode>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    Schema {
        #[command(subcommand)]
        command: SchemaCommands,
    },
    Call(CallArgs),
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    Path,
    Show,
    SetAuthKey { auth_key: String },
    ClearAuthKey,
}

#[derive(Debug, Subcommand)]
pub enum SchemaCommands {
    List,
    Show { api_id: String },
}

#[derive(Debug, Args)]
pub struct CallArgs {
    pub api_id: String,

    #[arg(long, help = "Use KRX sample endpoint and sample auth key")]
    pub sample: bool,

    #[arg(
        long,
        help = "Shortcut for the current public query parameter (YYYYMMDD)"
    )]
    pub date: Option<String>,

    #[arg(long, visible_alias = "json", help = "JSON object of query parameters")]
    pub params: Option<String>,

    #[arg(long, default_value_t = ResponseFormatArg::Json, value_enum)]
    pub format: ResponseFormatArg,

    #[arg(
        long,
        env = "KRX_API_KEY",
        help = "Issued KRX API key for real endpoint calls; falls back to ~/.config/krx/config.json"
    )]
    pub auth_key: Option<String>,

    #[arg(long, help = "Validate and render the request without calling the API")]
    pub dry_run: bool,

    #[arg(long, help = "When using --output json, print only the response body")]
    pub body_only: bool,

    #[arg(
        long,
        value_delimiter = ',',
        help = "When using --output json --format json, keep only selected response fields"
    )]
    pub fields: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_parses_body_only_flag() {
        let cli = Cli::try_parse_from([
            "krw",
            "call",
            "krx_dd_trd",
            "--date",
            "20200414",
            "--body-only",
        ])
        .unwrap();

        match cli.command {
            Commands::Call(args) => {
                assert!(args.body_only);
                assert_eq!(args.api_id, "krx_dd_trd");
            }
            _ => panic!("expected call command"),
        }
    }

    #[test]
    fn call_parses_fields_flag() {
        let cli = Cli::try_parse_from([
            "krw",
            "call",
            "krx_dd_trd",
            "--date",
            "20200414",
            "--fields",
            "BAS_DD,IDX_NM",
        ])
        .unwrap();

        match cli.command {
            Commands::Call(args) => {
                assert!(!args.body_only);
                assert_eq!(
                    args.fields,
                    Some(vec!["BAS_DD".to_string(), "IDX_NM".to_string()])
                );
            }
            _ => panic!("expected call command"),
        }
    }
}
