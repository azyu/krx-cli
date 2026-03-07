use std::collections::BTreeMap;

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::catalog::ApiSpec;
use crate::cli::CallArgs;
use crate::config::load_config;
use crate::error::{KrxCliError, Result};

const SAMPLE_AUTH_KEY: &str = "74D1B99DFBF345BBA3FB4476510A4BED4C78D13A";

#[derive(Debug, Serialize)]
pub struct RequestPlan {
    pub api_id: String,
    pub sample: bool,
    pub url: String,
    pub method: &'static str,
    pub query: BTreeMap<String, String>,
    pub masked_auth_key: String,
    #[serde(skip_serializing)]
    auth_key: String,
}

pub struct ResponseEnvelope {
    pub status: u16,
    pub content_type: Option<String>,
    pub body: String,
}

#[derive(Debug, Deserialize)]
struct KrxErrorResponse {
    #[serde(rename = "respMsg")]
    resp_msg: Option<String>,
}

pub fn parse_params(
    raw_params: Option<&str>,
    date: Option<&str>,
    api: &ApiSpec,
) -> Result<BTreeMap<String, String>> {
    let mut params = match (raw_params, date) {
        (Some(json), None) => parse_json_object(json)?,
        (None, Some(date)) => BTreeMap::from([("basDd".to_string(), date.to_string())]),
        (Some(_), Some(_)) => {
            return Err(KrxCliError::InvalidInput(
                "use either --params or --date, not both".to_string(),
            ));
        }
        (None, None) => {
            return Err(KrxCliError::InvalidInput(format!(
                "missing query parameters for {}; pass --date or --params",
                api.api_id
            )));
        }
    };

    validate_params(api, &mut params)?;
    Ok(params)
}

pub fn build_request_plan(
    api: &ApiSpec,
    args: &CallArgs,
    params: BTreeMap<String, String>,
) -> Result<RequestPlan> {
    let auth_key = resolve_auth_key(args)?;
    let mut url = if args.sample {
        api.sample_endpoint(&args.format.to_string())
    } else {
        api.real_endpoint()
    };

    if !params.is_empty() {
        let query = params
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join("&");
        url.push('?');
        url.push_str(&query);
    }

    Ok(RequestPlan {
        api_id: api.api_id.to_string(),
        sample: args.sample,
        url,
        method: "GET",
        query: params,
        masked_auth_key: mask_auth_key(&auth_key),
        auth_key,
    })
}

pub fn execute_request(plan: &RequestPlan) -> Result<ResponseEnvelope> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "AUTH_KEY",
        HeaderValue::from_str(&plan.auth_key)
            .map_err(|_| KrxCliError::InvalidInput("invalid auth key".to_string()))?,
    );

    let client = Client::builder().default_headers(headers).build()?;
    let response = client.get(&plan.url).send()?;
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());
    let body = response.text()?;

    if let Some(error) = map_krx_error(status, &body) {
        return Err(error);
    }

    Ok(ResponseEnvelope {
        status,
        content_type,
        body,
    })
}

fn map_krx_error(status: u16, body: &str) -> Option<KrxCliError> {
    if status != 401 {
        return None;
    }

    let payload: KrxErrorResponse = serde_json::from_str(body).ok()?;
    match payload.resp_msg.as_deref() {
        Some("Unauthorized Key") => Some(KrxCliError::UnauthorizedKey),
        Some("Unauthorized API Call") => Some(KrxCliError::UnauthorizedApiCall),
        _ => None,
    }
}

fn resolve_auth_key(args: &CallArgs) -> Result<String> {
    match (args.sample, args.auth_key.clone()) {
        (true, Some(auth_key)) => Ok(auth_key),
        (true, None) => Ok(SAMPLE_AUTH_KEY.to_string()),
        (false, Some(auth_key)) => Ok(auth_key),
        (false, None) => load_config()?.auth_key.ok_or(KrxCliError::MissingAuthKey),
    }
}

