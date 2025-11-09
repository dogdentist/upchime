use std::collections::HashMap;

use anyhow::anyhow;
use base64::Engine;
use reqwest::{
    Client, Method,
    header::{HeaderName, HeaderValue},
    redirect,
};
use serde::Deserialize;

use crate::constants;
pub enum PingResponse {
    Down((u64, u16)),
    Up((u64, u16)),
    Timeout,
}

#[derive(Debug, Deserialize)]
pub struct HttpMetadata {
    #[serde(rename = "m")]
    method: String,
    #[serde(rename = "b")]
    body: Option<String>,
    #[serde(rename = "mx")]
    successful_max: u16,
    #[serde(rename = "mi")]
    successful_min: u16,
    #[serde(rename = "i")]
    insecure: bool,
    // None means no redirects
    #[serde(rename = "r")]
    follow_redirects: Option<usize>,
    #[serde(rename = "h")]
    headers: Option<HashMap<String, String>>,
    #[serde(rename = "t")]
    timeout: Option<i32>,
}

fn build_request(metadata: &HttpMetadata) -> anyhow::Result<Client> {
    let redirect_policy = if let Some(v) = metadata.follow_redirects {
        if v > 0 {
            redirect::Policy::limited(v)
        } else {
            redirect::Policy::none()
        }
    } else {
        redirect::Policy::none()
    };

    let client = if metadata.insecure {
        reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
    } else {
        reqwest::ClientBuilder::new()
    };

    Ok(client
        .redirect(redirect_policy)
        .timeout(tokio::time::Duration::from_secs(
            if let Some(timeout) = metadata.timeout {
                timeout as u64
            } else {
                constants::HTTP_PING_TIMOUET
            },
        ))
        .user_agent(constants::USER_AGENT)
        .build()?)
}

pub async fn ping(target: &str, metadata: &HttpMetadata) -> anyhow::Result<PingResponse> {
    let method = Method::from_bytes(metadata.method.as_bytes())
        .map_err(|e| anyhow!("invalid method was provided for request, {}", e))?;
    let client = build_request(metadata)?;

    let mut req = if let Some(ref body) = metadata.body {
        let body = base64::prelude::BASE64_STANDARD
            .decode(body)
            .map_err(|e| anyhow!("invalid request body, {}", e))?;

        client.request(method, target).body(body)
    } else {
        client.request(method, target)
    };

    if let Some(ref headers) = metadata.headers {
        for (n, v) in headers {
            let n = HeaderName::from_bytes(n.as_bytes())
                .map_err(|e| anyhow!("invalid header name '{n}', {}", e))?;
            let v = HeaderValue::from_bytes(v.as_bytes())
                .map_err(|e| anyhow!("invalid header value '{v}', {}", e))?;

            req = req.header(n, v);
        }
    }

    let time_start = chrono::Utc::now().timestamp_millis();

    match req.send().await {
        Ok(res) => {
            let ping_duration = chrono::Utc::now().timestamp_millis() - time_start;
            let status = res.status().as_u16();

            if status >= metadata.successful_min && status <= metadata.successful_max {
                Ok(PingResponse::Up((ping_duration as u64, status)))
            } else {
                Ok(PingResponse::Down((ping_duration as u64, status)))
            }
        }
        Err(e) => {
            if e.is_timeout() {
                Ok(PingResponse::Timeout)
            } else {
                Err(anyhow!("request failed, {}", e))
            }
        }
    }
}
