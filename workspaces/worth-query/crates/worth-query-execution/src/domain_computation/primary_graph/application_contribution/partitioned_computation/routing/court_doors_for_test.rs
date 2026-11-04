//! Sealed test doors over this module's private members.
use super::*;

impl ComputationPartitionRouting {
    #[doc(hidden)]
    pub fn route_for_test(
        &mut self,
        item: PartitionItemId,
        digest: [u8; 32],
        admit: impl FnOnce(u64) -> Result<(), WorthQueryManagedComputationResourceDenial>,
    ) -> Result<(PartitionIdentity, PartitionWork), ComputationPartitionRoutingDenial> {
        Self::route(self, item, digest, admit)
    }
}
