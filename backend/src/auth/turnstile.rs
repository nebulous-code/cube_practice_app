//! Cloudflare Turnstile token verification.
//!
//! `verify` posts to Cloudflare's `siteverify` endpoint and rejects the
//! request when the call fails or the response says `success: false`.
//! Turnstile is pass/fail (no score), so the only knob is the secret.
//!
//! In dev (when `TURNSTILE_SECRET_KEY` is empty), verification is skipped
//! with a warning log so curl-based testing works without a captcha token.

use serde::Deserialize;

use crate::error::{AppError, AppResult};

const SITEVERIFY: &str = "https://challenges.cloudflare.com/turnstile/v0/siteverify";

#[derive(Clone)]
pub struct TurnstileVerifier {
    secret: String,
    http: reqwest::Client,
}

impl TurnstileVerifier {
    pub fn new(secret: String) -> Self {
        Self {
            secret,
            http: reqwest::Client::new(),
        }
    }

    /// Verify a Turnstile token. Returns Ok(()) on success.
    pub async fn verify(&self, token: &str) -> AppResult<()> {
        if self.secret.is_empty() {
            tracing::warn!("TURNSTILE_SECRET_KEY unset — skipping captcha check");
            return Ok(());
        }

        let response: SiteverifyResponse = self
            .http
            .post(SITEVERIFY)
            .form(&[("secret", self.secret.as_str()), ("response", token)])
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("turnstile request: {e}")))?
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("turnstile decode: {e}")))?;

        if !response.success {
            tracing::debug!(error_codes = ?response.error_codes, "turnstile rejected");
            return Err(AppError::CaptchaFailed);
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct SiteverifyResponse {
    success: bool,
    #[serde(rename = "error-codes", default)]
    error_codes: Vec<String>,
}
