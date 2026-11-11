use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::indexes::{BoundedEntityFieldLookupRequest, BoundedIndexParityMode};
use worth_relational::facade::runtime::{ProjectionAspectRequirement, ProjectionAspectScope};
use worth_relational::facade::storage::RecordLifecycleState;

use super::{
    WorthQueryProductIdempotencyAffinity, WorthQueryProviderIdempotencyResolution,
    WorthQueryProviderIdempotencyResolutionDenial,
};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationCommitOutcomeIdentity, WorthQueryApplicationIdempotencyBinding,
    WorthQueryRecordedIntentMatch,
};
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::primary_graph::schema_layout::WorthQueryProviderIdempotencyLayout;
pub(in crate::domain_computation::primary_graph::provider) mod original_binding;
mod original_receipt;
pub(in crate::domain_computation::primary_graph::provider) use original_receipt::RecordedOriginalBinding;
mod key_observation;
pub(in crate::domain_computation::primary_graph::provider) mod source_denial;
mod source_receipt;
pub(in crate::domain_computation::primary_graph::provider) use key_observation::observe;
pub use original_receipt::{
    WorthQueryRecoveredAftermathCausality, WorthQueryRecoveredApplicationCommitReceipt,
    WorthQueryRecoveredDispatchOutbox,
};

pub fn resolve_at_snapshot(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryProviderIdempotencyLayout,
    product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    binding: WorthQueryApplicationIdempotencyBinding,
) -> Result<WorthQueryProviderIdempotencyResolution, WorthQueryProviderIdempotencyResolutionDenial>
{
    let context = WorthQueryIdempotencySnapshotContext {
        provider,
        runtime,
        snapshot,
        layout,
    };
    let key = AspectValue::String(InternedString::from(binding.key_text()));
    let Some(entity) = locate_idempotency_entity(execution, &context, &key)? else {
        return Ok(WorthQueryProviderIdempotencyResolution::Absent);
    };
    // The current index and row locate the original artifact. They supply no
    // receipt fields; those are reread from the sealed creation patch.
    let record = read_idempotency_record(execution, &context, entity, &key)?;
    let commit = runtime
        .history()
        .immutable_commit_receipt_for_version(record.created_at_version)
        .ok_or(WorthQueryProviderIdempotencyResolutionDenial::Unavailable)?;
    let receipt = source_receipt::read_original(execution, &context, product, entity, &commit)?;
    let expected = WorthQueryProductIdempotencyAffinity::from_observation(product.observation());
    if receipt.recorded_affinity() != super::recorded_origin::affinity_text(&expected) {
        return Err(WorthQueryProviderIdempotencyResolutionDenial::ForeignAffinity);
    }
    if receipt.idempotency_key_identity() != binding.key_identity() {
        return Err(WorthQueryProviderIdempotencyResolutionDenial::Unavailable);
    }
    match binding.match_recorded_intent(receipt.recorded_intent())? {
        WorthQueryRecordedIntentMatch::Same => {
            Ok(WorthQueryProviderIdempotencyResolution::Readmitted(receipt))
        }
        WorthQueryRecordedIntentMatch::Drift => Ok(WorthQueryProviderIdempotencyResolution::Drift),
        WorthQueryRecordedIntentMatch::Unverifiable => {
            Err(WorthQueryProviderIdempotencyResolutionDenial::RecordedIntentUnverifiable)
        }
    }
}

struct WorthQueryIdempotencySnapshotContext<'a> {
    provider: &'a WorthQueryPrimaryGraphProvider,
    runtime: &'a worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &'a worth_relational::facade::snapshots::SnapshotHandle,
    layout: &'a WorthQueryProviderIdempotencyLayout,
}

struct WorthQueryProjectedIdempotencyRecord {
    created_at_version: worth_relational::facade::identity::VersionId,
}

fn locate_idempotency_entity(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    context: &WorthQueryIdempotencySnapshotContext<'_>,
    key: &AspectValue,
) -> Result<
    Option<worth_relational::facade::identity::EntityId>,
    WorthQueryProviderIdempotencyResolutionDenial,
> {
    let request = BoundedEntityFieldLookupRequest::new(
        context.snapshot.clone(),
        context.layout.key_index_id,
        context.layout.entity_kind,
        context.layout.key_locator.clone(),
        key.clone(),
        2,
    )
    .map_err(|_| WorthQueryProviderIdempotencyResolutionDenial::Unavailable)?;
    let lookup = context
        .runtime
        .index_access()
        .execute_bounded_entity_field_lookup(request, BoundedIndexParityMode::Production, execution)
        .map_err(|denied| match denied.kind() {
            worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::Execution(
                cause,
            ) => idempotency_read_stop(cause),
            _ => WorthQueryProviderIdempotencyResolutionDenial::Unavailable,
        })?;
    if lookup.overflowed() || lookup.candidate_entity_ids().len() > 1 {
        return Err(WorthQueryProviderIdempotencyResolutionDenial::Unavailable);
    }
    Ok(lookup.candidate_entity_ids().first().copied())
}

fn read_idempotency_record(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    context: &WorthQueryIdempotencySnapshotContext<'_>,
    entity_id: worth_relational::facade::identity::EntityId,
    key: &AspectValue,
) -> Result<WorthQueryProjectedIdempotencyRecord, WorthQueryProviderIdempotencyResolutionDenial> {
    let field = context
        .layout
        .key_locator
        .field_path()
        .fields()
        .first()
        .cloned()
        .ok_or(WorthQueryProviderIdempotencyResolutionDenial::Unavailable)?;
    let aspect = context.layout.key_locator.aspect().aspect_key().clone();
    let scope = ProjectionAspectScope::from_requirements([ProjectionAspectRequirement::fields(
        aspect.clone(),
        [field.clone()],
    )]);
    let view = context
        .runtime
        .read_truth()
        .project_snapshot(context.snapshot)
        .ok_or(WorthQueryProviderIdempotencyResolutionDenial::Unavailable)?;
    view.entity_record_with_projection_scope(execution, entity_id, scope, |record| {
        (record.kind_id() == context.layout.entity_kind
            && record.lifecycle() == RecordLifecycleState::Live
            && record.aspect_field_value(&aspect, &field) == Some(key))
        .then(|| WorthQueryProjectedIdempotencyRecord {
            created_at_version: record.created_at_version(),
        })
    })
    .map_err(idempotency_read_stop)?
    .ok_or(WorthQueryProviderIdempotencyResolutionDenial::Unavailable)
}

pub fn idempotency_read_stop(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
) -> WorthQueryProviderIdempotencyResolutionDenial {
    WorthQueryProviderIdempotencyResolutionDenial::ExecutionDenied(
        super::super::relational_execution_denial::relational_execution_kind(cause, None),
    )
}
