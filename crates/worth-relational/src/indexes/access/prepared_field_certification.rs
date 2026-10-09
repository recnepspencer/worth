use worth_foundational::facade::AspectValue;

use crate::indexes::data::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenial,
    BoundedEntityFieldLookupDenialKind, BoundedEntityFieldLookupOutcome,
};
use crate::runtime::{RelationalRuntime, VisibilityProjectionView};
use crate::storage::data::AuthoritativeFieldComparisonKey;
use crate::visibility::materialization::read_records::entity_query_locus_comparison_key;
use crate::visibility::snapshot_states::SnapshotStateBasis;

use super::PreparedEntityFieldLookup;

/// Certification deliberately traverses authoritative storage at the same
/// retained root. Production never constructs this materialized comparison.
pub(super) fn certify<E>(
    runtime: &RelationalRuntime,
    prepared: &PreparedEntityFieldLookup,
    value: &AspectValue,
    limit: usize,
    outcome: &BoundedEntityFieldLookupOutcome,
    check_live: &mut impl FnMut() -> Result<(), E>,
) -> Result<(), Stop<E>> {
    let view =
        VisibilityProjectionView::new(runtime, SnapshotStateBasis::Exact(prepared.basis().clone()));
    let expected = AuthoritativeFieldComparisonKey::from_aspect_value(value);
    let mut storage_ids = Vec::new();
    view.try_for_each_entity_record(
        Some(prepared.entity_kind()),
        |_| check_live().map_err(Stop::Admission),
        |record| {
            if entity_query_locus_comparison_key(record, prepared.field_locator()).as_ref()
                == Some(&expected)
            {
                storage_ids.push(record.entity_id);
            }
            Ok(())
        },
    )?;
    storage_ids.sort();
    check_live().map_err(Stop::Admission)?;
    let overflowed = storage_ids.len() > limit;
    storage_ids.truncate(limit);
    if storage_ids == outcome.candidate_entity_ids() && overflowed == outcome.overflowed() {
        Ok(())
    } else {
        Err(Stop::Lookup(
            BoundedEntityFieldLookupDenial::new(
                BoundedEntityFieldLookupDenialKind::StorageParityMismatch,
                prepared.index_id(),
            )
            .with_examined_entry_count(outcome.examined_entry_count()),
        ))
    }
}