fn parse_json_object(input: &str) -> Result<BTreeMap<String, String>> {
    let value: serde_json::Value = serde_json::from_str(input)?;
    let object = value
        .as_object()
        .ok_or_else(|| KrxCliError::InvalidInput("--params must be a JSON object".to_string()))?;

    let mut params = BTreeMap::new();
    for (key, value) in object {
        match value {
            serde_json::Value::String(text) => {
                params.insert(key.clone(), text.clone());
            }
            _ => {
                return Err(KrxCliError::InvalidInput(format!(
                    "parameter {key} must be a string"
                )));
            }
        }
    }
    Ok(params)
}

fn validate_params(api: &ApiSpec, params: &BTreeMap<String, String>) -> Result<()> {
    if params.keys().any(|key| key != "basDd") {
        return Err(KrxCliError::InvalidInput(format!(
            "{} only supports basDd in the current public schema",
            api.api_id
        )));
    }

    let bas_dd = params
        .get("basDd")
        .ok_or_else(|| KrxCliError::InvalidInput("missing basDd".to_string()))?;

    reject_control_chars("basDd", bas_dd)?;

    if bas_dd.len() != 8 || !bas_dd.chars().all(|char| char.is_ascii_digit()) {
        return Err(KrxCliError::InvalidInput(
            "basDd must be an 8-digit YYYYMMDD string".to_string(),
        ));
    }

    Ok(())
}

fn reject_control_chars(field: &str, value: &str) -> Result<()> {
    if value.chars().any(|char| char.is_control()) {
        return Err(KrxCliError::InvalidInput(format!(
            "{field} contains control characters"
        )));
    }
    if value.contains(['?', '#', '%']) {
        return Err(KrxCliError::InvalidInput(format!(
            "{field} contains reserved URL characters"
        )));
    }
    Ok(())
}

fn mask_auth_key(auth_key: &str) -> String {
    if auth_key.len() <= 8 {
        return "*".repeat(auth_key.len());
    }

    let suffix = &auth_key[auth_key.len() - 4..];
    format!("{}{}", "*".repeat(auth_key.len() - 4), suffix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::find_api;
    use crate::cli::ResponseFormat;

    #[test]
    fn parse_params_accepts_date_shortcut() {
        let api = find_api("krx_dd_trd").unwrap();
        let params = parse_params(None, Some("20240131"), api).unwrap();
        assert_eq!(params.get("basDd"), Some(&"20240131".to_string()));
    }

    #[test]
    fn parse_params_rejects_unknown_field() {
        let api = find_api("krx_dd_trd").unwrap();
        let error = parse_params(Some(r#"{"foo":"bar"}"#), None, api).unwrap_err();
        assert!(error.to_string().contains("only supports basDd"));
    }

    #[test]
    fn parse_params_rejects_invalid_date() {
        let api = find_api("krx_dd_trd").unwrap();
        let error = parse_params(None, Some("2024-01-31"), api).unwrap_err();
        assert!(error.to_string().contains("8-digit"));
    }

    #[test]
    fn build_request_plan_uses_sample_endpoint() {
        let api = find_api("krx_dd_trd").unwrap();
        let args = CallArgs {
            api_id: "krx_dd_trd".to_string(),
            sample: true,
            date: Some("20240131".to_string()),
            params: None,
            format: ResponseFormat::Json,
            auth_key: None,
            body_only: false,
            dry_run: true,
        };
        let plan = build_request_plan(
            api,
            &args,
            BTreeMap::from([("basDd".to_string(), "20240131".to_string())]),
        )
        .unwrap();
        assert!(plan.url.contains("/svc/sample/apis/idx/krx_dd_trd.json"));
    }

    #[test]
    fn map_krx_error_maps_unauthorized_key() {
        let error =
            map_krx_error(401, r#"{"respCode":"401","respMsg":"Unauthorized Key"}"#).unwrap();
        assert!(matches!(error, KrxCliError::UnauthorizedKey));
    }

    #[test]
    fn map_krx_error_maps_unauthorized_api_call() {
        let error = map_krx_error(
            401,
            r#"{"respCode":"401","respMsg":"Unauthorized API Call"}"#,
        )
        .unwrap();
        assert!(matches!(error, KrxCliError::UnauthorizedApiCall));
    }
}
