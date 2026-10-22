//! Canonical comparison distinguishes encoding failure from refused buffer custody.
use super::OracleMismatch;

pub(super) enum ComparisonStop {
    Encoding(OracleMismatch),
    Refused,
}
impl From<OracleMismatch> for ComparisonStop {
    fn from(cause: OracleMismatch) -> Self {
        Self::Encoding(cause)
    }
}
impl ComparisonStop {
    pub(super) fn refused() -> Self {
        crate::backend::request_refusal(Self::Refused)
    }

    pub(super) fn finish(self) -> OracleMismatch {
        match self {
            Self::Encoding(cause) => cause,
            Self::Refused => OracleMismatch::Stop,
        }
    }
}
