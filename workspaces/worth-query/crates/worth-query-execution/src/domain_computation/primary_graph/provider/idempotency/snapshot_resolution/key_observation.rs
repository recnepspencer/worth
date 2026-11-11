//! Correction lookup shares the ordinary original-row and World receipt owner.
use super::*;
pub(in crate::domain_computation::primary_graph::provider) fn observe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    key: [u8; 32],
) -> Result<
    Option<WorthQueryRecoveredApplicationCommitReceipt>,
    WorthQueryProviderIdempotencyResolutionDenial,
> {
    provider.graph.with_runtime(execution, |runtime| {
        let snapshot = crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(
            runtime,
            product.observation().basis().relational_basis(),
        )
        .map_err(|_| WorthQueryProviderIdempotencyResolutionDenial::Unavailable)?;
        let result = read(execution, provider, runtime, &snapshot, product, key);
        crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
        result
    })
}
fn read(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    key: [u8; 32],
) -> Result<
    Option<WorthQueryRecoveredApplicationCommitReceipt>,
    WorthQueryProviderIdempotencyResolutionDenial,
> {
    let context = WorthQueryIdempotencySnapshotContext {
        provider,
        runtime,
        snapshot,
        layout: provider.graph.layout.provider_idempotency(),
    };
    let key_text = AspectValue::String(InternedString::from(
        WorthQueryApplicationIdempotencyBinding::recorded_key_text(key),
    ));
    let Some(entity) = locate_idempotency_entity(execution, &context, &key_text)? else {
        return Ok(None);
    };
    let record = read_idempotency_record(execution, &context, entity, &key_text)?;
    let commit = runtime
        .history()
        .immutable_commit_receipt_for_version(record.created_at_version)
        .ok_or(WorthQueryProviderIdempotencyResolutionDenial::Unavailable)?;
    let receipt = source_receipt::read_original(execution, &context, product, entity, &commit)?;
    let expected = WorthQueryProductIdempotencyAffinity::from_observation(product.observation());
    if receipt.recorded_affinity() != super::super::recorded_origin::affinity_text(&expected) {
        return Err(WorthQueryProviderIdempotencyResolutionDenial::ForeignAffinity);
    }
    if receipt.idempotency_key_identity() != &key {
        return Err(WorthQueryProviderIdempotencyResolutionDenial::Unavailable);
    }
    Ok(Some(receipt))
}
