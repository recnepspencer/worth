use crate::indexes::data::{
    BoundedIndexParityMode, BoundedRelatedEntityOrderedLookupDenial,
    BoundedRelatedEntityOrderedLookupDenialKind, BoundedRelatedEntityOrderedLookupOutcome,
    BoundedRelatedEntityOrderedLookupRequest, DerivedIndexEntries, DerivedIndexKind,
    RelatedEntityEndpoint, RelatedEntityOrderingBoundary, RelatedEntityOrderingEntry,
    RelatedEntityOrderingField,
};
use crate::runtime::RelationalRuntime;
use crate::visibility::snapshot_states::resolve_snapshot_handle;

use super::super::projected_field_values::{
    build_related_entity_ordering_index, compare_related_entries, RelatedEntityOrderingProjection,
};
use super::IndexAccess;

impl IndexAccess<'_> {
    pub fn execute_bounded_related_entity_ordered_lookup(
        &self,
        request: BoundedRelatedEntityOrderedLookupRequest,
        parity_mode: BoundedIndexParityMode,
    ) -> Result<BoundedRelatedEntityOrderedLookupOutcome, BoundedRelatedEntityOrderedLookupDenial>
    {
        let runtime = self.runtime;
        runtime.performance_access().count_query_index_attempt();
        let snapshot = resolve_snapshot_handle(runtime, request.snapshot()).ok_or_else(|| {
            denial(
                BoundedRelatedEntityOrderedLookupDenialKind::SnapshotUnavailable,
                &request,
            )
        })?;
        let projection = runtime
            .read_truth()
            .project_snapshot(&snapshot)
            .ok_or_else(|| {
                denial(
                    BoundedRelatedEntityOrderedLookupDenialKind::SnapshotUnavailable,
                    &request,
                )
            })?;
        let source =
            super::super::projected_field_values::IndexProjectionSource::exact(&projection)
                .expect("resolved snapshot projection must carry an exact basis");
        let prepared = prepare_related_lookup(runtime, &snapshot, &request)?;
        let page = select_page(
            &prepared.parent_entries(),
            prepared.contract.ordering(),
            &request,
        )?;
        verify_entries(&source, &prepared.contract, &page.rows)?;
        let outcome = outcome_from_page(prepared.generation_id, page, parity_mode);
        if parity_mode == BoundedIndexParityMode::Certification {
            certify_storage_parity(&source, &prepared.contract, &outcome)?;
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
struct PreparedRelatedLookup<'request> {
    generation_id: crate::indexes::data::DerivedIndexGenerationId,
    generation: std::sync::Arc<crate::indexes::data::DerivedIndexGeneration>,
    contract: RelatedLookupContract<'request>,
}

impl PreparedRelatedLookup<'_> {
    fn parent_entries(&self) -> crate::indexes::data::DerivedIndexRows<RelatedEntityOrderingEntry> {
        let DerivedIndexEntries::RelatedEntityOrdering(entries) = &self.generation.entries else {
            return Default::default();
        };
        entries
            .get(&self.contract.parent)
            .cloned()
            .unwrap_or_default()
    }
}

/// The lookup shape this request is bound to, holding its definition by shared
/// ownership so the ordering fields outlive the subsystem read that found them.
struct RelatedLookupContract<'request> {
    definition: std::sync::Arc<crate::indexes::data::DerivedIndexDefinition>,
    relation_kind: crate::identity::data::KindId,
    parent_endpoint: RelatedEntityEndpoint,
    child_kind: crate::identity::data::KindId,
    parent: crate::identity::data::EntityId,
    request: &'request BoundedRelatedEntityOrderedLookupRequest,
}

impl RelatedLookupContract<'_> {
    fn ordering(&self) -> &[RelatedEntityOrderingField] {
        let DerivedIndexKind::RelatedEntityOrdering { ordering, .. } = &self.definition.kind else {
            return &[];
        };
        ordering
    }
}

