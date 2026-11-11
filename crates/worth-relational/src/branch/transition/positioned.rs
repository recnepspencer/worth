use super::{RelationalBranchTransition, RelationalBranchTransitionSequence};
use crate::publication::patch::data::PatchStreamPosition;

/// One branch transition at its place in durable history: its own sequence
/// and the performed commit position it follows.
///
/// Only Relational issues one: at the durable step of a live transition, or
/// by readmitting a transition artifact. Holding one confers no authority to
/// change a branch.
#[derive(Debug, Clone)]
pub struct PositionedBranchTransition {
    sequence: RelationalBranchTransitionSequence,
    follows: PatchStreamPosition,
    transition: RelationalBranchTransition,
    schema_carrier: Option<[u8; 32]>,
    caller_association: Option<crate::history::data::RelationalCallerAssociation>,
    recorded_artifact_digest: Option<[u8; 32]>,
    allocation_memory: Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>,
}

impl PositionedBranchTransition {
    pub(crate) fn admit(
        sequence: RelationalBranchTransitionSequence,
        follows: PatchStreamPosition,
        transition: RelationalBranchTransition,
    ) -> Self {
        Self {
            allocation_memory: None,
            schema_carrier: None,
            caller_association: None,
            recorded_artifact_digest: None,
            sequence,
            follows,
            transition,
        }
    }

    pub(crate) fn with_source_memory(
        mut self,
        memory: Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>,
    ) -> Self {
        self.allocation_memory = memory;
        self
    }

    pub fn sequence(&self) -> RelationalBranchTransitionSequence {
        self.sequence
    }

    /// The last performed commit position when the transition was performed.
    /// Zero when no commit preceded it.
    pub fn follows(&self) -> PatchStreamPosition {
        self.follows
    }

    pub fn transition(&self) -> &RelationalBranchTransition {
        &self.transition
    }

    pub fn caller_association(&self) -> Option<&crate::history::data::RelationalCallerAssociation> {
        self.caller_association.as_ref()
    }

    /// Placement retained from the acknowledged original frame or its admitted image.
    pub fn recorded_artifact_digest(&self) -> Option<[u8; 32]> {
        self.recorded_artifact_digest
    }

    pub(crate) fn with_caller_association(
        mut self,
        association: Option<crate::history::data::RelationalCallerAssociation>,
    ) -> Self {
        self.caller_association = association;
        self
    }

    pub(crate) fn with_recorded_artifact_digest(mut self, digest: Option<[u8; 32]>) -> Self {
        self.recorded_artifact_digest = digest;
        self
    }

    pub(crate) fn schema_carrier(&self) -> Option<[u8; 32]> {
        self.schema_carrier
    }

    pub(crate) fn with_schema_carrier(mut self, carrier: Option<[u8; 32]>) -> Self {
        self.schema_carrier = carrier;
        self
    }
}
impl PartialEq for PositionedBranchTransition {
    fn eq(&self, other: &Self) -> bool {
        self.sequence == other.sequence
            && self.follows == other.follows
            && self.transition == other.transition
            && self.schema_carrier == other.schema_carrier
            && self.caller_association == other.caller_association
    }
}
impl Eq for PositionedBranchTransition {}
