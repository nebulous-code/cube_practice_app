//! Resend REST client.
//!
//! Single endpoint we use today — `POST https://api.resend.com/emails`.
//! Response is ignored on success; failures bubble up as `AppError::Internal`
//! so the calling handler logs the underlying issue but doesn't leak it to
//! the client.

use serde::Serialize;

use crate::error::{AppError, AppResult};

const ENDPOINT: &str = "https://api.resend.com/emails";

#[derive(Clone)]
pub struct ResendClient {
    api_key: String,
    from: String,
    http: reqwest::Client,
}

impl ResendClient {
    pub fn new(api_key: String, from: String) -> Self {
        Self {
            api_key,
            from,
            http: reqwest::Client::new(),
        }
    }

    /// `to` is a single recipient address — we don't multi-send anywhere.
    /// `text` is plain text; `html` is the HTML alternate.
    pub async fn send(&self, to: &str, subject: &str, text: &str, html: &str) -> AppResult<()> {
        if self.api_key.is_empty() {
            tracing::warn!(
                to = %to,
                subject = %subject,
                "RESEND_API_KEY unset — would have sent email"
            );
            return Ok(());
        }

        let body = ResendBody {
            from: &self.from,
            to: [to],
            subject,
            text,
            html,
        };

        let resp = self
            .http
            .post(ENDPOINT)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("resend request: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp
                .text()
                .await
                .unwrap_or_else(|_| "<could not read body>".into());
            return Err(AppError::Internal(format!(
                "resend returned {status}: {body}"
            )));
        }

        Ok(())
    }
}

#[derive(Serialize)]
struct ResendBody<'a> {
    from: &'a str,
    to: [&'a str; 1],
    subject: &'a str,
    text: &'a str,
    html: &'a str,
}
