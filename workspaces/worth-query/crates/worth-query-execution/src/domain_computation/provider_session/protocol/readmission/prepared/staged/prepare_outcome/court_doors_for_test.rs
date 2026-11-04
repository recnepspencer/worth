//! Sealed test doors over this module's private members.
use super::*;

impl<'run> WorthQuerySessionBoundReadsAndEffects<'run> {
    #[doc(hidden)]
    pub fn prepare_for_commit_for_test(
        self,
    ) -> Result<WorthQuerySessionPrepareOutcome<'run>, WorthQueryProviderSessionFailure> {
        Self::prepare_for_commit(self)
    }
}
