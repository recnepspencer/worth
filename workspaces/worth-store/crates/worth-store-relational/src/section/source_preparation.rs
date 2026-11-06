//! Closed Home preparation retains its exact source before descriptor effects.
use super::RelationalSectionReadmissionDenial as Denial;
use std::sync::Arc;
use worth_execution::{ExecutionArray, ExecutionImmutableBytes, ExecutionRequest};
use worth_relational::facade::durability::{
    RelationalDurableReadBounds, RelationalDurableRecoverySource,
};
use worth_store::physical_runtime::{AnchoredRecordReadMembership, ServingPhysicalRuntime};

pub struct RelationalHomeSourcePreparation {
    source: Arc<crate::cold_source::StoreColdSource>,
    selected: Option<super::admission_section::DecodedHomeAdmission>,
}
pub struct PreparedFreshRelationalHomeSource {
    pub(super) source: Arc<crate::cold_source::StoreColdSource>,
    pub(super) transition: Option<worth_proof::HomeProgramTransitionAuthority>,
    pub(super) transition_required: Option<worth_proof::HomeProgramTransitionRequired>,
}
pub struct PreparedRelationalHomeSource {
    pub(super) world_selection: Option<
        Arc<
            worth_proof::VerifiedHomeWorldSelection<
                Arc<worth_relational::facade::durability::RelationalWorldSourceCatalog>,
            >,
        >,
    >,
    pub(super) transition: Option<worth_proof::HomeProgramTransitionAuthority>,
    pub(super) transition_required: Option<worth_proof::HomeProgramTransitionRequired>,
    pub(super) source: Arc<crate::cold_source::StoreColdSource>,
    pub(super) original: Box<RelationalDurableRecoverySource>,
    pub(super) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl PreparedFreshRelationalHomeSource {
    /// The consuming Home composition attaches its actual verifier before
    /// construction exposes any Native receiver or initial-installation port.
    pub fn with_home_transition_authority(
        mut self,
        authority: worth_proof::HomeProgramTransitionAuthority,
        required: Option<worth_proof::HomeProgramTransitionRequired>,
    ) -> Self {
        self.transition = Some(authority);
        self.transition_required = required;
        self
    }
}
impl PreparedRelationalHomeSource {
    pub fn with_home_world_selection(
        mut self,
        selection: Option<
            Arc<
                worth_proof::VerifiedHomeWorldSelection<
                    Arc<worth_relational::facade::durability::RelationalWorldSourceCatalog>,
                >,
            >,
        >,
    ) -> Self {
        self.world_selection = selection;
        self
    }

