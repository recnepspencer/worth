//! Typed denials for application decision reads.
use super::{WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind};

pub fn denial(
    kind: WorthQueryApplicationAttemptDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(kind, subject)
}
