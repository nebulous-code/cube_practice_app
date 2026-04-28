//! reCAPTCHA v3 token verification.
//!
//! `verify` posts to Google's `siteverify` endpoint and rejects the request
//! when either the call fails server-side, the response says `success: false`,
//! or the score falls below `min_score`.
//!
//! In dev (when `RECAPTCHA_SECRET_KEY` is empty), verification is skipped with
//! a warning log so curl-based testing works without a captcha token.

use serde::Deserialize;

use crate::error::{AppError, AppResult};

const SITEVERIFY: &str = "https://www.google.com/recaptcha/api/siteverify";

#[derive(Clone)]
pub struct RecaptchaVerifier {
    secret: String,
    min_score: f32,
    http: reqwest::Client,
}

impl RecaptchaVerifier {
    pub fn new(secret: String, min_score: f32) -> Self {
        Self {
            secret,
            min_score,
            http: reqwest::Client::new(),
        }
    }

    /// Verify a v3 token. Returns Ok(()) on success.
    pub async fn verify(&self, token: &str) -> AppResult<()> {
        if self.secret.is_empty() {
            tracing::warn!("RECAPTCHA_SECRET_KEY unset — skipping captcha check");
            return Ok(());
        }

        let response: SiteverifyResponse = self
            .http
            .post(SITEVERIFY)
            .form(&[("secret", self.secret.as_str()), ("response", token)])
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("recaptcha request: {e}")))?
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("recaptcha decode: {e}")))?;

        if !response.success {
            tracing::debug!(error_codes = ?response.error_codes, "recaptcha rejected");
            return Err(AppError::RecaptchaFailed);
        }
        if response.score.unwrap_or(0.0) < self.min_score {
            tracing::debug!(
                score = response.score,
                threshold = self.min_score,
                "recaptcha score below threshold"
            );
            return Err(AppError::RecaptchaFailed);
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct SiteverifyResponse {
    success: bool,
    score: Option<f32>,
    #[serde(rename = "error-codes", default)]
    error_codes: Vec<String>,
}