fn prepare_related_lookup<'request>(
    runtime: &RelationalRuntime,
    snapshot: &crate::snapshots::data::SnapshotHandle,
    request: &'request BoundedRelatedEntityOrderedLookupRequest,
) -> Result<PreparedRelatedLookup<'request>, BoundedRelatedEntityOrderedLookupDenial> {
    let contract = resolve_related_lookup_contract(runtime, request)?;
    let generation = super::generation_selection::exact_published_generation(
        runtime,
        snapshot,
        &contract.definition,
    )
    .ok_or_else(|| {
        denial(
            BoundedRelatedEntityOrderedLookupDenialKind::ExactGenerationUnavailable,
            request,
        )
    })?;
    if request
        .expected_generation()
        .is_some_and(|expected| expected != generation.generation_id)
    {
        return Err(denial(
            BoundedRelatedEntityOrderedLookupDenialKind::ExpectedGenerationMismatch,
            request,
        ));
    }
    if !matches!(
        generation.entries,
        DerivedIndexEntries::RelatedEntityOrdering(_)
    ) {
        return Err(denial(
            BoundedRelatedEntityOrderedLookupDenialKind::WrongIndexKind,
            request,
        ));
    }
    Ok(PreparedRelatedLookup {
        generation_id: generation.generation_id,
        generation,
        contract,
    })
}

fn resolve_related_lookup_contract<'request>(
    runtime: &RelationalRuntime,
    request: &'request BoundedRelatedEntityOrderedLookupRequest,
) -> Result<RelatedLookupContract<'request>, BoundedRelatedEntityOrderedLookupDenial> {
    let definition = runtime
        .indexes
        .definition(request.index_id())
        .ok_or_else(|| {
            denial(
                BoundedRelatedEntityOrderedLookupDenialKind::IndexNotInstalled,
                request,
            )
        })?;
    let DerivedIndexKind::RelatedEntityOrdering {
        relation_kind,
        parent_endpoint,
        child_kind,
        ..
    } = &definition.kind
    else {
        return Err(denial(
            BoundedRelatedEntityOrderedLookupDenialKind::WrongIndexKind,
            request,
        ));
    };
    if *child_kind != request.child_kind() {
        return Err(denial(
            BoundedRelatedEntityOrderedLookupDenialKind::WrongIndexKind,
            request,
        ));
    }
    let (relation_kind, parent_endpoint, child_kind) =
        (*relation_kind, *parent_endpoint, *child_kind);
    Ok(RelatedLookupContract {
        definition,
        relation_kind,
        parent_endpoint,
        child_kind,
        parent: request.parent_entity_id(),
        request,
    })
}

struct SelectedPage {
    rows: Vec<RelatedEntityOrderingEntry>,
    has_more: bool,
    seek_comparison_count: usize,
}

fn select_page(
    entries: &crate::indexes::data::DerivedIndexRows<RelatedEntityOrderingEntry>,
    ordering: &[RelatedEntityOrderingField],
    request: &BoundedRelatedEntityOrderedLookupRequest,
) -> Result<SelectedPage, BoundedRelatedEntityOrderedLookupDenial> {
    let mut seek_comparison_count = 0;
    let start = match request.after() {
        None => 0,
        Some(boundary) => entries
            .binary_search_by(|entry| {
                seek_comparison_count += 1;
                compare_related_entries(entry, boundary.entry(), ordering)
            })
            .map(|index| index + 1)
            .map_err(|_| {
                denial(
                    BoundedRelatedEntityOrderedLookupDenialKind::ForeignBoundary,
                    request,
                )
            })?,
    };
    let available = entries.len().saturating_sub(start);
    let returned = available.min(request.page_width());
    Ok(SelectedPage {
        rows: (start..start + returned)
            .map(|i| entries.get(i).expect("bounded page index").clone())
            .collect(),
        has_more: available > returned,
        seek_comparison_count,
    })
}

