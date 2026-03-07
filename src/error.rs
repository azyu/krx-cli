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

    #[error("could not resolve home directory for ~/.config/krx")]
    HomeDirNotFound,

    #[error("io failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("json parse failed: {0}")]
    Json(#[from] serde_json::Error),
}
