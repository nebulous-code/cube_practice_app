//! Email templates — plain text + matching HTML for each of the three messages
//! we send. Copy lifted from `docs/milestones/01_auth_and_accounts.md` §5.

pub struct Email {
    pub subject: String,
    pub text: String,
    pub html: String,
}

const HTML_HEAD: &str = r#"<!doctype html><html><body style="font-family: 'Newsreader', Georgia, serif; background:#F4EFE3; color:#1F1B16; padding:32px;"><div style="max-width:480px; margin:0 auto; background:#FBF7ED; border:1px solid #E5DCC8; border-radius:14px; padding:32px;">"#;
const HTML_TAIL: &str = "</div></body></html>";

fn code_box(code: &str) -> String {
    format!(
        r#"<p style="font-family:'JetBrains Mono', monospace; font-size:32px; letter-spacing:6px; text-align:center; background:#F4EFE3; padding:16px; border-radius:10px; margin:24px 0; color:#1F1B16;">{code}</p>"#
    )
}

pub fn verification(code: &str) -> Email {
    let text = format!(
        "Welcome to Quiet Cube.\n\n\
         Your 6-digit verification code is: {code}\n\n\
         This code expires in 10 minutes. If you didn't sign up, ignore this email."
    );
    let html = format!(
        "{HTML_HEAD}\
         <p style=\"margin:0 0 16px;\">Welcome to Quiet Cube.</p>\
         <p style=\"margin:0;\">Your 6-digit verification code:</p>\
         {code_html}\
         <p style=\"font-size:13px; color:#6E6455; margin:0;\">This code expires in 10 minutes. If you didn't sign up, ignore this email.</p>\
         {HTML_TAIL}",
        code_html = code_box(code)
    );
    Email {
        subject: "Verify your Quiet Cube email".into(),
        text,
        html,
    }
}

pub fn email_change_verification(code: &str, new_email: &str) -> Email {
    let text = format!(
        "You requested to change the email on your Quiet Cube account to {new_email}.\n\n\
         Your 6-digit verification code is: {code}\n\n\
         This code expires in 10 minutes. Until you confirm, sign-in continues to work with your previous email."
    );
    let html = format!(
        "{HTML_HEAD}\
         <p style=\"margin:0 0 16px;\">You requested to change the email on your Quiet Cube account to <strong>{new_email}</strong>.</p>\
         <p style=\"margin:0;\">Your 6-digit verification code:</p>\
         {code_html}\
         <p style=\"font-size:13px; color:#6E6455; margin:0;\">This code expires in 10 minutes. Until you confirm, sign-in continues to work with your previous email.</p>\
         {HTML_TAIL}",
        code_html = code_box(code)
    );
    Email {
        subject: "Confirm your new Quiet Cube email".into(),
        text,
        html,
    }
}

pub fn password_reset(code: &str) -> Email {
    let text = format!(
        "Someone requested a password reset for your Quiet Cube account.\n\n\
         Your 6-digit reset code is: {code}\n\n\
         This code expires in 1 hour. If you didn't request a reset, ignore this email — your password hasn't changed."
    );
    let html = format!(
        "{HTML_HEAD}\
         <p style=\"margin:0 0 16px;\">Someone requested a password reset for your Quiet Cube account.</p>\
         <p style=\"margin:0;\">Your 6-digit reset code:</p>\
         {code_html}\
         <p style=\"font-size:13px; color:#6E6455; margin:0;\">This code expires in 1 hour. If you didn't request a reset, ignore this email — your password hasn't changed.</p>\
         {HTML_TAIL}",
        code_html = code_box(code)
    );
    Email {
        subject: "Reset your Quiet Cube password".into(),
        text,
        html,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verification_includes_code_and_expiry() {
        let e = verification("123456");
        assert!(e.text.contains("123456"));
        assert!(e.text.contains("10 minutes"));
        assert!(e.html.contains("123456"));
        assert_eq!(e.subject, "Verify your Quiet Cube email");
    }

    #[test]
    fn email_change_includes_new_address() {
        let e = email_change_verification("987654", "newuser@example.com");
        assert!(e.text.contains("987654"));
        assert!(e.text.contains("newuser@example.com"));
        assert!(e.html.contains("newuser@example.com"));
    }

    #[test]
    fn password_reset_uses_one_hour_expiry() {
        let e = password_reset("555555");
        assert!(e.text.contains("555555"));
        assert!(e.text.contains("1 hour"));
    }
}
