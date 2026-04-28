//! Cookie helpers for the session cookie.
//!
//! Single source of truth for cookie attributes — keep set/clear in sync so
//! the browser actually drops the cookie when we want it to (a clear with
//! mismatched attributes is silently ignored).
//!
//! Path = `/` and SameSite=Strict per `docs/Cube_Practice_Design_Doc.md` §5.

use axum_extra::extract::cookie::{Cookie, SameSite};

use crate::auth::extractor::SESSION_COOKIE;

/// Build a Set-Cookie for a fresh session. 30-day max-age matches the JWT.
pub fn session_cookie(token: String) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, token))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(time::Duration::days(30))
        .build()
}

/// Build a Set-Cookie that tells the browser to drop the session cookie.
/// Attributes (including SameSite + Secure + Path) must match `session_cookie`
/// or browsers will silently ignore the clear.
pub fn clear_session_cookie() -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, ""))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(time::Duration::ZERO)
        .build()
}
