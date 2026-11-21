//! A recorded pending transition continues from its first nondurable step.
use super::super::runtime::DirectoryHome;
use super::{
    refusal::WorthQueryHomeProgramTransitionDenial as Denial,
    step_execution::{self, HomeProgramTransitionSteps},
};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    HomeProgramTransitionBodyDigest, HomeProgramTransitionPlanDigest,
    HomeProgramTransitionStepAssociation, HomeProgramTransitionTargetMeaning,
};
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_query_execution::facade::application_home_integration::{
    PreparedExistingHomeSignalProgram, PreparedHomeProgramTransitionTarget,
};
use worth_runtime_world::facade::*;
use worth_store_application::{
    AdmittedApplicationHomeProgramStep, ApplicationHomeProgramTransitionResumption as Resumption,
};

/// The recomputed plan's recorded meaning. Store compares it with the intent
/// before every step it admits, so drift refuses before any new effect.
pub(super) struct RecomputedHomeProgramTransitionPlan {
    pub(super) digest: HomeProgramTransitionPlanDigest,
    pub(super) steps: u32,
    pub(super) target: HomeProgramTransitionTargetMeaning,
    pub(super) bodies: Vec<HomeProgramTransitionBodyDigest>,
}

/// Durable steps are skipped; a recovered original is continued or resumed;
/// an unopened original is opened once. Returns when every step is durable.
pub(super) fn resume<
    A: ApplicationSchemaComposition,
    AP,
    Schema: ApplicationSchemaComposition,
    Program,
>(
    home: &mut DirectoryHome,
    target: &PreparedHomeProgramTransitionTarget<Schema, Program>,
    predecessor: &PreparedExistingHomeSignalProgram<A, AP>,
    mut steps: HomeProgramTransitionSteps,
    recomputed: &RecomputedHomeProgramTransitionPlan,
    budget: worth_relational::facade::indexes::DerivedIndexMaintenanceBudget,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    let cancellation = RuntimeWorldCancellationSource::new();
    loop {
        request.consult().map_err(Denial::Request)?;
        let source = home
            .world()
            .observation_port()
            .observe_root_product_branch()
            .map_err(Denial::Observation)?;
        let resumption = home
            .resume_home_program_transition(
                &source,
                recomputed.digest,
                recomputed.steps,
                recomputed.target,
                &recomputed.bodies,
                request,
            )
            .map_err(Denial::Resumption)?;
        let performed = match resumption {
            Resumption::StepsDurable => return Ok(()),
            Resumption::Unopened { step, mut grant } => {
                let execution = grant.take_execution().ok_or(Denial::InvalidPlan)?;
                let program_source = target
                    .admit_home_transition_program_source(
                        predecessor,
                        home.relational().runtime(),
                        &source,
                        &home.branch_program_source_owner(),
                        request,
                    )
                    .map_err(Denial::Author)?;
                let pending = home
                    .world()
                    .publication_port()
                    .prepare_readmitted_home_transition_step(
                        request,
                        source.clone(),
                        program_source,
                        steps.change(step),
                        grant.plan(),
                        step,
                        grant.original(),
                        grant.admission(),
                        execution,
                        &cancellation.token(),
                        None,
                    )
                    .map_err(|(_, cause)| Denial::UnopenedStep(cause))?;
                let opened = step_execution::opened(pending.advance(request), request)?;
                step_execution::execute(
                    home,
                    target,
                    &mut steps,
                    step,
                    &source,
                    &grant,
                    opened,
                    budget,
                    request,
                    &cancellation,
                )?
            }
            Resumption::Recovered { step, grant, claim } => match claim.native_receipt() {
                // World holds the durable Native effect: continue that original.
                Some(_) => {
                    continue_original(home, &grant, step, claim, request, &cancellation)?;
                    continue;
                }
                // Only the World open is durable: resume it, then run the body once.
                None => {
                    let opened = home
                        .world()
                        .publication_port()
                        .resume_home_transition_open(
                            request,
                            grant.plan(),
                            HomeProgramTransitionStepAssociation::from_recorded(
                                grant.plan().identity(),
                                step,
                            ),
                            grant.original(),
                            grant.admission(),
                            claim,
                            steps.change(step),
                            &cancellation.token(),
                            None,
                        )
                        .map_err(|refused| Denial::ResumedOpen(Box::new(refused)))?;
                    step_execution::execute(
                        home,
                        target,
                        &mut steps,
                        step,
                        &source,
                        &grant,
                        opened,
                        budget,
                        request,
                        &cancellation,
                    )?
                }
            },
        };
        home.accept_home_program_transition_step(performed, request)
            .map_err(Denial::Completion)?;
    }
}

/// World advances the recovered original occurrence; nothing executes again.
fn continue_original(
    home: &DirectoryHome,
    grant: &AdmittedApplicationHomeProgramStep,
    step: u32,
    mut claim: WorldRecoveredUnpublishedEffects,
    request: &ExecutionRequest<'_, '_>,
    cancellation: &RuntimeWorldCancellationSource,
) -> Result<(), Denial> {
    let port = home.world().publication_port();
    loop {
        match port.continue_home_transition_effect(
            request,
            grant.plan(),
            HomeProgramTransitionStepAssociation::from_recorded(grant.plan().identity(), step),
            grant.original(),
            grant.admission(),
            claim,
            &cancellation.token(),
            None,
        ) {
            WorldHomeTransitionEffectOutcome::Continued(publication) => {
                drop(publication);
                return Ok(());
            }
            WorldHomeTransitionEffectOutcome::Pending(pending) => {
                if let Err(cause) = request.consult() {
                    return Err(Denial::StoppedContinuation {
                        cause,
                        custody: pending,
                    });
                }
                std::thread::yield_now();
                claim = pending;
            }
            outcome @ (WorldHomeTransitionEffectOutcome::EvidenceRequired(_)
            | WorldHomeTransitionEffectOutcome::Indeterminate(_)
            | WorldHomeTransitionEffectOutcome::ReopenRequired(_)
            | WorldHomeTransitionEffectOutcome::Refused { .. }) => {
                return Err(Denial::Continuation(Box::new(outcome)))
            }
        }
    }
}
