//! Root-free original Native coordinates captured from an actual admitted basis.
pub mod capture;
pub mod successor;
use super::{copying, RelationalRecordedComponentRefusal as Refusal};
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::branch::basis::RelationalBranchBasisDescriptor;
use crate::history::data::CanonicalCommitEnvelope;
use crate::runtime::state::subsystems::services::sequence::origin::RelationalRuntimeOrigin;
use std::sync::{Arc, Weak};
pub use successor::AdmittedRelationalMemoryComponent;
use worth_execution::ExecutionRequest;

trait MemoryComponentOriginBacking: std::fmt::Debug + Send + Sync {
    fn as_origin(&self) -> &MemoryComponentOrigin;
}

impl MemoryComponentOriginBacking for MemoryComponentOrigin {
    fn as_origin(&self) -> &MemoryComponentOrigin {
        self
    }
}

/// Actual memory-owner input, with no root, runtime service or live result pin.
/// Deserialized descriptions cannot construct this observation.
#[derive(Debug, Clone)]
pub struct ObservedRelationalMemoryComponent {
    original: Arc<dyn MemoryComponentOriginBacking>,
}
#[derive(Debug)]
pub enum MemoryComponentOrigin {
    Live(MemoryComponentMeaning),
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
        let MemoryComponentOrigin::Live(meaning) = self.original.as_origin();
        meaning
            .canonical
            .as_ref()
            .is_some_and(|canonical| original.matches_original_envelope(canonical))
    }
}
pub trait ObservedRelationalMemoryComponentFieldsPort {
    fn owner_original(&self) -> &MemoryComponentOrigin;
}

impl ObservedRelationalMemoryComponentFieldsPort for ObservedRelationalMemoryComponent {
    fn owner_original(&self) -> &MemoryComponentOrigin {
        self.original.as_origin()
    }
}

pub fn assemble_observed_relational_memory_component(
    original: Arc<MemoryComponentOrigin>,
) -> ObservedRelationalMemoryComponent {
    ObservedRelationalMemoryComponent { original }
}
