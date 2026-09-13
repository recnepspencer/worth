use worth_foundational::facade::{AspectValue, FieldKey, InternedString};
use worth_relational::facade::indexes::{BoundedEntityFieldLookupRequest, BoundedIndexParityMode};
use worth_relational::facade::runtime::{ProjectionAspectRequirement, ProjectionAspectScope};
use worth_relational::facade::storage::RecordLifecycleState;
use worth_relational::facade::transactions::AspectFieldLocator;

use super::{WorthQueryProductIdempotencyAffinity, WorthQueryProviderIdempotencyResolution};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationCommitOutcomeIdentity, WorthQueryApplicationIdempotencyBinding,
};
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::primary_graph::schema_layout::WorthQueryProviderIdempotencyLayout;

pub(super) fn resolve_at_snapshot(
    provider: &WorthQueryPrimaryGraphProvider,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryProviderIdempotencyLayout,
    expected_product: &WorthQueryProductIdempotencyAffinity,
    binding: WorthQueryApplicationIdempotencyBinding,
) -> Result<WorthQueryProviderIdempotencyResolution, &'static str> {
    let key = AspectValue::String(InternedString::from(binding.key_text()));
    let context = WorthQueryIdempotencySnapshotContext {
        provider,
        runtime,
        snapshot,
        layout,
    };
    let Some(entity_id) = locate_idempotency_entity(&context, &key)? else {
        return Ok(WorthQueryProviderIdempotencyResolution::Absent);
    };
    let record = read_idempotency_record(&context, entity_id, &key)?;
    resolve_projected_idempotency(&context, expected_product, binding, record)
}

struct WorthQueryIdempotencySnapshotContext<'a> {
    provider: &'a WorthQueryPrimaryGraphProvider,
    runtime: &'a worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &'a worth_relational::facade::snapshots::SnapshotHandle,
    layout: &'a WorthQueryProviderIdempotencyLayout,
}

struct WorthQueryIdempotencyProjectionFields {
    key: FieldKey,
    intent: FieldKey,
    outcome_identity: FieldKey,
    emitted_effect_count: FieldKey,
}

struct WorthQueryProjectedIdempotencyRecord {
    created_at_version: worth_relational::facade::identity::VersionId,
    intent: Option<AspectValue>,
    outcome_identity: Option<AspectValue>,
    emitted_effect_count: Option<AspectValue>,
}

fn locate_idempotency_entity(
    context: &WorthQueryIdempotencySnapshotContext<'_>,
    key: &AspectValue,
) -> Result<Option<worth_relational::facade::identity::EntityId>, &'static str> {
    let request = BoundedEntityFieldLookupRequest::new(
        context.snapshot.clone(),
        context.layout.key_index_id,
        context.layout.entity_kind,
        context.layout.key_locator.clone(),
        key.clone(),
        2,
    )
    .map_err(|_| "provider idempotency lookup request was rejected")?;
    let lookup = context
        .runtime
        .index_access()
        .execute_bounded_entity_field_lookup(request, BoundedIndexParityMode::Production)
        .map_err(|_| "provider idempotency index lookup failed")?;
    if lookup.overflowed() || lookup.candidate_entity_ids().len() > 1 {
        return Err("provider idempotency key is not unique");
    }
    Ok(lookup.candidate_entity_ids().first().copied())
}

