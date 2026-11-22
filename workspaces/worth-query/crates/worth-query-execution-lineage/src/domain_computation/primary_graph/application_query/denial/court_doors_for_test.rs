//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationQueryAdmissionDenial {
    #[doc(hidden)]
    pub fn new_for_test(
        kind: WorthQueryApplicationQueryAdmissionDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self::new(kind, subject)
    }
}
