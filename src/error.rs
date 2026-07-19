use serde::Deserialize;
use thiserror::Error;

/// Errors returned by [`crate::OpenAiClient`].
#[derive(Debug, Error)]
pub enum OpenAiError {
    /// Transport level failure (connection, TLS, timeout, ...).
    #[error("http transport error: {0}")]
    Http(#[from] reqwest::Error),

    /// The API responded with a non-success status code.
    #[error("OpenAI API error (status {status}): {message}")]
    Api {
        /// HTTP status code (e.g. 401, 429).
        status: u16,
        /// Human-readable description from the API.
        message: String,
        /// Error class, e.g. `"invalid_request_error"`.
        error_type: Option<String>,
        /// Machine-readable code, e.g. `"invalid_api_key"`.
        code: Option<String>,
        /// The request parameter at fault, if any.
        param: Option<String>,
    },

    /// The API responded with a body that could not be deserialized.
    #[error("failed to parse API response: {error} (body: {body})")]
    Parse {
        /// The deserialization error.
        error: String,
        /// The offending body, truncated to 2 KiB.
        body: String,
    },

    /// Client side configuration problem (e.g. missing API key).
    #[error("configuration error: {0}")]
    Config(String),

    /// A local file could not be read.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
}

impl OpenAiError {
    pub(crate) fn parse(error: impl ToString, body: &str) -> Self {
        const MAX_BODY: usize = 2048;
        let body = if body.len() > MAX_BODY {
            let mut end = MAX_BODY;
            while !body.is_char_boundary(end) {
                end -= 1;
            }
            format!("{}...", &body[..end])
        } else {
            body.to_string()
        };
        OpenAiError::Parse {
            error: error.to_string(),
            body,
        }
    }
}

/// Error payload the API returns as `{"error": {...}}`.
#[derive(Debug, Deserialize)]
pub(crate) struct ApiErrorEnvelope {
    pub(crate) error: ApiErrorBody,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ApiErrorBody {
    #[serde(default)]
    pub(crate) message: String,
    #[serde(rename = "type")]
    pub(crate) error_type: Option<String>,
    pub(crate) code: Option<String>,
    pub(crate) param: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_api_error_envelope() {
        let body = r#"{"error":{"message":"Invalid API key","type":"invalid_request_error","code":"invalid_api_key","param":null}}"#;
        let envelope: ApiErrorEnvelope = serde_json::from_str(body).unwrap();
        assert_eq!(envelope.error.message, "Invalid API key");
        assert_eq!(
            envelope.error.error_type.as_deref(),
            Some("invalid_request_error")
        );
        assert_eq!(envelope.error.code.as_deref(), Some("invalid_api_key"));
        assert_eq!(envelope.error.param, None);
    }

    #[test]
    fn parse_error_truncates_long_bodies() {
        let body = "x".repeat(5000);
        let err = OpenAiError::parse("bad json", &body);
        match err {
            OpenAiError::Parse { body, .. } => assert!(body.len() < 3000),
            _ => panic!("expected Parse variant"),
        }
    }
}
