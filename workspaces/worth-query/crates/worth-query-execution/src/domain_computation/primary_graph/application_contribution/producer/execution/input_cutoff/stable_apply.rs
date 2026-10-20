//! Stable publication occurs only when the canonical owner cursor applies it.
use super::*;

pub(super) fn publish<Schema, Binding, OperationType, Input, Scope>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    observation: &worth_runtime_world::facade::ProductBranchObservation,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    operation: &WorthQueryAdmittedApplicationOperation<Schema, OperationType, Input, Scope>,
    fresh: crate::domain_computation::primary_graph::application_query::BoundStableObservedSourceFacts,
    verified: crate::domain_computation::primary_graph::output_lineage::VerifiedInputCutoff<'_>,
    required_output: RequiredOutputDemandContext,
    resources: super::super::super::WorthQueryProducerDemandResources,
    admission: &mut InvalidationEditAdmission,
) -> Result<PublishedStableLineage, ProducerExecutionStop>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    let scope = operation.operation_scope_binding();
    let handle = &runtime.primary_provider.graph;
    let source_owner = &handle.source_owner.invalidation_owner;
    let lane = runtime
        .primary_provider
        .admitted_application_branch_commit_lane(observation, admission)
        .map_err(|stop| {
            use crate::domain_computation::primary_graph::provider::WorthQueryBranchCommitLaneDenial as Stop;
            match stop {
                Stop::Preparation(stop) => cutoff_admission_denial(stop),
                Stop::RetainedCapacityExhausted { .. } | Stop::CapacityCounterOverflow => denial(
                    WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                    "producer input cutoff branch coordination",
                ).with_recovery_posture(crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable),
            }
        })?;
    let coordination = lane.enter();
    let address = prepare_stable_address::<Operation<Schema, Binding>>(
        &handle.output_lineage,
        verified,
        scope,
        observation,
        &coordination,
        execution,
        admission,
    )?;
    let publication = address.prepare_record(fresh, operation, resources, execution, admission)?;
    // Registry preparation is outside both lineage and Product guards.
    let mut prerequisites = required_output.prepare_prerequisites(
        publication.consumed_outputs().iter(),
        source_owner,
        admission,
    )?;
    prerequisites.reserve_identity(publication.identity(), admission)?;
    let registration = publication.prepare_current_registration(
        source_owner,
        prerequisites.work_membership(),
        admission,
    )
        .map_err(|stop| match stop {
            crate::domain_computation::primary_graph::output_lineage::invalidation::SettlementRegistrationStop::Admission(stop) => cutoff_admission_denial(stop),
            _ => denial(WorthQueryOutputDemandDenialKind::PublicationStale, "producer input cutoff source changed during preparation"),
        })?;
    let displaced = publication.displaced_settlement();
    let published = publication.publish::<Operation<Schema, Binding>, _, _, _, _>(
        operation,
        observation,
        &runtime.product_runtime.owner.observation_port(),
        registration,
        execution, admission,
    ).map_err(|stop| match stop {
        crate::domain_computation::primary_graph::output_lineage::StablePublicationStop::Demand(stop) => ProducerExecutionStop::ExecutionStopped(stop),
        crate::domain_computation::primary_graph::output_lineage::StablePublicationStop::RequestAuthority(stop) => super::denial::request_authority_denied(Binding::IDENTITY, stop),
    })?;
    prerequisites
        .publish(Arc::clone(published.exact_settlement()))
        .retire(displaced, source_owner);
    Ok(published)
}
