use std::collections::BTreeSet;

use crate::indexes::data::{
    BoundedEntityFieldLookupDenial, BoundedEntityFieldLookupDenialKind,
    BoundedEntityFieldLookupOutcome, BoundedEntityFieldLookupRequest, BoundedIndexParityMode,
    DerivedIndexEntries, DerivedIndexKind,
};
use crate::storage::data::AuthoritativeFieldComparisonKey;
use crate::visibility::materialization::read_records::entity_query_locus_comparison_key;
use crate::visibility::snapshot_states::resolve_snapshot_handle;

use super::IndexAccess;

impl IndexAccess<'_> {
    pub fn execute_bounded_entity_field_lookup(
        &self,
        request: BoundedEntityFieldLookupRequest,
        parity_mode: BoundedIndexParityMode,
    ) -> Result<BoundedEntityFieldLookupOutcome, BoundedEntityFieldLookupDenial> {
        let runtime = self.runtime;
        runtime.performance_access().count_query_index_attempt();
        let snapshot = resolve_snapshot_handle(runtime, request.snapshot()).ok_or_else(|| {
            lookup_denial(
                BoundedEntityFieldLookupDenialKind::SnapshotUnavailable,
                &request,
            )
        })?;
        let projection = runtime
            .read_truth()
            .project_snapshot(&snapshot)
            .ok_or_else(|| {
                lookup_denial(
                    BoundedEntityFieldLookupDenialKind::SnapshotUnavailable,
                    &request,
                )
            })?;
        let source =
            super::super::projected_field_values::IndexProjectionSource::exact(&projection)
                .expect("resolved snapshot projection must carry an exact basis");
        let prepared = prepare_entity_field_lookup(runtime, &snapshot, &request)?;
        let indexed_ids = prepared.indexed_ids();
        let overflowed = indexed_ids.len() > request.candidate_limit();
        let examined_entry_count = indexed_ids.len().min(request.candidate_limit());
        let candidate_entity_ids = verify_bounded_index_entries(&source, &request, &indexed_ids)?;
        let outcome = BoundedEntityFieldLookupOutcome::new(
            prepared.generation_id,
            candidate_entity_ids,
            examined_entry_count,
            overflowed,
            parity_mode,
        );
        if parity_mode == BoundedIndexParityMode::Certification {
            certify_storage_parity(&source, &request, &outcome)?;
            runtime
                .performance_access()
                .count_query_index_parity_verification();
        }
        runtime.performance_access().count_query_index_path();
        Ok(outcome)
    }
}

/// The generation this lookup is pinned to, carried by shared ownership so the
/// entries stay readable without retaining the index subsystem lock.
struct PreparedEntityFieldLookup {
    generation_id: crate::indexes::data::DerivedIndexGenerationId,
    generation: std::sync::Arc<crate::indexes::data::DerivedIndexGeneration>,
    key: AuthoritativeFieldComparisonKey,
}

impl PreparedEntityFieldLookup {
    fn indexed_ids(
        &self,
    ) -> crate::indexes::data::DerivedIndexRows<crate::identity::data::EntityId> {
        let DerivedIndexEntries::EntityField(entries) = &self.generation.entries else {
            return Default::default();
        };
        entries.get(&self.key).cloned().unwrap_or_default()
    }
}

fn prepare_entity_field_lookup(
    runtime: &crate::runtime::RelationalRuntime,
    snapshot: &crate::snapshots::data::SnapshotHandle,
    request: &BoundedEntityFieldLookupRequest,
) -> Result<PreparedEntityFieldLookup, BoundedEntityFieldLookupDenial> {
    let definition = runtime
        .indexes
        .definition(request.index_id())
        .ok_or_else(|| {
            lookup_denial(
                BoundedEntityFieldLookupDenialKind::IndexNotInstalled,
                request,
            )
        })?;
    if !matches!(
        &definition.kind,
        DerivedIndexKind::EntityField { field_locator } if field_locator == request.field_locator()
    ) {
        return Err(lookup_denial(
            BoundedEntityFieldLookupDenialKind::WrongIndexKind,
            request,
        ));
    }
    let generation =
        super::generation_selection::exact_published_generation(runtime, snapshot, &definition)
            .ok_or_else(|| {
                lookup_denial(
                    BoundedEntityFieldLookupDenialKind::ExactGenerationUnavailable,
                    request,
                )
            })?;
    if !matches!(generation.entries, DerivedIndexEntries::EntityField(_)) {
        return Err(lookup_denial(
            BoundedEntityFieldLookupDenialKind::WrongIndexKind,
            request,
        ));
    }
    Ok(PreparedEntityFieldLookup {
        generation_id: generation.generation_id,
        key: AuthoritativeFieldComparisonKey::from_aspect_value(request.value()),
        generation,
    })
}

fn verify_bounded_index_entries(
    source: &super::super::projected_field_values::IndexProjectionSource<'_, '_>,
    request: &BoundedEntityFieldLookupRequest,
    indexed_ids: &crate::indexes::data::DerivedIndexRows<crate::identity::data::EntityId>,
) -> Result<Vec<crate::identity::data::EntityId>, BoundedEntityFieldLookupDenial> {
    let expected = AuthoritativeFieldComparisonKey::from_aspect_value(request.value());
    let mut seen = BTreeSet::new();
    let mut candidates = Vec::with_capacity(indexed_ids.len().min(request.candidate_limit()));
    for entity_id in indexed_ids.iter().take(request.candidate_limit()) {
        let record = source
            .with_entity(*entity_id, Clone::clone)
            .ok_or_else(|| corrupt_index_denial(request))?;
        if !seen.insert(record.entity_id)
            || entity_query_locus_comparison_key(&record, request.field_locator())
                != Some(expected.clone())
        {
            return Err(corrupt_index_denial(request));
        }
        if record.kind.kind_id == request.entity_kind() {
            candidates.push(record.entity_id);
        }
    }
    Ok(candidates)
}

fn certify_storage_parity(
    source: &super::super::projected_field_values::IndexProjectionSource<'_, '_>,
    request: &BoundedEntityFieldLookupRequest,
    indexed: &BoundedEntityFieldLookupOutcome,
) -> Result<(), BoundedEntityFieldLookupDenial> {
    let expected = AuthoritativeFieldComparisonKey::from_aspect_value(request.value());
    let mut storage_ids = Vec::new();
    source.for_each_entity(request.entity_kind(), |record| {
        if entity_query_locus_comparison_key(record, request.field_locator())
            == Some(expected.clone())
        {
            storage_ids.push(record.entity_id);
        }
    });
    storage_ids.sort();
    let storage_overflowed = storage_ids.len() > request.candidate_limit();
    storage_ids.truncate(request.candidate_limit());
    if storage_ids == indexed.candidate_entity_ids() && storage_overflowed == indexed.overflowed() {
        Ok(())
    } else {
        Err(lookup_denial(
            BoundedEntityFieldLookupDenialKind::StorageParityMismatch,
            request,
        ))
    }
}

fn corrupt_index_denial(
    request: &BoundedEntityFieldLookupRequest,
) -> BoundedEntityFieldLookupDenial {
    lookup_denial(
        BoundedEntityFieldLookupDenialKind::CorruptIndexEntries,
        request,
    )
}

fn lookup_denial(
    kind: BoundedEntityFieldLookupDenialKind,
    request: &BoundedEntityFieldLookupRequest,
) -> BoundedEntityFieldLookupDenial {
    BoundedEntityFieldLookupDenial::new(kind, request.index_id())
}
