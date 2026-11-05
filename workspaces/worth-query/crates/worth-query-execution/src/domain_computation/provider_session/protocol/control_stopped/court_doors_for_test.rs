//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProviderSessionCommitControlStopped {
    #[doc(hidden)]
    pub fn execution_preparation_for_test(
        kind: WorthQueryProviderSessionControlStopKind,
        detail: impl Into<String>,
    ) -> Self {
        Self::execution_preparation(kind, detail)
    }
}
