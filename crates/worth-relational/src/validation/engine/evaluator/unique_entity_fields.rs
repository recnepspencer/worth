mod projected_entity_field_values;

use std::collections::BTreeMap;

use worth_foundational::facade::{AspectFieldLocator, AspectValue, CanonicalFieldPath, FieldKey};

use crate::diagnostics::data::DiagnosticCode;
use crate::storage::data::authoritative_aspect_value_field_comparison_key;
use crate::validation::data::{
    InvariantClass, InvariantViolation, InvariantViolationFields, StorageInconsistencyScan,
};

use super::super::context::InvariantExecutionContext;
use super::common::{storage_inconsistency_violation, StorageInconsistencyContext};
use projected_entity_field_values::projected_entity_aspect_field_value_for_metadata;

/// The authoritative uniqueness evaluator is deliberately a selected-state scan.
/// Until a branch-qualified uniqueness index exists, the rule is Global and must
/// inspect every selected partition. Commit-boundary observations provide a
/// detached candidate materialized by the canonical mutation engine; other
/// observations use their ordinary committed or speculative state directly.
pub(super) fn evaluate_unique_entity_aspect_field(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    field_locator: &AspectFieldLocator,
) -> Option<InvariantViolation> {
    let field = single_field(field_locator)?;
    let mut seen = BTreeMap::new();
    let state_view = context.enforcement_state_view();
    for partition_id in state_view.state().partition_ids_iter() {
        if !context.checkpoint(1) {
            return None;
        }
        if state_view.state().get_partition(partition_id).is_none() {
            if !context.check_result_peak(4096) {
                return None;
            }
            return Some(storage_inconsistency_violation(
                class,
                format!(
                    "partition {:?} missing during selected-state uniqueness scan",
                    partition_id
                ),
                StorageInconsistencyContext::default()
                    .with_partition_id(partition_id)
                    .with_scan(StorageInconsistencyScan::HistoricalUniqueEntityAspectField)
                    .with_field(field.clone()),
            ));
        }
        let Some(slot_count) = state_view.entity_slot_scan_count(partition_id) else {
            continue;
        };
        for slot in 0..slot_count {
            if !context.checkpoint(1) {
                return None;
            }
            context.metrics().count_entity_slot_scans(1);
            let Some(metadata) = state_view.entity_metadata_for_slot(partition_id, slot) else {
                continue;
            };
            let state_bytes = state_view
                .entity_aspect_state(metadata.entity_id)
                .map_or(0, |state| state.owned_allocation_capacity_bytes() as u64);
            if !context.check_scratch_peak(state_bytes.saturating_mul(4).saturating_add(128)) {
                return None;
            }
            let Some(projected) =
                projected_entity_aspect_field_value_for_metadata(context, &metadata, field_locator)
            else {
                continue;
            };
            let value = projected.value;
            let value_bytes = value.owned_allocation_capacity_bytes() as u64;
            if !context.claim_scratch(value_bytes.saturating_mul(4).saturating_add(128)) {
                return None;
            }
            let comparison_key = authoritative_aspect_value_field_comparison_key(&value);
            if let Some((count, _)) = seen.get_mut(&comparison_key) {
                *count += 1;
            } else {
                seen.insert(comparison_key, (1usize, value));
            }
        }
    }
    for (count, value) in seen.into_values() {
        if count > 1 {
            let bytes = 4096_u64
                .saturating_add(field_locator.owned_allocation_capacity_bytes() as u64)
                .saturating_add(value.owned_allocation_capacity_bytes() as u64);
            if !context.claim_result(bytes) {
                return None;
            }
            return Some(duplicate_field_violation(class, field_locator, value));
        }
    }
    None
}

fn single_field(field_locator: &AspectFieldLocator) -> Option<&FieldKey> {
    match field_locator.field_path().fields() {
        [field] => Some(field),
        _ => None,
    }
}

fn duplicate_field_violation(
    class: InvariantClass,
    field_locator: &AspectFieldLocator,
    value: AspectValue,
) -> InvariantViolation {
    InvariantViolation {
        class,
        code: DiagnosticCode::InvariantViolation,
        detail: format!(
            "entity aspect field '{}:{}' must be unique, duplicate {:?} value",
            field_locator.aspect().aspect_key().as_str(),
            field_path_presentation_label(field_locator.field_path()),
            value.value_family()
        ),
        fields: InvariantViolationFields::UniqueEntityField {
            field_locator: field_locator.clone(),
            value,
        },
    }
}

fn field_path_presentation_label(path: &CanonicalFieldPath) -> String {
    path.fields()
        .iter()
        .map(FieldKey::as_str)
        .collect::<Vec<_>>()
        .join(".")
}
