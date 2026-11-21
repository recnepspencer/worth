//! Sealed test doors over this module's private members.
use super::*;

impl ComputationFactAttribution {
    #[doc(hidden)]
    pub fn normalize_adjacencies_for_test(
        &mut self,
        handler: &mut BTreeSet<Key>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Cause> {
        Self::normalize_adjacencies(self, handler, execution)
    }
}
