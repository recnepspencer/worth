//! A committed original intent lends one World reservation, never a new attempt.
use super::*;
use worth_foundational::facade::{
    CompositeAttemptAssociation, HomeProgramTransitionPlanDescription,
    HomeProgramTransitionStepAssociation,
};
use worth_runtime_world::facade::{ProductBranchObservation, WorldHomeTransitionStepReservation};

pub struct AdmittedApplicationHomeProgramStep {
    plan: HomeProgramTransitionPlanDescription,
    original: CompositeAttemptAssociation,
    permission: worth_proof::AdmittedHomeProgramTransitionStep<
        worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity,
    >,
    reservation: Option<WorldHomeTransitionStepReservation>,
    execution: Option<worth_proof::AdmittedHomeProgramTransitionExecution>,
    _names: Option<worth_execution::ExecutionMemoryReservation>,
}
impl AdmittedApplicationHomeProgramStep {
    pub fn take_execution(
        &mut self,
    ) -> Option<worth_proof::AdmittedHomeProgramTransitionExecution> {
        self.execution.take()
    }
    pub(super) fn with_execution(
        mut self,
        execution: worth_proof::AdmittedHomeProgramTransitionExecution,
    ) -> Self {
        self.execution = Some(execution);
        self
    }
    pub fn plan(&self) -> &HomeProgramTransitionPlanDescription {
        &self.plan
    }
    pub fn original(&self) -> &CompositeAttemptAssociation {
        &self.original
    }
    pub fn admission(
        &self,
    ) -> &worth_proof::AdmittedHomeProgramTransitionStep<
        worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity,
    > {
        &self.permission
    }
    pub fn take_reservation(&mut self) -> Option<WorldHomeTransitionStepReservation> {
        self.reservation.take()
    }
}

pub(crate) fn admit(
    context: &crate::home_context::HomeContext,
    step: u32,
    expected: &ProductBranchObservation,
    request: &ExecutionRequest<'_, '_>,
) -> Result<AdmittedApplicationHomeProgramStep, ApplicationHomeTransitionIntentDenial> {
    use ApplicationHomeTransitionIntentDenial as Denial;
    request.consult().map_err(Denial::Execution)?;
    let publication = context
        .transition
        .publication()
        .ok_or(Denial::IntentNotCommitted)?;
    if !matches!(
        publication.observation(),
        Some(ApplicationHomeTransitionIntentOutcome::Committed)
    ) {
        return Err(Denial::IntentNotCommitted);
    }
    let intent = &publication.intent;
    let original = intent
        .assignments()
        .get(step as usize)
        .ok_or(Denial::AssignmentCount)?;
    super::step_source::require_current_owner(
        publication.reserved.world.predecessor().owner_identity(),
        expected,
    )?;
    super::step_source::require_original_source(intent, step, expected)?;
    if expected.branch_identity() != publication.reserved.world.predecessor().branch_identity()
        || (step == 0 && expected != publication.reserved.world.predecessor())
    {
        return Err(Denial::SourceMismatch);
    }
    let step = HomeProgramTransitionStepAssociation::from_recorded(intent.plan().identity(), step);
    let commitment = intent
        .plan()
        .step_commitment(step, original)
        .map_err(|_| Denial::SourceMismatch)?;
    let prepared = PreparedStepPermission::prepare(intent, original, request)?;
    let reservation = publication
        .reserved
        .world
        .take_step(step.step())
        .ok_or(Denial::StepAlreadyClaimed)?;
    let caller_association = reservation.caller_association_commitment();
    Ok(prepared.issue(
        context,
        commitment,
        expected
            .basis()
            .relational_basis()
            .admission_identity()
            .clone(),
        caller_association,
        Some(reservation),
    ))
}

pub(super) struct PreparedStepPermission {
    plan: HomeProgramTransitionPlanDescription,
    original: CompositeAttemptAssociation,
    memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl PreparedStepPermission {
    pub(super) fn prepare(
        intent: &HomeProgramTransitionIntent,
        original: &CompositeAttemptAssociation,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, ApplicationHomeTransitionIntentDenial> {
        use ApplicationHomeTransitionIntentDenial as Denial;
        let names = intent
            .plan()
            .predecessor()
            .branch()
            .name()
            .len()
            .checked_add(original.branch().name().len())
            .ok_or(Denial::SizeLimit)?;
        let memory = request
            .reserve_retained_memory(names as u64)
            .map_err(Denial::Execution)?;
        let plan = intent.plan().clone();
        let original = original.clone();
        Ok(Self {
            plan,
            original,
            memory,
        })
    }

    pub(super) fn issue(
        self,
        context: &crate::home_context::HomeContext,
        commitment: [u8; 32],
        source: worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity,
        caller_association: [u8; 32],
        reservation: Option<WorldHomeTransitionStepReservation>,
    ) -> AdmittedApplicationHomeProgramStep {
        AdmittedApplicationHomeProgramStep {
            plan: self.plan,
            original: self.original,
            reservation,
            execution: None,
            permission: context
                .transition
                .admit_step(commitment, caller_association, source),
            _names: self.memory,
        }
    }
}
