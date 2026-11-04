//! Closed Home preparation retains its exact source before descriptor effects.
use super::RelationalSectionReadmissionDenial as Denial;
use std::sync::Arc;
use worth_execution::{ExecutionArray, ExecutionImmutableBytes, ExecutionRequest};
use worth_relational::facade::durability::{
    RelationalDurableCheckpointSource, RelationalDurableReadBounds, RelationalDurableRecoverySource,
};
use worth_store::physical_runtime::{AnchoredRecordReadMembership, ServingPhysicalRuntime};

pub struct RelationalHomeSourcePreparation {
    source: Arc<crate::cold_source::StoreColdSource>,
    checkpoint: Option<RelationalDurableCheckpointSource>,
}
pub struct PreparedFreshRelationalHomeSource {
    pub(super) source: Arc<crate::cold_source::StoreColdSource>,
}
pub struct PreparedRelationalHomeSource {
    pub(super) source: Arc<crate::cold_source::StoreColdSource>,
    pub(super) original: Box<RelationalDurableRecoverySource>,
    pub(super) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl RelationalHomeSourcePreparation {
    pub fn from_absent_home_read(
        serving: Arc<ServingPhysicalRuntime>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<PreparedFreshRelationalHomeSource, Denial> {
        let source = crate::cold_source::StoreColdSource::new_for_request(serving, request)
            .map_err(Denial::Execution)?;
        Ok(PreparedFreshRelationalHomeSource { source })
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
        Ok(PreparedFreshRelationalHomeSource { source })
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
        let checkpoint = super::readmission::prepare_home_source(&source, selected, request)?;
        Ok(Self { source, checkpoint })
    }
    pub fn coverage(&self) -> [u64; 2] {
        self.checkpoint.as_ref().map_or([0, 0], |source| {
            [source.covered().0, source.covered_transitions().0]
        })
    }
    pub fn finish(
        self,
        initial: Option<ExecutionImmutableBytes>,
        canonical: ExecutionArray<ExecutionImmutableBytes>,
        transitions: ExecutionArray<ExecutionImmutableBytes>,
        bounds: RelationalDurableReadBounds,
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
                    self.checkpoint,
                    initial,
                    canonical,
                    transitions,
                    bounds,
                    self.source.clone(),
                    request,
                )
            })
            .map_err(Denial::Execution)?
            .map_err(Denial::InitialInputs)?;
        Ok(PreparedRelationalHomeSource {
            source: self.source,
            original: Box::new(original),
            _memory: memory,
        })
    }
}
