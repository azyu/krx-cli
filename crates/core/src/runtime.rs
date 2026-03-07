use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

use crate::catalog::find_api;
use crate::client::{
    RequestPlan, ResponseEnvelope, build_request_plan, execute_request, validate_query_params,
    validate_selected_fields,
};
use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormat {
    Json,
    Xml,
}

impl std::fmt::Display for ResponseFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => f.write_str("json"),
            Self::Xml => f.write_str("xml"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CallRequest {
    pub api_id: String,
    pub sample: bool,
    pub format: ResponseFormat,
    pub auth_key: Option<String>,
    pub query: BTreeMap<String, String>,
    pub selected_fields: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResponseBody {
    Json(Value),
    Text(String),
}

#[derive(Debug, Clone)]
pub struct CallResponse {
    pub api_id: String,
    pub sample: bool,
    pub format: ResponseFormat,
    pub status: u16,
    pub content_type: Option<String>,
    pub body: ResponseBody,
}

pub fn plan_call(request: &CallRequest) -> Result<RequestPlan> {
    let api = find_api(&request.api_id)?;
    validate_query_params(api, &request.query)?;
    validate_selected_fields(request.selected_fields.as_deref(), api)?;

    build_request_plan(
        api,
        request.sample,
        request.format,
        request.auth_key.as_deref(),
        request.query.clone(),
    )
}

pub fn execute_call(request: &CallRequest) -> Result<CallResponse> {
    let plan = plan_call(request)?;
    let response = execute_request(&plan)?;

    Ok(build_call_response(
        request.api_id.clone(),
        request.sample,
        request.format,
        response,
        request.selected_fields.as_deref(),
    ))
}

fn build_call_response(
    api_id: String,
    sample: bool,
    format: ResponseFormat,
    response: ResponseEnvelope,
    selected_fields: Option<&[String]>,
) -> CallResponse {
    let body = match format {
        ResponseFormat::Json => match serde_json::from_str(&response.body) {
            Ok(body) => ResponseBody::Json(filter_body_fields(body, selected_fields)),
            Err(_) => ResponseBody::Text(response.body),
        },
        ResponseFormat::Xml => ResponseBody::Text(response.body),
    };

    CallResponse {
        api_id,
        sample,
        format,
        status: response.status,
        content_type: response.content_type,
        body,
    }
}

fn filter_body_fields(body: Value, fields: Option<&[String]>) -> Value {
    let Some(fields) = fields else {
        return body;
    };

    let allowed = fields
        .iter()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    filter_json_value(body, &allowed)
}

fn filter_json_value(value: Value, allowed: &std::collections::BTreeSet<&str>) -> Value {
    match value {
        Value::Array(values) => Value::Array(
            values
                .into_iter()
                .map(|value| filter_array_item(value, allowed))
                .collect(),
        ),
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .map(|(key, value)| (key, filter_json_value(value, allowed)))
                .collect(),
        ),
        other => other,
    }
}

fn filter_array_item(value: Value, allowed: &std::collections::BTreeSet<&str>) -> Value {
    match value {
        Value::Object(object) if object.keys().any(|key| allowed.contains(key.as_str())) => {
            Value::Object(
                object
                    .into_iter()
                    .filter(|(key, _)| allowed.contains(key.as_str()))
                    .collect(),
            )
        }
        other => filter_json_value(other, allowed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_response(body: &str) -> ResponseEnvelope {
        ResponseEnvelope {
            status: 200,
            content_type: Some("application/json".to_string()),
            body: body.to_string(),
        }
    }

    #[test]
    fn build_call_response_filters_selected_fields_in_json_rows() {
        let response = build_call_response(
            "krx_dd_trd".to_string(),
            true,
            ResponseFormat::Json,
            sample_response(
                r#"{"OutBlock_0":{"result":"ok"},"OutBlock_1":[{"BAS_DD":"20200414","IDX_NM":"KRX 100","MKTCAP":"1000"}]}"#,
            ),
            Some(&["BAS_DD".to_string(), "IDX_NM".to_string()]),
        );

        assert_eq!(
            response.body,
            ResponseBody::Json(serde_json::json!({
                "OutBlock_0": { "result": "ok" },
                "OutBlock_1": [{ "BAS_DD": "20200414", "IDX_NM": "KRX 100" }]
            }))
        );
    }

    #[test]
    fn build_call_response_keeps_xml_as_text_body() {
        let response = build_call_response(
            "krx_dd_trd".to_string(),
            true,
            ResponseFormat::Xml,
            ResponseEnvelope {
                status: 200,
                content_type: Some("application/xml".to_string()),
                body: "<root />".to_string(),
            },
            None,
        );

        assert_eq!(response.body, ResponseBody::Text("<root />".to_string()));
    }

    #[test]
    fn build_call_response_returns_text_when_json_body_is_invalid() {
        let response = build_call_response(
            "krx_dd_trd".to_string(),
            true,
            ResponseFormat::Json,
            ResponseEnvelope {
                status: 200,
                content_type: Some("application/json".to_string()),
                body: "not-json".to_string(),
            },
            None,
        );

        assert_eq!(response.body, ResponseBody::Text("not-json".to_string()));
    }
}
