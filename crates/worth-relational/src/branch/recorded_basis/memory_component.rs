//! Root-free original Native coordinates captured from an actual admitted basis.
mod capture;
mod capture_runtime;
mod live;
mod successor;
mod successor_runtime;
use super::{
    copying, RecordedRelationalComponentReference, RelationalRecordedComponentRefusal as Refusal,
};
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::branch::basis::RelationalBranchBasisDescriptor;
use crate::history::data::CanonicalCommitEnvelope;
use crate::runtime::RelationalRuntimeState;
use std::sync::{Arc, Weak};
pub use successor::AdmittedRelationalMemoryComponent;
use worth_execution::ExecutionRequest;

/// Actual memory-owner input, with no root, runtime service or live result pin.
/// Deserialized descriptions cannot construct this observation.
#[derive(Debug, Clone)]
pub struct ObservedRelationalMemoryComponent {
    original: Arc<MemoryComponentOrigin>,
}
#[derive(Debug)]
enum MemoryComponentOrigin {
    Live(MemoryComponentMeaning),
    Interim(InterimComponentMeaning),
}
#[derive(Debug)]
struct InterimComponentMeaning {
    owner: Weak<RelationalRuntimeState>,
    description: RecordedRelationalComponentReference,
    canonical: Option<crate::durability::checkpoint_admission::CheckpointAdmittedCommit>,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
#[derive(Debug)]
struct MemoryComponentMeaning {
    owner: Weak<RelationalRuntimeState>,
    descriptor: RelationalBranchBasisDescriptor,
    canonical: Option<Weak<CanonicalCommitEnvelope>>,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl ObservedRelationalMemoryComponent {
    pub(crate) fn from_interim(
        request: &ExecutionRequest<'_, '_>,
        owner: Weak<RelationalRuntimeState>,
        description: RecordedRelationalComponentReference,
        canonical: Option<crate::durability::checkpoint_admission::CheckpointAdmittedCommit>,
    ) -> Result<
        Self,
        crate::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal,
    > {
        use crate::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal as Stop;
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<MemoryComponentOrigin>())
            .map_err(|_| Stop::CapacityOverflow)?
            .0
            .pad_to_align();
        let funding = request
            .reserve_retained_memory(layout.size() as u64)
            .map_err(|cause| Stop::Request(cause.into()))?;
        Ok(Self {
            original: Arc::new(MemoryComponentOrigin::Interim(InterimComponentMeaning {
                owner,
                description,
                canonical,
                _funding: funding,
            })),
        })
    }
    pub fn matches_original(
        &self,
        original: &crate::durability::authority::ObservedRelationalMemoryCommit,
    ) -> bool {
        match self.original.as_ref() {
            MemoryComponentOrigin::Live(meaning) => meaning
                .canonical
                .as_ref()
                .is_some_and(|canonical| original.matches_original_envelope(canonical)),
            MemoryComponentOrigin::Interim(meaning) => meaning
                .canonical
                .as_ref()
                .is_some_and(|canonical| original.matches_interim(&meaning.owner, canonical)),
        }
    }
}
fn map_interim_admission_refusal(
    cause: crate::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal,
) -> Refusal {
    match cause {
        crate::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal::Request(
            stop,
        ) => Refusal::Request(stop),
        other => Refusal::Source(
            crate::durability::data::recovery_stop::RelationalRecoveryStop::AdmissionImage(other),
        ),
    }
}
