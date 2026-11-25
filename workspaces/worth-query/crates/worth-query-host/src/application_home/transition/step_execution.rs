//! One opened original step: its Native body under the step's role contracts,
//! then the durable World publication that carries it.
use super::super::runtime::DirectoryHome;
use super::{
    plan::WorthQueryHomeProgramTransitionPlan,
    refusal::WorthQueryHomeProgramTransitionDenial as Denial,
};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    HomeProgramTransitionStepAssociation, HomeProgramTransitionStepRole,
};
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_query_execution::facade::application_home_integration::PreparedHomeProgramTransitionTarget;
use worth_relational::facade::{
    mvcc::{RelationalCallerAssociation, RelationalTransactionIntent},
    runtime::RelationalHomeProgramStepContracts,
    transactions::WorkerIntentBatch,
};
use worth_runtime_world::facade::*;
use worth_store_application::AdmittedApplicationHomeProgramStep;

/// The recomputed plan's steps; each original body is taken at most once.
pub(super) struct HomeProgramTransitionSteps {
    plan: WorthQueryHomeProgramTransitionPlan,
    taken: Vec<bool>,
}
impl HomeProgramTransitionSteps {
    pub(super) fn new(plan: WorthQueryHomeProgramTransitionPlan) -> Self {
        let taken = vec![false; plan.steps.len()];
        Self { plan, taken }
    }
    pub(super) fn count(&self) -> Result<u32, Denial> {
        u32::try_from(self.plan.steps.len()).map_err(|_| Denial::InvalidPlan)
    }
    /// The positioned World change: only the target step carries the schema.
    pub(super) fn change(&self, position: u32) -> RelationalTransactionIntent {
        self.plan.intent(position)
    }
    fn take_body(&mut self, position: u32) -> Result<Vec<WorkerIntentBatch>, Denial> {
        let taken = self
            .taken
            .get_mut(position as usize)
            .filter(|taken| !**taken)
            .ok_or(Denial::InvalidPlan)?;
        *taken = true;
        Ok(std::mem::take(
            &mut self.plan.steps[position as usize].batches,
        ))
    }
}

/// Compile the original body on the step's exact source and durably publish it
/// through its opened original World attempt.
pub(super) fn execute<Schema: ApplicationSchemaComposition, Program>(
    home: &DirectoryHome,
    target: &PreparedHomeProgramTransitionTarget<Schema, Program>,
    steps: &mut HomeProgramTransitionSteps,
    position: u32,
    source: &ProductBranchObservation,
    grant: &AdmittedApplicationHomeProgramStep,
    opened: WorldOpenedPublicationAttempt,
    budget: worth_relational::facade::indexes::DerivedIndexMaintenanceBudget,
    request: &ExecutionRequest<'_, '_>,
    cancellation: &RuntimeWorldCancellationSource,
) -> Result<PerformedCompositePublication, Denial> {
    let native = home.relational().runtime();
    let association = RelationalCallerAssociation::from_bytes(opened.caller_association_bytes())
        .expect("World emits its canonical bounded association");
    let selected = match steps.plan.role(position) {
        HomeProgramTransitionStepRole::BeforeTarget => {
            RelationalHomeProgramStepContracts::BeforeTarget
        }
        HomeProgramTransitionStepRole::Target => RelationalHomeProgramStepContracts::Target {
            contracts: target.native_contracts(),
            transition: steps.plan.schema_transition.clone(),
        },
        HomeProgramTransitionStepRole::AfterTarget => {
            RelationalHomeProgramStepContracts::AfterTarget
        }
    };
    let mut transaction = native
        .begin_home_program_step(
            request,
            source.basis().relational_basis(),
            selected,
            grant.plan(),
            HomeProgramTransitionStepAssociation::from_recorded(grant.plan().identity(), position),
            grant.original(),
            &association,
            grant.admission(),
            budget,
        )
        .map_err(Denial::Native)?;
    for batch in steps.take_body(position)? {
        transaction
            .push_batch(batch, request.allocation_policy())
            .map_err(Denial::Staging)?;
    }
    let proposal = transaction
        .validate(request, native)
        .map_err(Denial::Commit)?;
    let (opened, proposal) = opened
        .bind_query_native_proposal(proposal)
        .map_err(Denial::NativePreparation)?;
    let candidate = native
        .prepare_validated_proposal(request, proposal)
        .map_err(|cause| {
            if let worth_relational::facade::transactions::TransactionCommitError::Publication {
                error,
                ..
            } = &cause
                && let Some(indexes) = error.index_maintenance_denial()
            {
                Denial::Indexes(indexes.clone())
            } else {
                Denial::Commit(cause)
            }
        })?;
    let prepared = opened
        .complete_query_native_preparation(candidate, request)
        .map_err(Denial::NativePreparation)?;
    let mut outcome = home
        .world()
        .publication_port()
        .execute_durable_without_signal(*request, prepared, &cancellation.token());
    loop {
        match outcome {
            WorldDurablePublicationOutcome::Performed(performed) => return Ok(performed),
            WorldDurablePublicationOutcome::Pending(pending) => {
                if let Err(cause) = request.consult() {
                    return Err(Denial::StoppedPublication { cause, pending });
                }
                std::thread::yield_now();
                outcome = pending.advance(request)
            }
            outcome @ (WorldDurablePublicationOutcome::ProductUnpublished(_)
            | WorldDurablePublicationOutcome::RecoveryRequired { .. }) => {
                return Err(Denial::Publication(outcome))
            }
        }
    }
}

/// Advance an original World open until it is durably opened.
pub(super) fn opened(
    mut outcome: WorldOpenAttemptOutcome,
    request: &ExecutionRequest<'_, '_>,
) -> Result<WorldOpenedPublicationAttempt, Denial> {
    loop {
        match outcome {
            WorldOpenAttemptOutcome::Opened(opened) => return Ok(opened),
            WorldOpenAttemptOutcome::Pending(pending) => {
                if let Err(cause) = request.consult() {
                    return Err(Denial::StoppedOpen { cause, pending });
                }
                std::thread::yield_now();
                outcome = pending.advance(request);
            }
            outcome @ (WorldOpenAttemptOutcome::Unresolved(_)
            | WorldOpenAttemptOutcome::EvidenceRequired(_)
            | WorldOpenAttemptOutcome::Refused(_)) => return Err(Denial::Open(outcome)),
        }
    }
}
