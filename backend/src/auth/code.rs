//! Verification + reset code generation.
//!
//! 6-digit numeric strings, zero-padded so `042100` is just as valid as `942100`.
//! Generated with `OsRng` for cryptographic suitability.

use rand::{rngs::OsRng, Rng};

pub fn six_digit_code() -> String {
    let n: u32 = OsRng.gen_range(0..1_000_000);
    format!("{:06}", n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn always_six_chars() {
        for _ in 0..100 {
            let c = six_digit_code();
            assert_eq!(c.len(), 6, "code {c} must be 6 chars");
            assert!(c.chars().all(|ch| ch.is_ascii_digit()));
        }
    }
}
