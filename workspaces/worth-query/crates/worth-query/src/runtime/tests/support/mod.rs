use std::collections::BTreeMap;

pub(in crate::runtime::tests) use super::super::*;
pub(in crate::runtime::tests) use crate::declarative_live::{
    DeclarativeLiveViewShape, DeclarativeProjectionField,
};
use crate::memory_workspace::WorthQueryEntityIdentity;
pub(in crate::runtime::tests) use crate::program::{
    WorthQueryAdmittedAspectValueTemplate, WorthQueryOperation, WorthQueryPortType,
    WorthQueryProgramSource, WorthQueryProgramValue, WorthQuerySchemaAdapter, WorthQueryTypedPort,
    WorthQueryValueExpr, WorthQueryWriteCommandTemplate,
};
pub(in crate::runtime::tests) use crate::runtime::async_result_state::WorthQueryRuntimeAsyncResultProjection;
pub(in crate::runtime::tests) use crate::runtime::remask_posture::WorthQueryRuntimeRemaskProjection;
pub(in crate::runtime::tests) use crate::schema_view::SchemaFieldView;
pub(in crate::runtime::tests) use crate::subscription::QueryPatchGroupKind;
pub(in crate::runtime::tests) use worth_foundational::facade::{
    prepare_aspect_value_identity_basis, AspectKey, AspectLocator, AspectValue, CanonicalFieldPath,
    FieldKey, LocatorAuthority, ScalarAspectType,
};
pub(in crate::runtime::tests) use worth_runtime_bridge::facade::{
    BridgeAsyncCompletionClass, BridgeAsyncCompletionDenialClass, BridgeAsyncCompletionState,
    BridgeAsyncForwardCausalityClass, BridgeCommittedPatchEnvelope,
    BridgeCommittedPatchEnvelopeIdentity, BridgeCommittedPatchItem, BridgeCommittedPatchTarget,
    BridgeDeliveryReceipt, BridgeMappingId, BridgeMappingRegistration, CoarseRoutingMode,
    InvalidationSink, MappingSelector, RelationalBridgeRecordIdentityParts,
    RelationalBridgeSourceError, RelationalCommittedPatchRequest, RuntimeBridgeBuilder,
    SignalBridgeSinkError, SignalInvalidationScope, SnapshotReadContract, SnapshotReadPacket,
    SnapshotReadPacketResult, SnapshotReadRecord, SnapshotReadSource, TruthBranchIdentity,
    TruthCommitIdentity, TruthPatchIdentity, TruthPatchScope, TruthSnapshotIdentity,
    TruthSnapshotReader,
};
mod adapters;
mod bridge;
mod commands;
mod domains;
mod graph_composition;
mod mixed_cause;
mod mutation_authority_bridge;
mod mutation_receipt_support;
mod native_aspect_contracts;
mod program;
mod schema;
mod stateful_bridge_runtime;

pub(in crate::runtime::tests) use adapters::*;
pub(crate) use bridge::*;
pub(in crate::runtime::tests) use commands::*;
pub(in crate::runtime::tests) use domains::*;
pub(in crate::runtime::tests) use graph_composition::*;
pub(in crate::runtime::tests) use mixed_cause::*;
pub(in crate::runtime::tests) use mutation_authority_bridge::*;
pub(in crate::runtime::tests) use mutation_receipt_support::*;
pub(in crate::runtime::tests) use native_aspect_contracts::*;
pub(in crate::runtime::tests) use program::*;
pub(in crate::runtime::tests) use schema::*;
pub(crate) use stateful_bridge_runtime::*;

pub(in crate::runtime::tests) fn test_session_label(
    label: impl AsRef<str>,
) -> WorthQuerySessionLabel {
    WorthQuerySessionLabel::scoped_strs("worth-query-runtime-tests", [label.as_ref()])
        .expect("test session label should build")
}

pub(in crate::runtime::tests) fn test_mutation_metadata_key(
    key: impl Into<String>,
) -> WorthQueryMutationMetadataKey {
    WorthQueryMutationMetadataKey::new(key).expect("test metadata key should admit")
}

