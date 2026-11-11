//! Public diagnostic, field-value and visible-truth observations of merge execution.
use super::branch_history_recovery::current_head;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    AspectKey, AspectValue, ContractValidatedAspectValueView, FieldKey, InternedString,
};
use worth_relational::facade::{
    diagnostics::{RelationalDiagnosticValue, RelationalDiagnosticsEntry},
    history::{AspectHistoryDigest, BranchId},
    identity::{EntityId, PartitionId},
    runtime::{EntityReadRecord, RelationalRuntime},
};

pub(super) fn diagnostic_field<'a>(
    entry: &'a RelationalDiagnosticsEntry,
    field: &str,
) -> &'a RelationalDiagnosticValue {
    diagnostic_field_optional(entry, field)
        .unwrap_or_else(|| panic!("diagnostic field '{field}' missing from {entry:?}"))
}

pub(super) fn diagnostic_field_optional<'a>(
    entry: &'a RelationalDiagnosticsEntry,
    field: &str,
) -> Option<&'a RelationalDiagnosticValue> {
    let RelationalDiagnosticValue::Object(fields) = entry.fields.root() else {
        panic!("diagnostic entry fields are not an object: {entry:?}");
    };
    fields.get(field)
}

/// The display text of one declared field at the branch's actual admitted head.
pub(super) fn read_entity_aspect_field_display(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
    entity_id: EntityId,
    aspect_key: AspectKey,
    field: FieldKey,
) -> String {
    let version_id = current_head(runtime, branch)
        .expect("branch head")
        .version_id;
    runtime
        .read_truth()
        .try_read_version(execution, version_id)
        .expect("read request remains live")
        .expect("the actual branch head remains retained")
        .get_entity(entity_id)
        .and_then(|entity| entity_field_display(entity, &aspect_key, &field))
        .expect("aspect field display value")
}

fn entity_field_display(
    record: &EntityReadRecord,
    aspect_key: &AspectKey,
    field: &FieldKey,
) -> Option<String> {
    let state = record.authoritative_aspect_state.as_ref()?;
    let value = match state.get(aspect_key)?.view() {
        ContractValidatedAspectValueView::Scalar(value) => value,
        ContractValidatedAspectValueView::Struct(value) => value.get(field)?,
    };
    Some(display_text(value))
}

fn display_text(value: &AspectValue) -> String {
    match value {
        AspectValue::InlineBytes(value) => format!("inline-bytes:{:02x?}", value.as_bytes()),
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
        AspectValue::String(InternedString::Raw(value)) => value.clone(),
        AspectValue::String(InternedString::Stored(value)) => value.as_str().to_owned(),
        AspectValue::String(InternedString::Symbol(symbol)) => format!("symbol:{}", symbol.0),
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
        AspectValue::ContentRef(value) => format!("content-ref:{value}"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct VisibleRelationSummary {
    partition: PartitionId,
    source_name: String,
    target_name: String,
}

/// Named entities and named-endpoint relations visible at the actual main head.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct VisibleTruthSummary {
    entity_names: Vec<String>,
    relations: Vec<VisibleRelationSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AspectTruthBundle {
    pub(super) visible_truth: VisibleTruthSummary,
    pub(super) entity_history_digests: Vec<(EntityId, AspectHistoryDigest)>,
}

pub(super) fn capture_aspect_truth_bundle(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity_ids: &[EntityId],
) -> AspectTruthBundle {
    let main = BranchId("main".to_string());
    AspectTruthBundle {
        visible_truth: visible_truth(execution, runtime, &main),
        entity_history_digests: entity_ids
            .iter()
            .map(|entity_id| {
                let digest = runtime
                    .history()
                    .entity_aspect_history_with_trace(execution, &main, *entity_id, None)
                    .unwrap()
                    .aspect_history_digest();
                (*entity_id, digest)
            })
            .collect(),
    }
}

fn visible_truth(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
) -> VisibleTruthSummary {
    let version_id = current_head(runtime, branch)
        .expect("branch head")
        .version_id;
    let read = runtime
        .read_truth()
        .try_read_version(execution, version_id)
        .expect("read request remains live")
        .expect("the actual branch head remains retained");
    let name = |record: &EntityReadRecord| {
        entity_field_display(
            record,
            &AspectKey::new("name").unwrap(),
            &FieldKey::new("name").unwrap(),
        )
    };
    let mut entity_names = read.entities().iter().filter_map(name).collect::<Vec<_>>();
    entity_names.sort();
    let mut relations = read
        .relations()
        .iter()
        .filter_map(|relation| {
            Some(VisibleRelationSummary {
                partition: relation.relation_id.partition_id,
                source_name: read.get_entity(relation.source).and_then(name)?,
                target_name: read.get_entity(relation.target).and_then(name)?,
            })
        })
        .collect::<Vec<_>>();
    relations.sort();
    VisibleTruthSummary {
        entity_names,
        relations,
    }
}
