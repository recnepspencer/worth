//! Sealed test doors over this module's private members.
use super::*;

impl LogicalPostings {
    #[doc(hidden)]
    pub fn next_for_test(
        &mut self,
        state: &MarkState,
        key: &FactPostingKey,
        counts: &mut WorthQueryNativeMarkingWork,
    ) -> Result<Option<FactPosting>, CompanionPreflightStop> {
        Self::next(self, state, key, counts)
    }
}

impl LogicalPostings {
    #[doc(hidden)]
    pub fn prepare_for_test(
        state: &MarkState,
        key: &FactPostingKey,
        counts: &mut WorthQueryNativeMarkingWork,
    ) -> Result<Option<Self>, CompanionPreflightStop> {
        Self::prepare(state, key, counts)
    }
}
