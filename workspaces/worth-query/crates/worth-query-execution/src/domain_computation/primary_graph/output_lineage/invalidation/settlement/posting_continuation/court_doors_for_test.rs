//! Sealed test doors over this module's private members.
use super::*;

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
