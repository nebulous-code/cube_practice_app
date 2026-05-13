//! JWT signing and verification.
//!
//! Tokens are HS256-signed with `JWT_SECRET`, 30-day default expiry. Claims:
//!
//! - `sub` — `users.id`
//! - `sid` — `sessions.id`; the `AuthUser` extractor cross-checks this row
//!   for `revoked = false`, which is what makes logout / sign-out-all
//!   actually take effect before the token expires
//! - `iat`, `exp` — unix timestamps
//!
//! See `docs/milestones/01_auth_and_accounts.md` §4 for the full design.

use jsonwebtoken::{decode, encode, errors::ErrorKind, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// Default token lifetime — matches the spec.
pub const DEFAULT_TTL_DAYS: i64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Claims {
    pub sub: Uuid,
    pub sid: Uuid,
    pub iat: i64,
    pub exp: i64,
}

impl Claims {
    pub fn new(user_id: Uuid, session_id: Uuid, ttl_days: i64) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            sub: user_id,
            sid: session_id,
            iat: now,
            exp: now + ttl_days * 86_400,
        }
    }
}

/// Sign a fresh JWT for the given user + session.
pub fn sign(claims: &Claims, secret: &str) -> AppResult<String> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("jwt encode: {e}")))
}

/// Verify a JWT signature + expiry. Returns the decoded claims on success.
/// Maps signature/expiry failures to `AppError::Unauthorized` so the caller
/// can return a generic 401 without leaking which check failed.
pub fn decode_token(token: &str, secret: &str) -> AppResult<Claims> {
    let mut validation = Validation::default();
    validation.leeway = 0;

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| match e.kind() {
        ErrorKind::ExpiredSignature
        | ErrorKind::InvalidSignature
        | ErrorKind::InvalidToken
        | ErrorKind::InvalidIssuer
        | ErrorKind::InvalidAudience => AppError::Unauthorized,
        _ => {
            tracing::debug!(error = %e, "jwt decode failed");
            AppError::Unauthorized
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "test-secret-not-for-production-use-32+chars";

    fn fresh_claims() -> Claims {
        Claims::new(Uuid::new_v4(), Uuid::new_v4(), DEFAULT_TTL_DAYS)
    }

    #[test]
    fn sign_and_decode_roundtrip() {
        let claims = fresh_claims();
        let token = sign(&claims, SECRET).expect("sign");
        let decoded = decode_token(&token, SECRET).expect("decode");
        assert_eq!(decoded, claims);
    }

    #[test]
    fn wrong_secret_rejected() {
        let token = sign(&fresh_claims(), SECRET).expect("sign");
        let result = decode_token(&token, "different-secret");
        assert!(matches!(result, Err(AppError::Unauthorized)));
    }

    #[test]
    fn expired_token_rejected() {
        let now = chrono::Utc::now().timestamp();
        let claims = Claims {
            sub: Uuid::new_v4(),
            sid: Uuid::new_v4(),
            iat: now - 86_400,
            exp: now - 60,
        };
        let token = sign(&claims, SECRET).expect("sign");
        let result = decode_token(&token, SECRET);
        assert!(matches!(result, Err(AppError::Unauthorized)));
    }

    #[test]
    fn malformed_token_rejected() {
        let result = decode_token("not.a.jwt", SECRET);
        assert!(matches!(result, Err(AppError::Unauthorized)));
    }

    #[test]
    fn ttl_calculation_is_correct() {
        let claims = Claims::new(Uuid::new_v4(), Uuid::new_v4(), 30);
        assert_eq!(claims.exp - claims.iat, 30 * 86_400);
    }
}
