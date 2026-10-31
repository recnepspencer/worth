use std::sync::Arc;
use worth_relational::facade::runtime::{
    RelationalRuntime, RelationalRuntimeConstructionDenial, RelationalRuntimeDefinition,
};
use worth_store::physical_runtime::{AdmittedRecordPlacementPolicy, ServingPhysicalRuntime};

/// One actual native runtime attached to its two ordered journals.
/// This component carries no application, World or Query recovery proof.
pub struct RelationalStoreSection {
    pub(super) runtime: Arc<RelationalRuntime>,
    pub(super) cold_source: Arc<crate::cold_source::StoreColdSource>,
    pub(super) serving: Arc<ServingPhysicalRuntime>,
    pub(super) checkpoint_selection: Arc<dyn super::RelationalCheckpointSelectionSource>,
}

impl RelationalStoreSection {
    /// Attach the medium to the resolved definition exactly once. Readmission
    /// remains mandatory before composition may mint its next recovery stage.
    pub fn attach(
        definition: RelationalRuntimeDefinition,
        serving: Arc<ServingPhysicalRuntime>,
        placement: AdmittedRecordPlacementPolicy,
        checkpoint_selection: Arc<dyn super::RelationalCheckpointSelectionSource>,
    ) -> Result<Self, RelationalRuntimeConstructionDenial> {
        let cold_source = Arc::new(crate::cold_source::StoreColdSource::new(serving.clone()));
        Self::attach_with_source(
            definition,
            serving,
            placement,
            cold_source,
            checkpoint_selection,
        )
    }

    pub(super) fn attach_with_source(
        definition: RelationalRuntimeDefinition,
        serving: Arc<ServingPhysicalRuntime>,
        placement: AdmittedRecordPlacementPolicy,
        cold_source: Arc<crate::cold_source::StoreColdSource>,
        checkpoint_selection: Arc<dyn super::RelationalCheckpointSelectionSource>,
    ) -> Result<Self, RelationalRuntimeConstructionDenial> {
        let medium = crate::journal_medium::StoreJournalMedium::new(
            serving.clone(),
            placement,
            cold_source.clone(),
            checkpoint_selection.clone(),
        );
        let runtime = definition.build_durable(Box::new(medium))?;
        Ok(Self {
            runtime: Arc::new(runtime),
            cold_source,
            serving,
            checkpoint_selection,
        })
    }

    pub fn runtime(&self) -> &RelationalRuntime {
        &self.runtime
    }

    /// Closed Home composition access to the SAME owner. Ordinary callers
    /// retain borrowed runtime access and cannot extract lifecycle ownership.
    pub fn world_composition_runtime(&self) -> Arc<RelationalRuntime> {
        Arc::clone(&self.runtime)
    }

    pub fn cold_storage_reads(&self) -> crate::RelationalStoreColdReadCounters {
        self.cold_source.counters()
    }

    pub fn resolve(
        &self,
        position: worth_relational::facade::publication::PatchStreamPosition,
        digest: worth_relational::facade::durability::RelationalCanonicalCommitArtifactDigest,
    ) -> Result<crate::RelationalStoreCommitFate, crate::RelationalStoreJournalRefusal> {
        crate::resolution::resolve(&self.serving, position, digest)
    }

    pub fn resolve_branch_transition(
        &self,
        sequence: worth_relational::facade::branch::RelationalBranchTransitionSequence,
        digest: worth_relational::facade::durability::RelationalBranchTransitionArtifactDigest,
    ) -> Result<crate::RelationalStoreTransitionFate, crate::RelationalStoreTransitionJournalRefusal>
    {
        crate::transition_resolution::resolve(&self.serving, sequence, digest)
    }
}
