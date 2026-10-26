use super::*;

use worth_foundational::facade::{
    AspectFieldLocator, AspectValue, CanonicalFieldPath, FieldKey, InternedString, LocatorAuthority,
};

use crate::storage::data::AuthoritativeFieldComparisonKey;
use crate::visibility::materialization::read_records::{
    entity_query_locus_comparison_key, relation_query_locus_comparison_key,
};

pub(crate) fn install_empty_test_branch(runtime: &RelationalRuntime, branch_id: BranchId) {
    let cell = crate::branch::RelationalBranchReferenceCell::empty(
        runtime.runtime_instance_id(),
        branch_id.clone(),
    )
    .expect("test empty branch identity is valid");
    runtime.history.insert_branch_cell(cell);
    let root = crate::branch::RelationalBranchRoot::empty_with_schema(
        &runtime.config().schema.registry,
        crate::schema::data::runtime_descriptor_semantics_policy().current_write_version(),
    );
    runtime
        .history
        .install_branch_root(&branch_id, root)
        .expect("test retention owner admits the empty branch head");
}

pub(crate) fn read_entity_name(record: &EntityReadRecord) -> Option<String> {
    read_entity_field(record, field_key("name"))
}

pub(crate) fn read_entity_field(record: &EntityReadRecord, field_key: FieldKey) -> Option<String> {
    read_entity_aspect_field(record, aspect_key(field_key.as_str()), field_key)
}

pub(crate) fn read_entity_aspect_field(
    record: &EntityReadRecord,
    aspect_key: worth_foundational::facade::AspectKey,
    field_key: FieldKey,
) -> Option<String> {
    entity_query_locus_comparison_key(record, &test_aspect_field_locator(aspect_key, field_key))
        .and_then(display_text_from_comparison_key)
}

pub(crate) fn read_relation_field(
    record: &crate::facade::runtime::RelationReadRecord,
    field_key: FieldKey,
) -> Option<String> {
    relation_query_locus_comparison_key(
        record,
        &test_aspect_field_locator(aspect_key(field_key.as_str()), field_key),
    )
    .and_then(display_text_from_comparison_key)
}

fn test_aspect_field_locator(
    aspect_key: worth_foundational::facade::AspectKey,
    field_key: FieldKey,
) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Planned,
        aspect_key,
        CanonicalFieldPath::single(field_key),
    )
}

fn display_text_from_comparison_key(key: AuthoritativeFieldComparisonKey) -> Option<String> {
    crate::aspect_wire::decode_aspect_value(key.canonical_value_bytes())
        .ok()
        .map(display_text_for_test_aspect_value)
}

fn display_text_for_test_aspect_value(value: AspectValue) -> String {
    match value {
        AspectValue::Null => "null".to_string(),
        AspectValue::Bool(value) => value.to_string(),
        AspectValue::Int8(value) => value.to_string(),
        AspectValue::Int16(value) => value.to_string(),
        AspectValue::Int32(value) => value.to_string(),
        AspectValue::Int64(value) => value.to_string(),
        AspectValue::UInt8(value) => value.to_string(),
        AspectValue::UInt16(value) => value.to_string(),
        AspectValue::UInt32(value) => value.to_string(),
        AspectValue::UInt64(value) => value.to_string(),
        AspectValue::Float32(value) => format!("f32-bits:{}", value.bits()),
        AspectValue::Float64(value) => format!("f64-bits:{}", value.bits()),
        AspectValue::Decimal(value) => value.as_str().to_string(),
        AspectValue::BigInt(value) => value.as_str().to_string(),
        AspectValue::Rational(value) => format!(
            "{}/{}",
            value.numerator.as_str(),
            value.denominator.as_str()
        ),
        AspectValue::String(value) => display_text_for_test_interned_string(value),
        AspectValue::Bytes(value) => format!("bytes-ref:{}", value.0),
        AspectValue::Uuid(value) => value.iter().map(|byte| format!("{byte:02x}")).collect(),
        AspectValue::Date(value) => value.days_from_unix_epoch.to_string(),
        AspectValue::Time(value) => value.nanos_since_midnight.to_string(),
        AspectValue::Timestamp(value) => value.micros_since_unix_epoch.to_string(),
        AspectValue::TimestampTz(value) => format!(
            "{}:{}",
            value.utc_micros_since_unix_epoch, value.offset_minutes
        ),
        AspectValue::EntityRef(value) => format!(
            "entity:{}:{}:{}",
            value.partition_id.0, value.local_slot.0, value.generation.0
        ),
        AspectValue::ContentRef(value) => format!("content-ref:{}", value.0),
    }
}

