use std::collections::BTreeMap;
#[cfg(test)]
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::branch::{
    PreparedRelationalBranchRootCapture, RelationalBranchRoot, RelationalBranchRootCaptureDenial,
};
use crate::history::data::CanonicalCommitEnvelope;

use super::HistorySubsystem;

impl HistorySubsystem {
    pub(crate) fn prepare_branch_root_capture<P: crate::storage::overlay::PartitionAccess>(
        &self,
        partitions: &P,
        published_delta: &crate::storage::RelationalPublishedPartitionDelta,
        previous: Option<&Arc<RelationalBranchRoot>>,
        envelope: Arc<CanonicalCommitEnvelope>,
        registry: &crate::schema::data::RelationalSchemaRegistry,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<PreparedRelationalBranchRootCapture, RelationalBranchRootCaptureDenial> {
        #[cfg(test)]
        if self.root_capture_sabotage.swap(false, Ordering::Relaxed) {
            return Err(RelationalBranchRootCaptureDenial::UnresolvedContentSymbol(
                crate::symbols::data::Symbol(u32::MAX),
            ));
        }
        RelationalBranchRoot::prepare_capture(
            &self.root_identity_issuer,
            partitions,
            published_delta,
            previous,
            envelope,
            registry,
            symbols,
        )
    }

    #[cfg(test)]
    pub(crate) fn sabotage_next_root_capture(&self) {
        self.root_capture_sabotage.store(true, Ordering::Relaxed);
    }

    pub(crate) fn readmit_branch_root(
        &self,
        partitions: BTreeMap<
            crate::identity::data::PartitionId,
            crate::storage::overlay::PartitionState,
        >,
        envelope: Arc<CanonicalCommitEnvelope>,
        descriptor: crate::branch::RelationalBranchRootDescriptor,
        schema_authority: Arc<crate::branch::RelationalBranchRootSchemaAuthority>,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<Arc<RelationalBranchRoot>, RelationalBranchRootCaptureDenial> {
        RelationalBranchRoot::readmit(
            &self.root_identity_issuer,
            partitions,
            envelope,
            descriptor,
            schema_authority,
            symbols,
        )
    }
}
