use super::{WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind};

pub(super) fn canonical_key(
    value: String,
    subject: &str,
) -> Result<String, WorthQueryApplicationAttemptDenial> {
    if value.is_empty()
        || value.trim() != value
        || value.len() > 512
        || value.chars().any(char::is_control)
    {
        Err(denial(
            WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
            subject,
        ))
    } else {
        Ok(value)
    }
}

pub(super) fn denial(
    kind: WorthQueryApplicationAttemptDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(kind, subject)
}
