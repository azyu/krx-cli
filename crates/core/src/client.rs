use std::collections::{BTreeMap, BTreeSet};

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::catalog::ApiSpec;
use crate::config::load_config;
use crate::error::{KrxCliError, Result};
use crate::runtime::ResponseFormat;

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
    #[serde(rename = "respCode")]
    resp_code: Option<String>,
    #[serde(rename = "respMsg")]
    resp_msg: Option<String>,
}

pub fn parse_params(
    raw_params: Option<&str>,
    date: Option<&str>,
    api: &ApiSpec,
) -> Result<BTreeMap<String, String>> {
    let params = match (raw_params, date) {
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

    validate_query_params(api, &params)?;
    Ok(params)
}

pub fn parse_fields(raw_fields: Option<&[String]>, api: &ApiSpec) -> Result<Option<Vec<String>>> {
    validate_selected_fields(raw_fields, api)
}

pub fn build_request_plan(
    api: &ApiSpec,
    sample: bool,
    format: ResponseFormat,
    auth_key_override: Option<&str>,
    params: BTreeMap<String, String>,
) -> Result<RequestPlan> {
    validate_query_params(api, &params)?;
    let auth_key = resolve_auth_key(sample, auth_key_override)?;
    let mut url = if sample {
        api.sample_endpoint(&format.to_string())
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
        sample,
        url,
        method: "GET",
        query: params,
        masked_auth_key: mask_auth_key(&auth_key),
        auth_key,
    })
}

pub fn validate_query_params(api: &ApiSpec, params: &BTreeMap<String, String>) -> Result<()> {
    validate_params(api, params)
}

pub fn validate_selected_fields(
    raw_fields: Option<&[String]>,
    api: &ApiSpec,
) -> Result<Option<Vec<String>>> {
    let Some(raw_fields) = raw_fields else {
        return Ok(None);
    };

    if raw_fields.is_empty() {
        return Err(KrxCliError::InvalidInput(
            "--fields requires at least one field name".to_string(),
        ));
    }

    let mut seen = BTreeSet::new();
    let mut fields = Vec::new();

    for field in raw_fields {
        if field.is_empty() {
            return Err(KrxCliError::InvalidInput(
                "field names in --fields cannot be empty".to_string(),
            ));
        }

        reject_control_chars("field", field)?;

        if !api.output_field_names.contains(&field.as_str()) {
            return Err(KrxCliError::InvalidInput(format!(
                "unknown output field for {}: {field}",
                api.api_id
            )));
        }

        if seen.insert(field.clone()) {
            fields.push(field.clone());
        }
    }

    Ok(Some(fields))
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
    if (200..300).contains(&status) {
        return None;
    }

    if let Ok(KrxErrorResponse {
        resp_code,
        resp_msg,
    }) = serde_json::from_str::<KrxErrorResponse>(body)
    {
        match resp_msg.as_deref() {
            Some("Unauthorized Key") if status == 401 => return Some(KrxCliError::UnauthorizedKey),
            Some("Unauthorized API Call") if status == 401 => {
                return Some(KrxCliError::UnauthorizedApiCall);
            }
            Some(_) => {
                return Some(KrxCliError::KrxApiError {
                    status,
                    resp_code,
                    resp_msg,
                });
            }
            None => {}
        }
    }

    Some(KrxCliError::KrxApiError {
        status,
        resp_code: None,
        resp_msg: None,
    })
}

fn resolve_auth_key(sample: bool, auth_key_override: Option<&str>) -> Result<String> {
    match (sample, auth_key_override) {
        (true, Some(auth_key)) => Ok(auth_key.to_string()),
        (true, None) => Ok(SAMPLE_AUTH_KEY.to_string()),
        (false, Some(auth_key)) => Ok(auth_key.to_string()),
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
    use crate::runtime::ResponseFormat;

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
        let plan = build_request_plan(
            api,
            true,
            ResponseFormat::Json,
            None,
            BTreeMap::from([("basDd".to_string(), "20240131".to_string())]),
        )
        .unwrap();
        assert!(plan.url.contains("/svc/sample/apis/idx/krx_dd_trd.json"));
    }

    #[test]
    fn parse_fields_rejects_unknown_output_field() {
        let api = find_api("krx_dd_trd").unwrap();
        let error = parse_fields(Some(&["UNKNOWN".to_string()]), api).unwrap_err();
        assert!(error.to_string().contains("unknown output field"));
    }

    #[test]
    fn parse_fields_accepts_known_output_fields() {
        let api = find_api("krx_dd_trd").unwrap();
        let fields = vec!["BAS_DD".to_string(), "IDX_NM".to_string()];

        assert_eq!(
            parse_fields(Some(&fields), api).unwrap(),
            Some(vec!["BAS_DD".to_string(), "IDX_NM".to_string()])
        );
    }

    #[test]
    fn parse_fields_rejects_empty_field_name() {
        let api = find_api("krx_dd_trd").unwrap();
        let error = parse_fields(Some(&["".to_string()]), api).unwrap_err();
        assert!(error.to_string().contains("cannot be empty"));
    }

    #[test]
    fn validate_query_params_rejects_unknown_field_without_cli() {
        let api = find_api("krx_dd_trd").unwrap();
        let error = validate_query_params(
            api,
            &BTreeMap::from([("foo".to_string(), "bar".to_string())]),
        )
        .unwrap_err();
        assert!(error.to_string().contains("only supports basDd"));
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

    #[test]
    fn map_krx_error_maps_structured_non_401_error() {
        let error = map_krx_error(403, r#"{"respCode":"403","respMsg":"Forbidden"}"#).unwrap();

        match error {
            KrxCliError::KrxApiError {
                status,
                resp_code,
                resp_msg,
            } => {
                assert_eq!(status, 403);
                assert_eq!(resp_code.as_deref(), Some("403"));
                assert_eq!(resp_msg.as_deref(), Some("Forbidden"));
            }
            other => panic!("expected KrxApiError, got {other:?}"),
        }
    }

    #[test]
    fn map_krx_error_maps_unstructured_non_2xx_error() {
        let error = map_krx_error(500, "<html>server error</html>").unwrap();

        match error {
            KrxCliError::KrxApiError {
                status,
                resp_code,
                resp_msg,
            } => {
                assert_eq!(status, 500);
                assert!(resp_code.is_none());
                assert!(resp_msg.is_none());
            }
            other => panic!("expected KrxApiError, got {other:?}"),
        }
    }

    #[test]
    fn map_krx_error_ignores_success_status() {
        assert!(map_krx_error(200, r#"{"respCode":"200","respMsg":"OK"}"#).is_none());
    }
}
