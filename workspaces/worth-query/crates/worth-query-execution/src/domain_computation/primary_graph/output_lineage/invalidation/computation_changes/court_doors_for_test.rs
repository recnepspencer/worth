//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity;

impl ComputationRegistration {
    #[doc(hidden)]
    pub fn retained_for_test() -> Self {
        Self::retained()
    }
}

impl SourceInvalidationOwner {
    #[doc(hidden)]
    pub fn computation_changes_for_test(
        &self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        identity: &Arc<RecordedSettlementIdentity>,
        facts: &RetainedSourceFacts,
    ) -> Result<Option<ComputationChanges>, CompanionPreflightStop> {
        Self::computation_changes(self, runtime, snapshot, identity, facts)
    }
}

impl ComputationChanges {
    #[doc(hidden)]
    pub fn at_snapshot_for_test(
        &self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Option<&OrdSet<usize>> {
        Self::at_snapshot(self, runtime, snapshot)
    }
}

impl ComputationChanges {
    #[doc(hidden)]
    pub fn bind_original_for_test(
        self,
        original: crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
    ) -> Self {
        Self::bind_original(self, original)
    }
}

impl ComputationChanges {
    #[doc(hidden)]
    pub fn for_snapshot_for_test(
        &self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Option<Self> {
        Self::for_snapshot(self, runtime, snapshot)
    }
}

impl ComputationChanges {
    #[doc(hidden)]
    pub fn ordinals_for_test(&self) -> &OrdSet<usize> {
        Self::ordinals(self)
    }
}

impl ComputationChanges {
    #[doc(hidden)]
    pub fn matches_publication_source_for_test(
        &self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Result<bool, CompanionPreflightStop> {
        Self::matches_publication_source(self, runtime, snapshot)
    }
}
