//! Live reservation of original World attempts for one complete Home plan.
use worth_execution::{ExecutionArray, ExecutionMemoryReservation};
use worth_foundational::facade::{
    CompositeAttemptAssociation, HomeProgramTransitionPlanDescription,
};

/// The Home intent may describe these assignments, but only World issues them.
/// The actual operation keeps owner close from retiring a still-publishing plan.
#[derive(Debug)]
pub struct ReservedWorldHomeTransitionAssignments {
    pub(crate) description: HomeProgramTransitionPlanDescription,
    pub(crate) assignments: ExecutionArray<CompositeAttemptAssociation>,
    pub(crate) owner: crate::identity::RuntimeWorldOwnerIdentity,
    pub(crate) operation: crate::lifecycle::owner::RuntimeWorldOperationReservation,
    pub(crate) _names_memory: Option<ExecutionMemoryReservation>,
}

impl ReservedWorldHomeTransitionAssignments {
    pub fn description(&self) -> &HomeProgramTransitionPlanDescription {
        &self.description
    }
    pub fn assignments(&self) -> &[CompositeAttemptAssociation] {
        self.assignments.elements()
    }
}

#[derive(Debug)]
pub enum WorldHomeTransitionReservationRefusal {
    NotHomeBound,
    UnboundTransitionOwner,
    Admission(worth_proof::HomeProgramTransitionAdmissionDenial),
    Source(crate::publication::NoEffectCompositePublication),
    SourceMismatch,
    Identity(crate::identity::RuntimeWorldIdentityExhaustion),
    Request(worth_execution::WorkCeilingDenial),
    Backing(worth_execution::ExecutionRequestArrayDenial),
    CapacityOverflow,
}
