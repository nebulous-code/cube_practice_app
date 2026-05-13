//! Argon2id password hashing wrapper.
//!
//! Parameters are read once at startup from the environment so they can be tuned
//! per-deployment without a code change. Defaults are OWASP-recommended and match
//! what the spec calls for in `docs/Cube_Practice_Design_Doc.md` §5.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy)]
pub struct Argon2Config {
    pub m_kib: u32,
    pub t: u32,
    pub p: u32,
}

impl Argon2Config {
    pub const DEFAULT_M_KIB: u32 = 19_456;
    pub const DEFAULT_T: u32 = 2;
    pub const DEFAULT_P: u32 = 1;

    pub fn from_env() -> Self {
        Self {
            m_kib: std::env::var("ARGON2_M_KIB")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(Self::DEFAULT_M_KIB),
            t: std::env::var("ARGON2_T")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(Self::DEFAULT_T),
            p: std::env::var("ARGON2_P")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(Self::DEFAULT_P),
        }
    }

    fn argon2(&self) -> AppResult<Argon2<'static>> {
        let params = Params::new(self.m_kib, self.t, self.p, None)
            .map_err(|e| AppError::Internal(format!("argon2 params: {e}")))?;
        Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
    }
}

/// Hash a plaintext password. Returns the PHC-encoded hash string suitable for
/// `users.password_hash`. Salt is generated fresh per call.
pub fn hash_password(password: &str, config: Argon2Config) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = config.argon2()?;
    let phc = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("argon2 hash: {e}")))?;
    Ok(phc.to_string())
}

/// Verify a plaintext against an argon2 PHC hash. Returns Ok(true) if the password
/// matches, Ok(false) if it doesn't. Returns an error only if the stored hash is
/// malformed (which would indicate a bug or DB tampering).
pub fn verify_password(password: &str, phc_hash: &str) -> AppResult<bool> {
    let parsed = PasswordHash::new(phc_hash)
        .map_err(|e| AppError::Internal(format!("argon2 parse: {e}")))?;
    match Argon2::default().verify_password(password.as_bytes(), &parsed) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(AppError::Internal(format!("argon2 verify: {e}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lighter params for tests — full OWASP defaults make `cargo test` slow.
    fn fast_config() -> Argon2Config {
        Argon2Config {
            m_kib: 8 * 1024,
            t: 1,
            p: 1,
        }
    }

    #[test]
    fn roundtrip_succeeds() {
        let hash = hash_password("hunter2-correct-horse", fast_config()).expect("hash");
        assert!(verify_password("hunter2-correct-horse", &hash).expect("verify"));
    }

    #[test]
    fn wrong_password_rejected() {
        let hash = hash_password("hunter2-correct-horse", fast_config()).expect("hash");
        assert!(!verify_password("hunter2-wrong", &hash).expect("verify"));
    }

    #[test]
    fn each_hash_uses_a_unique_salt() {
        let h1 = hash_password("same-password", fast_config()).expect("hash");
        let h2 = hash_password("same-password", fast_config()).expect("hash");
        assert_ne!(h1, h2, "salts must differ");
        assert!(verify_password("same-password", &h1).expect("verify h1"));
        assert!(verify_password("same-password", &h2).expect("verify h2"));
    }

    #[test]
    fn malformed_hash_returns_internal_error() {
        let result = verify_password("anything", "not-a-real-hash");
        assert!(matches!(result, Err(AppError::Internal(_))));
    }
}
