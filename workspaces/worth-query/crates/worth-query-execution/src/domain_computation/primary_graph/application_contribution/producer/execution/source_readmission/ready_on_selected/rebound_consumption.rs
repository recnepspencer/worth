//! Re-certify and retain the exact resolved successor at the disclosure root.
use super::*;
use crate::basis::WorthQueryProductObservationLease;
use crate::domain_computation::primary_graph::application_contribution::producer::demand::ReboundConsumedOutput;
use crate::domain_computation::primary_graph::application_output_demand::ReadyCompletion;
use worth_relational::facade::snapshots::SnapshotHandle;

pub(super) fn retain<Schema: worth_query_installation::facade::ApplicationSchema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    completion: &ReadyCompletion,
    pending: &crate::domain_computation::primary_graph::invariant_projection::SelectedPendingConsumedOutput<'_>,
    product: &WorthQueryProductObservationLease,
    snapshot: &SnapshotHandle,
    positioned: &PositionedRelationalSnapshot,
    admission: &mut InvalidationEditAdmission,
) -> Result<ReboundConsumedOutput, ProducerExecutionStop> {
    let handle = &runtime.primary_provider.graph;
    let candidate = handle.output_lineage.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
        .resolve_required_settlement(runtime.runtime.authority_identity().as_u64(),
            &runtime.installed_schema.binding_identity(), completion, admission)
        .map_err(|stop| match stop {
            crate::domain_computation::primary_graph::output_lineage::RequiredSettlementStop::Admission(stop) =>
                ready::ready_resource_denial("resolved consumed output", stop),
            crate::domain_computation::primary_graph::output_lineage::RequiredSettlementStop::Foreign =>
                ProducerExecutionStop::ExecutionStopped(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ForeignSettlement, "resolved consumed output")),
        })?;
    let Ok(Some(candidate)) = candidate else {
        return Ok(ReboundConsumedOutput::Fresh);
    };
    if !candidate
        .matches_consumed_output(pending, admission)
        .map_err(|stop| ready::ready_resource_denial("resolved consumed output", stop))?
    {
        return Ok(ReboundConsumedOutput::Fresh);
    }
    handle
        .with_runtime(|relational| {
            let current = candidate.certify_current(
                &handle.source_owner.invalidation_owner,
                relational,
                product,
                snapshot,
                positioned,
                admission,
            )?;
            let CurrentAcceptedResult::Current(proof) = current else {
                return Ok(ReboundConsumedOutput::Fresh);
            };
            let Some(bound) = proof.bind_product(admission)? else {
                return Ok(ReboundConsumedOutput::Fresh);
            };
            bound
                .retain_consumed(&handle.source_owner.invalidation_owner, admission)
                .map(ReboundConsumedOutput::Equal)
                .map_err(CurrentAcceptedStop::Closure)
        })
        .map_err(|stop| ready::ready_currentness_denial("resolved consumed output", stop))
}
