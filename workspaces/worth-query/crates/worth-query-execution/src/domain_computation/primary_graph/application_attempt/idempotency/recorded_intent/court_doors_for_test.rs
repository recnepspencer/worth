//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationIdempotencyBinding {
    #[doc(hidden)]
    pub fn intent_text_for_test(self) -> String {
        Self::intent_text(self)
    }
}