fn read_idempotency_record(
    context: &WorthQueryIdempotencySnapshotContext<'_>,
    entity_id: worth_relational::facade::identity::EntityId,
    key: &AspectValue,
) -> Result<WorthQueryProjectedIdempotencyRecord, &'static str> {
    let fields = idempotency_projection_fields(context.layout)?;
    let scope = ProjectionAspectScope::from_requirements([ProjectionAspectRequirement::fields(
        context.layout.key_locator.aspect().aspect_key().clone(),
        [
            fields.key.clone(),
            fields.intent.clone(),
            fields.outcome_identity.clone(),
            fields.emitted_effect_count.clone(),
        ],
    )]);
    context
        .runtime
        .read_truth()
        .project_snapshot(context.snapshot)
        .and_then(|view| {
            view.entity_record_with_projection_scope(entity_id, scope, |record| {
                (record.kind_id() == context.layout.entity_kind
                    && record.lifecycle() == RecordLifecycleState::Live
                    && record.aspect_field_value(
                        context.layout.key_locator.aspect().aspect_key(),
                        &fields.key,
                    ) == Some(key))
                .then(|| WorthQueryProjectedIdempotencyRecord {
                    created_at_version: record.created_at_version(),
                    intent: record
                        .aspect_field_value(
                            context.layout.intent_locator.aspect().aspect_key(),
                            &fields.intent,
                        )
                        .cloned(),
                    outcome_identity: record
                        .aspect_field_value(
                            context
                                .layout
                                .outcome_identity_locator
                                .aspect()
                                .aspect_key(),
                            &fields.outcome_identity,
                        )
                        .cloned(),
                    emitted_effect_count: record
                        .aspect_field_value(
                            context
                                .layout
                                .emitted_effect_count_locator
                                .aspect()
                                .aspect_key(),
                            &fields.emitted_effect_count,
                        )
                        .cloned(),
                })
            })
        })
        .ok_or("provider idempotency record is not authoritative")
}

fn idempotency_projection_fields(
    layout: &WorthQueryProviderIdempotencyLayout,
) -> Result<WorthQueryIdempotencyProjectionFields, &'static str> {
    Ok(WorthQueryIdempotencyProjectionFields {
        key: required_locator_field(
            &layout.key_locator,
            "provider idempotency key locator is empty",
        )?,
        intent: required_locator_field(
            &layout.intent_locator,
            "provider idempotency intent locator is empty",
        )?,
        outcome_identity: required_locator_field(
            &layout.outcome_identity_locator,
            "provider idempotency outcome-identity locator is empty",
        )?,
        emitted_effect_count: required_locator_field(
            &layout.emitted_effect_count_locator,
            "provider idempotency emitted-effect-count locator is empty",
        )?,
    })
}

fn required_locator_field(
    locator: &AspectFieldLocator,
    denial: &'static str,
) -> Result<FieldKey, &'static str> {
    locator.field_path().fields().first().cloned().ok_or(denial)
}

fn resolve_projected_idempotency(
    context: &WorthQueryIdempotencySnapshotContext<'_>,
    expected_product: &WorthQueryProductIdempotencyAffinity,
    binding: WorthQueryApplicationIdempotencyBinding,
    record: WorthQueryProjectedIdempotencyRecord,
) -> Result<WorthQueryProviderIdempotencyResolution, &'static str> {
    let expected_intent = AspectValue::String(InternedString::from(binding.intent_text()));
    if record.intent.as_ref() != Some(&expected_intent) {
        return Ok(WorthQueryProviderIdempotencyResolution::Drift);
    }
    let committed = context
        .runtime
        .history()
        .historical_committed_version(record.created_at_version)
        .ok_or("provider idempotency creation commit is unavailable")?;
    let Some(AspectValue::UInt64(outcome_identity)) = record.outcome_identity else {
        return Err("provider idempotency outcome identity is unavailable");
    };
    let outcome_identity = WorthQueryApplicationCommitOutcomeIdentity::restore(outcome_identity)
        .ok_or("provider idempotency outcome identity is invalid")?;
    let Some(AspectValue::UInt64(emitted)) = record.emitted_effect_count else {
        return Err("provider idempotency emitted-effect count is unavailable");
    };
    let emitted = usize::try_from(emitted)
        .map_err(|_| "provider idempotency emitted-effect count exceeds host representation")?;
    let commit = committed.commit().clone();
    let committed = context
        .provider
        .observe_completed_application(&commit)
        .ok_or("provider idempotency commit evidence is unavailable")?;
    let performed_product = WorthQueryProductIdempotencyAffinity::from_reference(
        committed.product_publication().new_product_head(),
    );
    if &performed_product != expected_product {
        return Ok(WorthQueryProviderIdempotencyResolution::Drift);
    }
    if committed.application_outcome_identity() != Some(outcome_identity)
        || committed.runtime_instance_id() != context.snapshot.runtime_instance_id()
        || committed.emitted_effect_count() != emitted
    {
        return Err("provider idempotency commit evidence has foreign affinity");
    }
    Ok(WorthQueryProviderIdempotencyResolution::Equivalent(
        committed,
    ))
}
