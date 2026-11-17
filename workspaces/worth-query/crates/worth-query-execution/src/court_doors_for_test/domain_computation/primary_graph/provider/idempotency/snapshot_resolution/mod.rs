//! Doors under `domain_computation::primary_graph::provider::idempotency::snapshot_resolution`.
pub use crate::domain_computation::primary_graph::provider::idempotency::snapshot_resolution::idempotency_read_stop;
use crate::domain_computation::primary_graph::application_attempt::idempotency::envelope_candidates::EnvelopeCandidates;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
use crate::domain_computation::primary_graph::provider::{
    WorthQueryPrimaryGraphProvider, WorthQueryProviderIdempotencyResolution,
    WorthQueryProviderIdempotencyResolutionDenial,
};

pub fn resolve_at_snapshot(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &crate::domain_computation::primary_graph::schema_layout::WorthQueryProviderIdempotencyLayout,
    product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    binding: WorthQueryApplicationIdempotencyBinding,
) -> Result<WorthQueryProviderIdempotencyResolution, WorthQueryProviderIdempotencyResolutionDenial>
{
    let bindings = [binding];
    let candidates = EnvelopeCandidates::new(&bindings)?;
    crate::domain_computation::primary_graph::provider::idempotency::snapshot_resolution::resolve_at_snapshot(
        execution, provider, runtime, snapshot, layout, product, &candidates,
    )
}
