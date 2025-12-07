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

pub trait HttpClient {
    fn send(&self, request: Request) -> Response;
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
    fn send(&self, request: Request) -> Response {
        let mut builder = match request.method {
            HttpMethod::Get => self.client.get(&request.url),
            HttpMethod::Post => self.client.post(&request.url),
            HttpMethod::Put => self.client.put(&request.url),
            HttpMethod::Patch => self.client.patch(&request.url),
            HttpMethod::Delete => self.client.delete(&request.url),
        };

        let mut headers = reqwest::header::HeaderMap::with_capacity(request.headers.len());
        for (k, v) in &request.headers {
            headers.insert(
                reqwest::header::HeaderName::from_str(k).unwrap(),
                reqwest::header::HeaderValue::from_str(v).unwrap(),
            );
        }

        builder = builder.headers(headers);
        if let Some(body) = request.body {
            builder = builder.body(body);
        }

        let response = builder.send().unwrap();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string()))
            .collect();

        Response {
            status: StatusCode::from(response.status().as_u16()),
            headers,
            body: response.bytes().unwrap().to_vec(),
        }
    }
}
