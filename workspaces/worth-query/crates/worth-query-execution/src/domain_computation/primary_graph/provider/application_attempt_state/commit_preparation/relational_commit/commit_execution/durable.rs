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
    Stop::ProductPreparationRefused(Refused::new(Refusal::World(cause), query))
}
pub(super) fn commit(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    query: QueryPreparingApplicationCommit,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<WorthQueryCommittedApplicationSession, Stop> {
    let mut guard = opening::prepare(execution, provider, query)?;
    let outcome = progression::run(execution, provider, &mut guard, allocation_policy);
    completion::finish(provider, guard, outcome)
}
mod completion;
mod opening;
mod progression;
