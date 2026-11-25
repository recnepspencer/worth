//! The same original World open brackets Query's one Native preparation.
use super::durable_custody::{
    QueryDurablePreparationDenial as Cause, QueryDurableWorldCustody as World,
    QueryPreparingApplicationCommit, RetainedQueryDurablePreparation,
};
use super::*;
use crate::domain_computation::primary_graph::provider::QueryDurablePreparationGuard;
use crate::domain_computation::{
    WorthQueryProductPreparationPending as Pending, WorthQueryProductPreparationRefusal as Refusal,
    WorthQueryProductPreparationRefused as Refused, WorthQueryProductPreparationStage as Stage,
    WorthQueryProviderSessionCommitStop as Stop,
};
use worth_runtime_world::facade::WorldOpenAttemptRefusal;

pub(super) fn refused_before_open(
    query: QueryPreparingApplicationCommit,
    cause: WorldOpenAttemptRefusal,
) -> Stop {
    if let WorldOpenAttemptRefusal::Reservation(no_effect) = cause {
        use worth_execution::{MapKernelStop, WorkCeilingDenial};
        use worth_runtime_world::facade::NoEffectCause;
        match no_effect.cause() {
            NoEffectCause::StaleExpectedProductHead
            | NoEffectCause::CancelledBeforeEffect
            | NoEffectCause::DeadlineBeforeEffect
            | NoEffectCause::ExecutionRequest(WorkCeilingDenial::Stopped(
                MapKernelStop::Cancelled | MapKernelStop::DeadlineElapsed,
            )) => return world_stop::world_no_effect(no_effect),
            _ => {
                return Stop::ProductPreparationRefused(
                    worth_query_execution_runtime::facade::ports::OpaqueProductPublicationOutcome::new(
                        Refused::new(
                            Refusal::World(WorldOpenAttemptRefusal::Reservation(no_effect)),
                            Box::new(query),
                        ),
                        false,
                    ),
                );
            }
        }
    }
    Stop::ProductPreparationRefused(
        worth_query_execution_runtime::facade::ports::OpaqueProductPublicationOutcome::new(
            Refused::new(Refusal::World(cause), Box::new(query)),
            false,
        ),
    )
}
pub(super) fn commit(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    query: QueryPreparingApplicationCommit,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<ApplicationCommitExecution, Stop> {
    if query.members.is_empty()
        && query.required_prerequisites.is_some()
        && !query.attempt.has_nonrelational_completion_effects()
        && matches!(
            &query.native,
            durable_custody::QueryNativeCustody::Proposal(proposal)
                if proposal.proposal_identity().omits_publication_appendix()
        )
    {
        return prepare_omitted_appendix_adoption(execution, provider, query, allocation_policy);
    }
    let mut guard = opening::prepare(execution, provider, query)?;
    let outcome = progression::run(execution, provider, &mut guard, allocation_policy);
    completion::finish(provider, guard, outcome).map(ApplicationCommitExecution::Committed)
}

fn prepare_omitted_appendix_adoption(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    mut query: QueryPreparingApplicationCommit,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<ApplicationCommitExecution, Stop> {
    if let Err(cause) = native_work::prepare_native_candidate(&execution, provider, &mut query) {
        return Err(native_work::memory_stop(cause, &query.attempt));
    }
    if let Err(cause) =
        native_work::prepare_candidate_support(&execution, provider, &mut query, allocation_policy)
    {
        return Err(native_work::memory_stop(cause, &query.attempt));
    }
    let durable_custody::QueryPreparingApplicationCommit {
        attempt,
        native,
        before,
        required_prerequisites,
        prepared_output_witness,
        source_fact_rebase,
        ..
    } = query;
    let durable_custody::QueryNativeCustody::Candidate(candidate) = native else {
        unreachable!("Native preparation completed once")
    };
    let source_fact_rebase = source_fact_rebase.expect("candidate source facts prepared");
    match executed_adoption::prepare(
        &execution,
        provider,
        &attempt,
        &candidate,
        prepared_output_witness.is_some(),
        source_fact_rebase,
        required_prerequisites,
    )? {
        executed_adoption::AdoptionPreparation::Selected {
            selected,
            adoption,
            association,
            rebase,
            required,
        } => Ok(ApplicationCommitExecution::AdoptionPrepared(Box::new(
            executed_adoption::PreparedExecutedOutputAdoption::new(
                attempt,
                candidate,
                selected,
                adoption,
                association,
                before,
                rebase,
                required,
                prepared_output_witness.expect("adoption retains its real prepared witness"),
            ),
        ))),
        executed_adoption::AdoptionPreparation::Declined { .. } => Err(Stop::PreEffectDenied(
            failure("a body without its publication appendix requires current-output adoption"),
        )),
    }
}
pub mod completion;
pub mod opening;
pub mod progression;