pub(in crate::runtime::tests) fn test_entity_identity(
    identity: impl AsRef<str>,
) -> WorthQueryEntityIdentity {
    relational_test_entity_identity_from_label(identity.as_ref()).unwrap_or_else(|| {
        crate::memory_workspace::admit_authored_entity_label(identity)
            .expect("fixture digest must fit")
    })
}

fn relational_test_entity_identity_from_label(label: &str) -> Option<WorthQueryEntityIdentity> {
    parse_typed_relational_record_parts(label)
        .or_else(|| parse_collection_slot_relational_record_parts(label))
        .or_else(|| relational_test_entity_identity_named_fixture(label))
        .map(WorthQueryEntityIdentity::from_relational_record)
}

fn parse_typed_relational_record_parts(label: &str) -> Option<RelationalBridgeRecordIdentityParts> {
    let mut segments = label.split(':');
    let kind = segments.next()?;
    let partition_id = segments.next()?.parse().ok()?;
    let local_slot = segments.next()?.parse().ok()?;
    let generation = segments.next()?.parse().ok()?;
    if segments.next().is_some() {
        return None;
    }
    match kind {
        "entity" => Some(RelationalBridgeRecordIdentityParts::entity(
            partition_id,
            local_slot,
            generation,
        )),
        "relation" => Some(RelationalBridgeRecordIdentityParts::relation(
            partition_id,
            local_slot,
            generation,
        )),
        _ => None,
    }
}

fn parse_collection_slot_relational_record_parts(
    label: &str,
) -> Option<RelationalBridgeRecordIdentityParts> {
    let (collection, slot_text) = label.rsplit_once(':')?;
    let local_slot = slot_text.parse().ok()?;
    if relational_test_entity_identity_is_relation_collection(collection) {
        Some(RelationalBridgeRecordIdentityParts::relation(
            2, local_slot, 0,
        ))
    } else {
        Some(RelationalBridgeRecordIdentityParts::entity(
            1, local_slot, 0,
        ))
    }
}

fn relational_test_entity_identity_is_relation_collection(collection: &str) -> bool {
    collection.ends_with("Relation") || matches!(collection, "Edge" | "TaskEdge")
}

fn relational_test_entity_identity_named_fixture(
    label: &str,
) -> Option<RelationalBridgeRecordIdentityParts> {
    match label {
        "task-existing" | "vertex-a" | "face-1" => {
            Some(RelationalBridgeRecordIdentityParts::entity(1, 1, 0))
        }
        "task-existing-left" | "vertex-b" => {
            Some(RelationalBridgeRecordIdentityParts::entity(1, 2, 0))
        }
        "task-existing-right" => Some(RelationalBridgeRecordIdentityParts::entity(1, 3, 0)),
        "task-second-existing" => Some(RelationalBridgeRecordIdentityParts::entity(1, 4, 0)),
        "vertex-split" => Some(RelationalBridgeRecordIdentityParts::entity(1, 4, 0)),
        "vertex-merged" => Some(RelationalBridgeRecordIdentityParts::entity(1, 5, 0)),
        "vertex-c" => Some(RelationalBridgeRecordIdentityParts::entity(1, 6, 0)),
        "he-1" => Some(RelationalBridgeRecordIdentityParts::entity(3, 1, 0)),
        "he-2" => Some(RelationalBridgeRecordIdentityParts::entity(3, 2, 0)),
        "he-3" => Some(RelationalBridgeRecordIdentityParts::entity(3, 3, 0)),
        "loop-a" => Some(RelationalBridgeRecordIdentityParts::entity(4, 1, 0)),
        "loop-b" => Some(RelationalBridgeRecordIdentityParts::entity(4, 2, 0)),
        "loop-c" => Some(RelationalBridgeRecordIdentityParts::entity(4, 3, 0)),
        _ => None,
    }
}

