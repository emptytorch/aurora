use std::str::FromStr;

use crate::validated::HttpMethod;

#[derive(Debug)]
pub struct Request {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

#[derive(Debug)]
pub struct Response {
    pub status: StatusCode,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct StatusCode(u16);

impl From<u16> for StatusCode {
    fn from(value: u16) -> Self {
        StatusCode(value)
    }
}

impl StatusCode {
    pub fn is_success(self) -> bool {
        (200..300).contains(&self.0)
    }
}

impl Response {
    pub fn pretty_body(&self) -> String {
        let content_type = self
            .headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case("Content-Type"))
            .map(|(_, v)| v.as_str())
            .unwrap_or_default();

        let body_str = String::from_utf8_lossy(&self.body);
        if content_type.contains("application/json") {
            return serde_json::from_str::<serde_json::Value>(&body_str)
                .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| body_str.to_string()))
                .unwrap_or_else(|_| body_str.to_string());
        }

        body_str.to_string()
    }
}

#[derive(Debug)]
pub enum HttpError {
    InvalidUrl(String),
    InvalidHeaderName(String),
    InvalidHeaderValue(String),
    Connection(String),
    Timeout,
    Transport(String),
    BodyRead(String),
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::InvalidUrl(url) => write!(f, "invalid URL: `{url}`"),
            HttpError::InvalidHeaderName(name) => write!(f, "invalid header name: `{name}`"),
            HttpError::InvalidHeaderValue(value) => write!(f, "invalid header value: `{value}`"),
            HttpError::Connection(msg) => write!(f, "connection error: {msg}"),
            HttpError::Timeout => write!(f, "request timed out"),
            HttpError::Transport(msg) => write!(f, "transport error: {msg}"),
            HttpError::BodyRead(msg) => write!(f, "failed to read response body: {msg}"),
        }
    }
}

pub trait HttpClient {
    fn send(&self, request: Request) -> Result<Response, HttpError>;
}

pub struct ReqwestHttpClient {
    client: reqwest::blocking::Client,
}

impl ReqwestHttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl HttpClient for ReqwestHttpClient {
    fn send(&self, request: Request) -> Result<Response, HttpError> {
        let mut builder = match request.method {
            HttpMethod::Get => self.client.get(&request.url),
            HttpMethod::Post => self.client.post(&request.url),
            HttpMethod::Put => self.client.put(&request.url),
            HttpMethod::Patch => self.client.patch(&request.url),
            HttpMethod::Delete => self.client.delete(&request.url),
        };

        let mut headers = reqwest::header::HeaderMap::with_capacity(request.headers.len());
        for (k, v) in &request.headers {
            let name = reqwest::header::HeaderName::from_str(k)
                .map_err(|_| HttpError::InvalidHeaderName(k.clone()))?;
            let value = reqwest::header::HeaderValue::from_str(v)
                .map_err(|_| HttpError::InvalidHeaderValue(v.clone()))?;
            headers.insert(name, value);
        }

        builder = builder.headers(headers);
        if let Some(body) = request.body {
            builder = builder.body(body);
        }

        let response = builder.send().map_err(|e| {
            if e.is_timeout() {
                HttpError::Timeout
            } else if e.is_connect() {
                HttpError::Connection(e.to_string())
            } else {
                HttpError::Transport(e.to_string())
            }
        })?;

        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| {
                let value = v.to_str().map_err(|_| {
                    HttpError::InvalidHeaderValue(format!("{}: invalid UTF-8", k.to_string()))
                })?;
                Ok((k.to_string(), value.to_string()))
            })
            .collect::<Result<Vec<_>, HttpError>>()?;

        let status = StatusCode::from(response.status().as_u16());
        let body = response
            .bytes()
            .map_err(|e| HttpError::BodyRead(e.to_string()))?
            .to_vec();

        Ok(Response {
            status,
            headers,
            body,
        })
    }
}
