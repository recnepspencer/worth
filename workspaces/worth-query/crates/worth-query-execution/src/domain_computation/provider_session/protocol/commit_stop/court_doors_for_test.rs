//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProviderSessionCommitDeferred {
    #[doc(hidden)]
    pub fn new_for_test(
        kind: WorthQueryProviderSessionCommitDeferredKind,
        detail: impl Into<String>,
    ) -> Self {
        Self::new(kind, detail)
    }
}
