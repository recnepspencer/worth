//! Live reservation of original World attempts for one complete Home plan.
use worth_execution::{ExecutionArray, ExecutionMemoryReservation};
use worth_foundational::facade::{
    CompositeAttemptAssociation, HomeProgramTransitionPlanDescription,
    HomeProgramTransitionStepAssociation,
};

/// Descriptive canonical affinity, without permission to execute an owner.
pub fn home_transition_caller_association_commitment(
    request: &worth_execution::ExecutionRequest<'_, '_>,
    plan: &HomeProgramTransitionPlanDescription,
    step: HomeProgramTransitionStepAssociation,
    original: &CompositeAttemptAssociation,
) -> Result<[u8; 32], crate::durability::WorldDurableMediumDenial> {
    use sha2::{Digest, Sha256};
    let bytes =
        crate::durability::encode_home_transition_association(request, plan, step, original)?;
    Ok(Sha256::digest(bytes.elements()).into())
}

/// Only the verified Home receiver supplies an original assigned open.
pub(crate) struct HomeTransitionOpenAssignment {
    pub(crate) attempt: crate::identity::CompositePublicationAttemptIdentity,
    pub(crate) association: ExecutionArray<u8>,
    pub(crate) original_program_revision: [u8; 32],
}

/// The Home intent may describe these assignments, but only World issues them.
/// The actual operation keeps owner close from retiring a still-publishing plan.
#[derive(Debug)]
pub struct ReservedWorldHomeTransitionAssignments {
    pub(crate) description: HomeProgramTransitionPlanDescription,
    pub(crate) assignments: ExecutionArray<CompositeAttemptAssociation>,
    pub(crate) predecessor: crate::branch::ProductBranchObservation,
    pub(crate) _operation: crate::lifecycle::owner::RuntimeWorldOperationReservation,
    pub(crate) _names_memory: Option<ExecutionMemoryReservation>,
    pub(crate) steps: ExecutionArray<std::sync::Mutex<Option<WorldHomeTransitionStepReservation>>>,
}

/// One World-issued attempt can enter the original durable-open path once.
/// The token is not a declaration that its intent or any step is durable.
#[derive(Debug)]
pub struct WorldHomeTransitionStepReservation {
    pub(crate) attempt: crate::identity::CompositePublicationAttemptIdentity,
    pub(crate) plan: [u8; 32],
    pub(crate) step: u32,
    pub(crate) caller_association: [u8; 32],
}
impl WorldHomeTransitionStepReservation {
    pub fn caller_association_commitment(&self) -> [u8; 32] {
        self.caller_association
    }
}

impl ReservedWorldHomeTransitionAssignments {
    pub fn description(&self) -> &HomeProgramTransitionPlanDescription {
        &self.description
    }
    pub fn assignments(&self) -> &[CompositeAttemptAssociation] {
        self.assignments.elements()
    }
    pub fn predecessor(&self) -> &crate::branch::ProductBranchObservation {
        &self.predecessor
    }
    pub fn take_step(&self, step: u32) -> Option<WorldHomeTransitionStepReservation> {
        self.steps
            .elements()
            .get(step as usize)?
            .lock()
            .unwrap_or_else(|cause| cause.into_inner())
            .take()
    }
}

#[derive(Debug)]
pub enum WorldHomeTransitionReservationRefusal {
    Association(crate::durability::WorldDurableMediumDenial),
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
