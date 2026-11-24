//! Root-free original Native coordinates captured from an actual admitted basis.
pub mod capture;
pub mod successor;
use super::{
    copying, RecordedRelationalComponentReference, RelationalRecordedComponentRefusal as Refusal,
};
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::branch::basis::RelationalBranchBasisDescriptor;
use crate::history::data::CanonicalCommitEnvelope;
use crate::runtime::state::subsystems::services::sequence::origin::RelationalRuntimeOrigin;
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
pub enum MemoryComponentOrigin {
    Live(MemoryComponentMeaning),
    Interim(InterimComponentMeaning),
}
#[derive(Debug)]
pub struct InterimComponentMeaning {
    owner: RelationalRuntimeOrigin,
    description: RecordedRelationalComponentReference,
    canonical:
        Option<crate::durability::checkpoint_admission::admitted_commit::CheckpointAdmittedCommit>,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
#[derive(Debug)]
pub struct MemoryComponentMeaning {
    pub owner: RelationalRuntimeOrigin,
    pub descriptor: RelationalBranchBasisDescriptor,
    pub canonical: Option<Weak<CanonicalCommitEnvelope>>,
    pub _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl ObservedRelationalMemoryComponent {
    pub fn matches_original(
        &self,
        original: &crate::durability::authority::associated_effects::observed_memory_commit::ObservedRelationalMemoryCommit,
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
pub fn map_interim_admission_refusal(
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

/// Operations used by the Relational owners.
pub trait ObservedRelationalMemoryComponentOwnerPort: Sized {
    fn from_interim(
        request: &ExecutionRequest<'_, '_>,
        owner: RelationalRuntimeOrigin,
        description: RecordedRelationalComponentReference,
        canonical: Option<
            crate::durability::checkpoint_admission::admitted_commit::CheckpointAdmittedCommit,
        >,
    ) -> Result<
        Self,
        crate::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal,
    >;
}

impl ObservedRelationalMemoryComponentOwnerPort for ObservedRelationalMemoryComponent {
    fn from_interim(
        request: &ExecutionRequest<'_, '_>,
        owner: RelationalRuntimeOrigin,
        description: RecordedRelationalComponentReference,
        canonical: Option<
            crate::durability::checkpoint_admission::admitted_commit::CheckpointAdmittedCommit,
        >,
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
}

pub trait ObservedRelationalMemoryComponentFieldsPort {
    fn owner_original(&self) -> &Arc<MemoryComponentOrigin>;
}

impl ObservedRelationalMemoryComponentFieldsPort for ObservedRelationalMemoryComponent {
    fn owner_original(&self) -> &Arc<MemoryComponentOrigin> {
        &self.original
    }
}

pub fn assemble_observed_relational_memory_component(
    original: Arc<MemoryComponentOrigin>,
) -> ObservedRelationalMemoryComponent {
    ObservedRelationalMemoryComponent { original }
}
