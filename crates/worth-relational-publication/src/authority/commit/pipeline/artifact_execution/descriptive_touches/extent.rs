//! The touch artifact's allocation declaration comes from its immutable inputs.
use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use super::*;
use worth_relational_contracts::internal_port as contracts;

pub(super) fn memory_bound(
    catalog: &worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::catalog_publication::CapturedIndexDefinitionCatalog,
    selected: &worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState,
    working: &worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState,
    deltas: &[CanonicalRecordAspectDelta],
    maximum: usize,
) -> Result<u64, TransactionCommitError> {
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
            visit_changed_fields(binding, delta.structural_change, |field, _| {
                let field_bytes = field.owned_allocation_capacity_bytes() as u64;
                // The field transition is borrowed; only its retained path is allocated.
                bytes = bytes.saturating_add(field_bytes)
                    .saturating_add(binding.aspect_key.owned_allocation_capacity_bytes() as u64)
                    .saturating_add(size_of::<worth_foundational::facade::FieldKey>() as u64);
                catalog.catalog.with_field(&binding.aspect_key, field, |definitions| {
                    for definition in definitions.iter().filter(|definition| definition.index_id.0 < catalog.next_index_id) {
                        if !index::targets_field(binding, &delta.target, field, definition) { continue; }
                        let locator = match (&delta.target, &definition.kind) {
                            (RecordRef::Entity(_), DerivedIndexKind::EntityField { field_locator })
                            | (RecordRef::Relation(_), DerivedIndexKind::RelationField { field_locator }) => field_locator,
                            _ => continue,
                        };
                        let old = index::authoritative_state(selected, &delta.target);
                        let new = index::authoritative_state(working, &delta.target);
                        for state in [old, new] {
                            if let Some(value) = worth_relational_branch::internal_port::visibility::materialization::read_records::projection::query_locus_projection::authoritative_state_query_locus_value(state, locator) {
                                bytes = bytes.saturating_add(contracts::storage::data::authoritative_field_comparison_key::AuthoritativeFieldComparisonKey::required_encoded_capacity_bytes(value).unwrap_or(u64::MAX));
                            }
                        }
                        bytes = bytes.saturating_add(binding.aspect_key.owned_allocation_capacity_bytes() as u64)
                            .saturating_add(field_bytes).saturating_add(size_of::<worth_foundational::facade::FieldKey>() as u64);
                    }
                });
                Ok::<(), Failure>(())
            }).map_err(|failure| TransactionCommitError::execution(contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial { kind: contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(contracts::execution::denial::kernel_failure(failure)), partition_identity: None }))?;
        }
    }
    Ok(bytes)
}
