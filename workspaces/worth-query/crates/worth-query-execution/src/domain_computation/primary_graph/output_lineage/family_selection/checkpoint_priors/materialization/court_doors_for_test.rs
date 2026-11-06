//! Sealed test doors over this module's private members.
use super::*;

impl NativePriorCheckpointOutput<'_> {
    #[doc(hidden)]
    pub fn materialize_identity_for_test(
        &self,
    ) -> Result<Option<WorthQueryAcceptedOutputCheckpointIdentity>, Denial> {
        Self::materialize_identity(self)
    }
}