fn outcome_from_page(
    generation_id: crate::indexes::data::DerivedIndexGenerationId,
    page: SelectedPage,
    parity_mode: BoundedIndexParityMode,
) -> BoundedRelatedEntityOrderedLookupOutcome {
    let child_entity_ids = page
        .rows
        .iter()
        .map(RelatedEntityOrderingEntry::child_entity_id)
        .collect();
    let next_boundary = page
        .has_more
        .then(|| {
            page.rows
                .last()
                .map(RelatedEntityOrderingBoundary::from_entry)
        })
        .flatten();
    BoundedRelatedEntityOrderedLookupOutcome::new(
        generation_id,
        child_entity_ids,
        next_boundary,
        page.rows.len() + page.seek_comparison_count,
        page.seek_comparison_count,
        parity_mode,
    )
}

fn verify_entries(
    projection: &super::super::projected_field_values::IndexProjectionSource<'_, '_>,
    contract: &RelatedLookupContract<'_>,
    entries: &[RelatedEntityOrderingEntry],
) -> Result<(), BoundedRelatedEntityOrderedLookupDenial> {
    for entry in entries {
        let relation = projection
            .with_relation(entry.relation_id(), Clone::clone)
            .ok_or_else(|| corrupt(contract.request))?;
        let (observed_parent, observed_child) = match contract.parent_endpoint {
            RelatedEntityEndpoint::SourceParent => (relation.source, relation.target),
            RelatedEntityEndpoint::TargetParent => (relation.target, relation.source),
        };
        if relation.kind.kind_id != contract.relation_kind
            || observed_parent != contract.parent
            || observed_child != entry.child_entity_id()
            || projection.with_entity(observed_child, |record| record.kind.kind_id)
                != Some(contract.child_kind)
        {
            return Err(corrupt(contract.request));
        }
        let values = contract
            .ordering()
            .iter()
            .map(|field| {
                super::super::projected_field_values::entity_aspect_field_ordering_value(
                    projection,
                    observed_child,
                    field.locator(),
                )
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| corrupt(contract.request))?;
        if !values
            .iter()
            .zip(entry.ordering_values())
            .all(|(value, expected)| value == expected.value())
            || values.len() != entry.ordering_values().len()
        {
            return Err(corrupt(contract.request));
        }
    }
    Ok(())
}

fn certify_storage_parity(
    projection: &super::super::projected_field_values::IndexProjectionSource<'_, '_>,
    contract: &RelatedLookupContract<'_>,
    indexed: &BoundedRelatedEntityOrderedLookupOutcome,
) -> Result<(), BoundedRelatedEntityOrderedLookupDenial> {
    let storage = build_related_entity_ordering_index(
        projection,
        &RelatedEntityOrderingProjection::new(
            contract.relation_kind,
            contract.parent_endpoint,
            contract.child_kind,
            contract.ordering(),
        ),
    );
    let expected = select_page(
        &storage
            .get(&contract.parent)
            .cloned()
            .unwrap_or_default()
            .into(),
        contract.ordering(),
        contract.request,
    )?;
    let expected = outcome_from_page(indexed.generation_id(), expected, indexed.parity_mode());
    if expected.child_entity_ids() == indexed.child_entity_ids()
        && expected.next_boundary() == indexed.next_boundary()
    {
        Ok(())
    } else {
        Err(denial(
            BoundedRelatedEntityOrderedLookupDenialKind::StorageParityMismatch,
            contract.request,
        ))
    }
}

fn corrupt(
    request: &BoundedRelatedEntityOrderedLookupRequest,
) -> BoundedRelatedEntityOrderedLookupDenial {
    denial(
        BoundedRelatedEntityOrderedLookupDenialKind::CorruptIndexEntries,
        request,
    )
}

fn denial(
    kind: BoundedRelatedEntityOrderedLookupDenialKind,
    request: &BoundedRelatedEntityOrderedLookupRequest,
) -> BoundedRelatedEntityOrderedLookupDenial {
    BoundedRelatedEntityOrderedLookupDenial::new(kind, request.index_id())
}
