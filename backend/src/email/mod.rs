//! Outbound email — Resend REST wrapper + the three templates we send.
//!
//! Templates per `docs/milestones/01_auth_and_accounts.md` §5. Subject lines
//! and copy are kept here so they can be reviewed without reading code.

mod resend;
mod templates;

pub use resend::ResendClient;
pub use templates::{email_change_verification, password_reset, verification};
