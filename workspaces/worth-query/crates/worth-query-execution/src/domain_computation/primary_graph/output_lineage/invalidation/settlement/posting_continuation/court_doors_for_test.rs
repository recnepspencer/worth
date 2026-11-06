//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::StoreDenial;

impl PostingRebaseBasis {
    #[doc(hidden)]
    pub fn seal_for_test(
        self,
        body: FactPayload,
        after: PositionedRelationalSnapshot,
        control: StorageControl<'_, '_>,
    ) -> Result<PostingContinuation, StoreDenial> {
        Self::seal(self, body, after, control)
    }
}
