//! The touch artifact's allocation declaration comes from its immutable inputs.
use super::*;

pub(super) fn memory_bound(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    selected: &crate::branch::SelectedRelationalBranchState,
    working: &crate::runtime::WorkingState,
    deltas: &[CanonicalRecordAspectDelta],
    maximum: usize,
    mode: &TouchPreparationMode,
    lease: &ExecutionResourceLease<'_>,
) -> Result<u64, TransactionCommitError> {
    if matches!(mode, TouchPreparationMode::NativeMerge) {
        // Full-partition merge growth retains its existing preparation bound.
        return Ok(lease.policy().budget().charged_memory_bytes() / 8);
    }
    let mut bytes = (maximum as u64)
        .saturating_mul(size_of::<Touch>() as u64)
        .saturating_mul(2);
    for delta in deltas {
        for aspect in &delta.changed_aspects {
            bytes = bytes.saturating_add(aspect.owned_allocation_capacity_bytes() as u64);
        }
        for binding in &delta.evaluated_bindings {
            if !binding.changed {
                continue;
            }
            visit_possible_fields(binding, |field| {
                let field_bytes = field.owned_allocation_capacity_bytes() as u64;
                // The changed-field scratch and a possible retained field path.
                bytes = bytes.saturating_add((2 * size_of::<(worth_foundational::facade::FieldKey, crate::storage::data::RelationalFieldPresence)>()) as u64)
                    .saturating_add(field_bytes)
                    .saturating_add(binding.aspect_key.owned_allocation_capacity_bytes() as u64)
                    .saturating_add(field_bytes).saturating_add(size_of::<worth_foundational::facade::FieldKey>() as u64);
                runtime.index_definitions.with_field(&binding.aspect_key, field, |definitions| {
                    for definition in definitions {
                        let locator = match (&delta.target, &definition.kind) {
                            (RecordRef::Entity(_), DerivedIndexKind::EntityField { field_locator })
                            | (RecordRef::Relation(_), DerivedIndexKind::RelationField { field_locator }) => field_locator,
                            _ => continue,
                        };
                        let old = index::authoritative_state(selected, &delta.target);
                        let new = index::authoritative_state(working, &delta.target);
                        for state in [old, new] {
                            if let Some(value) = crate::visibility::materialization::read_records::authoritative_state_query_locus_value(state, locator) {
                                bytes = bytes.saturating_add(crate::storage::data::AuthoritativeFieldComparisonKey::required_encoded_capacity_bytes(value).unwrap_or(u64::MAX));
                            }
                        }
                        bytes = bytes.saturating_add(binding.aspect_key.owned_allocation_capacity_bytes() as u64)
                            .saturating_add(field_bytes).saturating_add(size_of::<worth_foundational::facade::FieldKey>() as u64);
                    }
                });
                Ok::<(), Failure>(())
            }).map_err(|failure| TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial { kind: crate::transactions::data::CommitExecutionDenialKind::Cause(crate::execution::kernel_failure(failure)), partition_identity: None }))?;
        }
    }
    Ok(bytes)
}
