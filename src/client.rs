use std::str::FromStr;

use crate::{
    machine::{Request, Response, StatusCode},
    validated::HttpMethod,
};

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
