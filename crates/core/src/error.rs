use thiserror::Error;

pub type Result<T> = std::result::Result<T, KrxCliError>;

#[derive(Debug, Error)]
pub enum KrxCliError {
    #[error("unknown api id: {0}")]
    UnknownApi(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error(
        "real endpoint requires --auth-key, KRX_API_KEY, or ~/.config/krx/config.json auth_key"
    )]
    MissingAuthKey,

    #[error(
        "krx rejected AUTH_KEY. the key itself is not valid for this request. next action: verify the issued key, save it with `config set-auth-key`, and retry"
    )]
    UnauthorizedKey,

    #[error(
        "krx rejected this API call even though the key looks valid. next action: submit API 이용신청 for this service and wait for approval"
    )]
    UnauthorizedApiCall,

    #[error(
        "{}",
        krx_api_error_message(*status, resp_code.as_deref(), resp_msg.as_deref())
    )]
    KrxApiError {
        status: u16,
        resp_code: Option<String>,
        resp_msg: Option<String>,
    },

    #[error("could not resolve home directory for ~/.config/krx")]
    HomeDirNotFound,

    #[error("io failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("json parse failed: {0}")]
    Json(#[from] serde_json::Error),
}

impl KrxCliError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnknownApi(_) => "unknown_api",
            Self::InvalidInput(_) => "invalid_input",
            Self::MissingAuthKey => "missing_auth_key",
            Self::UnauthorizedKey => "unauthorized_key",
            Self::UnauthorizedApiCall => "unauthorized_api_call",
            Self::KrxApiError { .. } => "krx_api_error",
            Self::HomeDirNotFound => "home_dir_not_found",
            Self::Io(_) => "io_failed",
            Self::Http(_) => "http_request_failed",
            Self::Json(_) => "json_parse_failed",
        }
    }
}

fn krx_api_error_message(status: u16, resp_code: Option<&str>, resp_msg: Option<&str>) -> String {
    match (resp_code, resp_msg) {
        (Some(resp_code), Some(resp_msg)) => {
            format!("krx returned HTTP {status} ({resp_code}): {resp_msg}")
        }
        (None, Some(resp_msg)) => format!("krx returned HTTP {status}: {resp_msg}"),
        _ => format!("krx returned HTTP {status} without a structured error body"),
    }
}

#[cfg(test)]
mod tests {
    use super::KrxCliError;

    #[test]
    fn krx_api_error_uses_stable_error_code() {
        let error = KrxCliError::KrxApiError {
            status: 403,
            resp_code: Some("403".to_string()),
            resp_msg: Some("Forbidden".to_string()),
        };

        assert_eq!(error.code(), "krx_api_error");
    }

    #[test]
    fn krx_api_error_formats_structured_message() {
        let error = KrxCliError::KrxApiError {
            status: 500,
            resp_code: Some("500".to_string()),
            resp_msg: Some("Internal Error".to_string()),
        };

        assert_eq!(
            error.to_string(),
            "krx returned HTTP 500 (500): Internal Error"
        );
    }

    #[test]
    fn krx_api_error_formats_unstructured_message() {
        let error = KrxCliError::KrxApiError {
            status: 502,
            resp_code: None,
            resp_msg: None,
        };

        assert_eq!(
            error.to_string(),
            "krx returned HTTP 502 without a structured error body"
        );
    }
}