fn display_text_for_test_interned_string(value: InternedString) -> String {
    match value {
        InternedString::Raw(value) => value,
        InternedString::Symbol(symbol) => format!("symbol:{}", symbol.0),
    }
}

pub(crate) fn all_aspect_filter(
    names: impl IntoIterator<Item = &'static str>,
) -> ProjectionAspectFilter {
    ProjectionAspectFilter::whole_aspects(
        ProjectionAspectFilterMode::All,
        names.into_iter().map(aspect_key),
    )
}

pub(crate) fn any_aspect_filter(
    names: impl IntoIterator<Item = &'static str>,
) -> ProjectionAspectFilter {
    ProjectionAspectFilter::whole_aspects(
        ProjectionAspectFilterMode::Any,
        names.into_iter().map(aspect_key),
    )
}

pub(crate) fn entity_aspect_history_digest(
    runtime: &RelationalRuntime,
    entity_id: crate::facade::identity::EntityId,
    filter: Option<&ProjectionAspectFilter>,
) -> crate::facade::history::AspectHistoryDigest {
    entity_aspect_history_digest_on_branch(
        runtime,
        &BranchId("main".to_string()),
        entity_id,
        filter,
    )
}

pub(crate) fn entity_aspect_history_digest_on_branch(
    runtime: &RelationalRuntime,
    branch_id: &BranchId,
    entity_id: crate::facade::identity::EntityId,
    filter: Option<&ProjectionAspectFilter>,
) -> crate::facade::history::AspectHistoryDigest {
    runtime
        .history()
        .entity_aspect_history_with_trace(branch_id, entity_id, filter)
        .aspect_history_digest()
}

pub(crate) fn relation_aspect_history_digest(
    runtime: &RelationalRuntime,
    relation_id: RelationId,
    filter: Option<&ProjectionAspectFilter>,
) -> crate::facade::history::AspectHistoryDigest {
    relation_aspect_history_digest_on_branch(
        runtime,
        &BranchId("main".to_string()),
        relation_id,
        filter,
    )
}

pub(crate) fn relation_aspect_history_digest_on_branch(
    runtime: &RelationalRuntime,
    branch_id: &BranchId,
    relation_id: RelationId,
    filter: Option<&ProjectionAspectFilter>,
) -> crate::facade::history::AspectHistoryDigest {
    runtime
        .history()
        .relation_aspect_history_with_trace(branch_id, relation_id, filter)
        .aspect_history_digest()
}

pub(crate) fn lineage_aspect_history_digest(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    lineage_id: LineageId,
    filter: Option<&ProjectionAspectFilter>,
) -> crate::facade::history::LineageAspectResolutionDigest {
    lineage_aspect_history_digest_on_branch(
        execution,
        runtime,
        &BranchId("main".to_string()),
        lineage_id,
        filter,
    )
}

pub(crate) fn lineage_aspect_history_digest_on_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch_id: &BranchId,
    lineage_id: LineageId,
    filter: Option<&ProjectionAspectFilter>,
) -> crate::facade::history::LineageAspectResolutionDigest {
    runtime
        .lineage_access()
        .entity_aspect_history_with_trace(execution,
            crate::facade::lineage::HistoricalResolutionRequest {
                branch_id: branch_id.clone(),
                lineage_id,
                boundedness_basis:
                    crate::facade::lineage::HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
            },
            filter,
        ).unwrap()
        .lineage_aspect_resolution_digest()
}
