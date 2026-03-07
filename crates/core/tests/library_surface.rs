use std::collections::BTreeMap;

use krx_core::runtime::{CallRequest, ResponseFormat, plan_call};

fn sample_request() -> CallRequest {
    CallRequest {
        api_id: "krx_dd_trd".to_string(),
        sample: true,
        format: ResponseFormat::Json,
        auth_key: None,
        query: BTreeMap::from([("basDd".to_string(), "20240131".to_string())]),
        selected_fields: None,
    }
}

#[test]
fn plan_call_uses_sample_endpoint_without_clap() {
    let plan = plan_call(&sample_request()).expect("plan should be built");

    assert!(plan.url.contains("/svc/sample/apis/idx/krx_dd_trd.json"));
    assert_eq!(plan.query.get("basDd"), Some(&"20240131".to_string()));
}

#[test]
fn plan_call_rejects_unknown_query_field_without_clap() {
    let mut request = sample_request();
    request.query = BTreeMap::from([("foo".to_string(), "bar".to_string())]);

    let error = plan_call(&request).expect_err("unknown fields should be rejected");
    assert!(error.to_string().contains("only supports basDd"));
}

#[test]
fn plan_call_rejects_unknown_selected_field_without_clap() {
    let mut request = sample_request();
    request.selected_fields = Some(vec!["UNKNOWN".to_string()]);

    let error = plan_call(&request).expect_err("unknown fields should be rejected");
    assert!(error.to_string().contains("unknown output field"));
}
