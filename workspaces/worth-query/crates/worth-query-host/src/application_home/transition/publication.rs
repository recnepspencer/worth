//! Execute each original positioned World open once through Native's compiler.
use super::super::runtime::DirectoryHome;
use super::{plan::*, refusal::WorthQueryHomeProgramTransitionDenial as Denial};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    HomeProgramTransitionStepAssociation, HomeProgramTransitionStepRole,
};
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_query_execution::facade::application_home_integration::{
    PreparedExistingHomeSignalProgram, PreparedHomeProgramTransitionTarget,
};
use worth_relational::facade::{mvcc::RelationalCallerAssociation, runtime::*};
use worth_runtime_world::facade::*;

pub(super) fn publish<
    A: ApplicationSchemaComposition,
    AP,
    Schema: ApplicationSchemaComposition,
    Program,
>(
    home: &mut DirectoryHome,
    target: &PreparedHomeProgramTransitionTarget<Schema, Program>,
    predecessor: &PreparedExistingHomeSignalProgram<A, AP>,
    plan: WorthQueryHomeProgramTransitionPlan,
    budget: worth_relational::facade::indexes::DerivedIndexMaintenanceBudget,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    let cancellation = RuntimeWorldCancellationSource::new();
    let WorthQueryHomeProgramTransitionPlan {
        steps,
        target: target_position,
        schema_transition,
    } = plan;
    for (position, step) in steps.into_iter().enumerate() {
        let position = u32::try_from(position).map_err(|_| Denial::InvalidPlan)?;
        let intent = match (&schema_transition, position == target_position) {
            (Some((schema, policy)), true) => {
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary()
                    .with_schema_transition(schema.clone(), policy.clone())
            }
            (Some(_), false) | (None, true) | (None, false) => {
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary()
            }
        };
        let source = home
            .world()
            .observation_port()
            .observe_root_product_branch()
            .map_err(Denial::Observation)?;
        let program_source = target
            .admit_home_transition_program_source(
                predecessor,
                home.relational().runtime(),
                &source,
                &home.branch_program_source_owner(),
                request,
            )
            .map_err(Denial::Author)?;
        let mut grant = home
            .admit_home_program_transition_step(position, &source, request)
            .map_err(Denial::Intent)?;
        let reservation = grant.take_reservation().ok_or(Denial::InvalidPlan)?;
        let pending = home
            .world()
            .publication_port()
            .prepare_home_transition_step(
                request,
                source.clone(),
                program_source,
                intent,
                grant.plan(),
                grant.original(),
                reservation,
                grant.admission(),
                &cancellation.token(),
                None,
            )
            .map_err(|(reservation, cause)| Denial::OpenPreparation { reservation, cause })?;
        let mut opened = pending.advance(request);
        let opened = loop {
            match opened {
                WorldOpenAttemptOutcome::Opened(opened) => break opened,
                WorldOpenAttemptOutcome::Pending(pending) => {
                    if let Err(cause) = request.consult() {
                        return Err(Denial::StoppedOpen { cause, pending });
                    }
                    std::thread::yield_now();
                    opened = pending.advance(request);
                }
                outcome @ (WorldOpenAttemptOutcome::Unresolved(_)
                | WorldOpenAttemptOutcome::EvidenceRequired(_)
                | WorldOpenAttemptOutcome::Refused(_)) => return Err(Denial::Open(outcome)),
            }
        };
        let native = home.relational().runtime();
        let association =
            RelationalCallerAssociation::from_bytes(opened.caller_association_bytes())
                .expect("World emits its canonical bounded association");
        let role = match position.cmp(&target_position) {
            std::cmp::Ordering::Less => HomeProgramTransitionStepRole::BeforeTarget,
            std::cmp::Ordering::Equal => HomeProgramTransitionStepRole::Target,
            std::cmp::Ordering::Greater => HomeProgramTransitionStepRole::AfterTarget,
        };
        let selected = match role {
            HomeProgramTransitionStepRole::BeforeTarget => {
                RelationalHomeProgramStepContracts::BeforeTarget
            }
            HomeProgramTransitionStepRole::Target => RelationalHomeProgramStepContracts::Target {
                contracts: target.native_contracts(),
                transition: schema_transition.clone(),
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
                HomeProgramTransitionStepAssociation::from_recorded(
                    grant.plan().identity(),
                    position,
                ),
                grant.original(),
                &association,
                grant.admission(),
            )
            .map_err(Denial::Native)?;
        for batch in step.batches {
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
        let mut candidate = native
            .prepare_validated_proposal(request, proposal)
            .map_err(Denial::Commit)?;
        target
            .prepare_home_program_transition_indexes(request, native, &mut candidate, role, budget)
            .map_err(Denial::Indexes)?;
        let prepared = opened
            .complete_query_native_preparation(candidate, request)
            .map_err(Denial::NativePreparation)?;
        let mut outcome = home
            .world()
            .publication_port()
            .execute_durable_without_signal(*request, prepared, &cancellation.token());
        let performed = loop {
            match outcome {
                WorldDurablePublicationOutcome::Performed(performed) => break performed,
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
        };
        home.accept_home_program_transition_step(performed, request)
            .map_err(Denial::Completion)?;
    }
    Ok(())
}