pub(in crate::runtime::tests) fn test_write_adjacent_origin_identity(
    class: WorthQueryEffectWriteAdjacentTriggerClass,
    origin: impl AsRef<str>,
) -> crate::evidence_identity::WorthQueryEvidenceIdentity {
    crate::evidence_identity::WorthQueryEvidenceIdentity::compose(
        crate::evidence_identity::WorthQueryEvidenceScope::EffectIntentReceiptPhase,
    )
    .field_shape(
        crate::evidence_identity::WorthQueryEvidenceTag::new("identity_family"),
        "test_effect_write_adjacent_origin_v1",
    )
    .field_shape(
        crate::evidence_identity::WorthQueryEvidenceTag::new("class"),
        class.as_str(),
    )
    .field_value(
        crate::evidence_identity::WorthQueryEvidenceTag::new("origin_fixture"),
        origin.as_ref(),
    )
    .seal()
    .expect("fixture digest must fit")
}

pub(in crate::runtime::tests) fn live_subscription_async_identity(
    runtime: &WorthQueryRuntime,
    view_name: &str,
) -> (
    crate::evidence_identity::WorthQueryEvidenceIdentity,
    crate::evidence_identity::WorthQueryEvidenceIdentity,
) {
    let state = runtime
        .live_subscriptions
        .get(&WorthQueryLiveArtifactTarget::from_view_name(view_name))
        .expect("live subscription state should exist");
    (
        state.installation.basis_binding_identity().clone(),
        state.active_lane_handle.checkpoint_identity().clone(),
    )
}

pub(in crate::runtime::tests) fn retained_test_row(
    fields: impl IntoIterator<Item = (impl Into<String>, AspectValue)>,
) -> WorthQueryRetainedMaterializedRow {
    WorthQueryRetainedMaterializedRow::from_scalar_values(
        fields
            .into_iter()
            .map(|(field, value)| {
                (
                    retained_test_field_path(field.into())
                        .expect("test retained field path should parse"),
                    value,
                )
            })
            .collect::<BTreeMap<_, _>>(),
    )
    .expect("test retained row should admit native scalar values")
}