    pub fn with_home_transition_authority(
        mut self,
        authority: worth_proof::HomeProgramTransitionAuthority,
        required: Option<worth_proof::HomeProgramTransitionRequired>,
    ) -> Self {
        self.transition = Some(authority);
        self.transition_required = required;
        self
    }
}
impl RelationalHomeSourcePreparation {
    pub fn from_absent_home_read(
        serving: Arc<ServingPhysicalRuntime>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<PreparedFreshRelationalHomeSource, Denial> {
        let source = crate::cold_source::StoreColdSource::new_for_request(serving, request)
            .map_err(Denial::Execution)?;
        Ok(PreparedFreshRelationalHomeSource {
            source,
            transition: None,
            transition_required: None,
        })
    }
    #[cfg(feature = "certification-test-authority")]
    pub fn from_absent_home_read_with_read_observer_for_test(
        serving: Arc<ServingPhysicalRuntime>,
        observer: Box<
            dyn Fn(
                    worth_relational::facade::publication::PatchStreamPosition,
                    crate::RelationalStoreColdReadCounters,
                ) + Send
                + Sync,
        >,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<PreparedFreshRelationalHomeSource, Denial> {
        let source = crate::cold_source::StoreColdSource::new_with_owned_observer_for_request(
            serving, observer, request,
        )
        .map_err(Denial::Execution)?;
        Ok(PreparedFreshRelationalHomeSource {
            source,
            transition: None,
            transition_required: None,
        })
    }
    pub fn from_original_home_read(
        serving: Arc<ServingPhysicalRuntime>,
        selected: Option<(
            ExecutionImmutableBytes,
            std::ops::Range<usize>,
            &AnchoredRecordReadMembership,
        )>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let source = crate::cold_source::StoreColdSource::new_for_request(serving, request)
            .map_err(Denial::Execution)?;
        Self::prepare(source, selected, request)
    }
    #[cfg(feature = "certification-test-authority")]
    pub fn from_original_home_read_with_read_observer_for_test(
        serving: Arc<ServingPhysicalRuntime>,
        selected: Option<(
            ExecutionImmutableBytes,
            std::ops::Range<usize>,
            &AnchoredRecordReadMembership,
        )>,
        request: &ExecutionRequest<'_, '_>,
        observer: Box<
            dyn Fn(
                    worth_relational::facade::publication::PatchStreamPosition,
                    crate::RelationalStoreColdReadCounters,
                ) + Send
                + Sync,
        >,
    ) -> Result<Self, Denial> {
        let source = crate::cold_source::StoreColdSource::new_with_owned_observer_for_request(
            serving, observer, request,
        )
        .map_err(Denial::Execution)?;
        Self::prepare(source, selected, request)
    }
    fn prepare(
        source: Arc<crate::cold_source::StoreColdSource>,
        selected: Option<(
            ExecutionImmutableBytes,
            std::ops::Range<usize>,
            &AnchoredRecordReadMembership,
        )>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let selected = super::readmission::prepare_home_source(&source, selected, request)?;
        Ok(Self { source, selected })
    }
    /// DATA from this same selected package; World owns its companion admission.
    pub fn admission_image(
        &self,
    ) -> Option<&worth_relational::facade::durability::RelationalCheckpointAdmissionArtifact> {
        self.selected.as_ref().map(|selected| &selected.image)
    }
    pub fn world_admission_companion(&self) -> Option<&[u8]> {
        self.selected
            .as_ref()
            .map(|selected| &selected.backing[selected.companion.clone()])
    }
    pub fn coverage(&self) -> [u64; 2] {
        self.selected
            .as_ref()
            .map(|selected| &selected.checkpoint)
            .map_or([0, 0], |source| {
                [source.covered().0, source.covered_transitions().0]
            })
    }
    pub fn finish(
        self,
        initial: Option<ExecutionImmutableBytes>,
        canonical: ExecutionArray<ExecutionImmutableBytes>,
        transitions: ExecutionArray<ExecutionImmutableBytes>,
        bounds: RelationalDurableReadBounds,
        world_selection: Option<
            Arc<worth_relational::facade::durability::RelationalWorldSourceCatalog>,
        >,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<PreparedRelationalHomeSource, Denial> {
        let covered = self.coverage()[0];
        for (offset, _) in canonical.iter().enumerate() {
            self.source.observe_original_commit_transport(
                worth_relational::facade::publication::PatchStreamPosition(
                    covered + offset as u64 + 1,
                ),
            );
        }
        let memory = request
            .reserve_retained_memory(std::mem::size_of::<RelationalDurableRecoverySource>() as u64)
            .map_err(Denial::Execution)?;
        let original = request
            .in_scope(|_| {
                RelationalDurableRecoverySource::from_home_original_bytes(
                    self.selected.map(|selected| selected.checkpoint),
                    initial,
                    canonical,
                    transitions,
                    bounds,
                    self.source.clone(),
                    world_selection,
                    request,
                )
            })
            .map_err(Denial::Execution)?
            .map_err(Denial::InitialInputs)?;
        Ok(PreparedRelationalHomeSource {
            world_selection: None,
            transition: None,
            transition_required: None,
            source: self.source,
            original: Box::new(original),
            _memory: memory,
        })
    }
}