pub(in crate::runtime::tests) fn retained_test_field_path(
    path: impl Into<String>,
) -> Result<WorthQueryRetainedFieldPath, String> {
    let path = path.into();
    let fields = path
        .split('.')
        .map(|field| {
            FieldKey::new(field.to_string())
                .ok_or_else(|| format!("`{path}` is not a test retained scalar field path"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let path = CanonicalFieldPath::new(fields)
        .ok_or_else(|| format!("`{path}` is not a test retained scalar field path"))?;
    Ok(WorthQueryRetainedFieldPath::from_canonical_field_path(path))
}

pub(in crate::runtime::tests) fn retained_string_test_row(
    field: impl Into<String>,
    value: impl Into<String>,
) -> WorthQueryRetainedMaterializedRow {
    retained_test_row([(
        field.into(),
        crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(value),
    )])
}

pub(in crate::runtime::tests) fn test_native_scalar_value<'a>(
    row: &'a WorthQueryEntity,
    field_path: &str,
) -> Option<&'a AspectValue> {
    let path = CanonicalFieldPath::new(
        field_path
            .split('.')
            .map(|field| FieldKey::new(field.to_string()))
            .collect::<Option<Vec<_>>>()?,
    )?;
    row.scalar_value_at(&path)
}

pub(in crate::runtime::tests) fn test_native_string_value(
    row: &WorthQueryEntity,
    field_path: &str,
) -> Option<String> {
    let AspectValue::String(value) = test_native_scalar_value(row, field_path)? else {
        return None;
    };
    Some(match value.raw_text() {
        Some(text) => text.to_owned(),
        None => format!("symbol:{}", value.as_symbol()?.0),
    })
}

pub(in crate::runtime::tests) fn test_has_native_field_prefix(
    row: &WorthQueryEntity,
    prefix: &str,
) -> bool {
    row.native_field_values().any(|(field_path, _)| {
        field_path
            .fields()
            .first()
            .is_some_and(|field| field.as_str() == prefix)
    })
}

pub(in crate::runtime::tests) fn test_aspect_touch(touch_fixture: &str) -> WorthQueryAspectTouch {
    let mut segments = touch_fixture.split('.');
    let aspect_key = AspectKey::new(
        segments
            .next()
            .expect("test touch fixture should name an aspect"),
    )
    .expect("test aspect key should admit");
    let field_segments = segments
        .map(|field| FieldKey::new(field).expect("test field key should admit"))
        .collect::<Vec<_>>();
    if field_segments.is_empty() {
        WorthQueryAspectTouch::whole_aspect(aspect_key)
    } else {
        WorthQueryAspectTouch::aspect_field_path(
            aspect_key,
            CanonicalFieldPath::new(field_segments).expect("test field path should admit"),
        )
    }
}

pub(in crate::runtime::tests) fn test_aspect_touches<const N: usize>(
    touch_fixtures: [&str; N],
) -> [WorthQueryAspectTouch; N] {
    touch_fixtures.map(test_aspect_touch)
}

pub(in crate::runtime::tests) fn identity_id_field_key() -> crate::authoring::AspectFieldKey {
    let aspect = AspectKey::new("identity").expect("identity aspect key should admit");
    let field = FieldKey::new("id").expect("id field key should admit");
    crate::authoring::AspectFieldKey::from_native_keys(&aspect, &field)
}

pub(in crate::runtime::tests) fn title_value_field_key() -> crate::authoring::AspectFieldKey {
    let aspect = AspectKey::new("title").expect("title aspect key should admit");
    let field = FieldKey::new("value").expect("value field key should admit");
    crate::authoring::AspectFieldKey::from_native_keys(&aspect, &field)
}

pub(in crate::runtime::tests) fn kind_value_field_key() -> crate::authoring::AspectFieldKey {
    let aspect = AspectKey::new("kind").expect("kind aspect key should admit");
    let field = FieldKey::new("value").expect("value field key should admit");
    crate::authoring::AspectFieldKey::from_native_keys(&aspect, &field)
}

pub(in crate::runtime::tests) fn edge_kind_field_key() -> crate::authoring::AspectFieldKey {
    let aspect = AspectKey::new("edge").expect("edge aspect key should admit");
    let field = FieldKey::new("kind").expect("kind field key should admit");
    crate::authoring::AspectFieldKey::from_native_keys(&aspect, &field)
}

pub(in crate::runtime::tests) fn status_value_field_key() -> crate::authoring::AspectFieldKey {
    let aspect = AspectKey::new("status").expect("status aspect key should admit");
    let field = FieldKey::new("value").expect("value field key should admit");
    crate::authoring::AspectFieldKey::from_native_keys(&aspect, &field)
}

pub(in crate::runtime::tests) fn test_string_aspect_value(value: impl Into<String>) -> AspectValue {
    crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(value)
}

pub(in crate::runtime::tests) fn test_authored_string_aspect_value(
    value: impl Into<String>,
) -> WorthQueryAuthoredAspectValue {
    WorthQueryAuthoredAspectValue::string(value)
}

pub(in crate::runtime::tests) fn test_native_string_value_identity(
    value: impl Into<String>,
) -> String {
    prepare_aspect_value_identity_basis(&test_string_aspect_value(value))
        .as_str()
        .to_string()
}

pub(in crate::runtime::tests) fn test_authored_string_terminal_digest(
    aspect_path: &str,
    value: impl Into<String>,
) -> String {
    format!(
        "{}=set:{}",
        test_aspect_touch(aspect_path).admitted_touch_digest_part(),
        test_native_string_value_identity(value)
    )
}

pub(in crate::runtime::tests) fn test_native_entity_ref_value(
    identity: &WorthQueryEntityIdentity,
) -> AspectValue {
    let parts = identity
        .relational_entity_record_parts()
        .expect("test native entity references require relational entity authority");
    AspectValue::EntityRef(worth_foundational::facade::EntityId::new(
        worth_foundational::facade::PartitionId(parts.partition_id()),
        parts.local_slot(),
        parts.generation(),
    ))
}

pub(in crate::runtime::tests) fn test_update_string_aspect_command(
    entity_identity: WorthQueryEntityIdentity,
    touch_fixture: &str,
    value: impl Into<String>,
) -> WorthQueryWriteCommand {
    WorthQueryWriteCommand::UpdateAspect {
        entity_identity,
        aspect: WorthQueryAuthoredAspectMutation::new_set(
            test_aspect_touch(touch_fixture),
            test_string_aspect_value(value),
        )
        .expect("test update aspect should build"),
    }
}

#[path = "test_intent_input.rs"]
mod test_intent_input;
pub use test_intent_input::test_intent_input;
