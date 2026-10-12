use crate::indexes::data::{
    BoundedIndexParityMode, BoundedRelatedEntityOrderedLookupDenial,
    BoundedRelatedEntityOrderedLookupDenialKind, BoundedRelatedEntityOrderedLookupOutcome,
    BoundedRelatedEntityOrderedLookupRequest, DerivedIndexEntries, DerivedIndexKind,
    RelatedEntityEndpoint, RelatedEntityOrderingBoundary, RelatedEntityOrderingEntry,
    RelatedEntityOrderingField,
};
use crate::runtime::RelationalPinnedRead;

use super::super::projected_field_values::{
    build_related_entity_ordering_index, compare_related_entries, RelatedEntityOrderingProjection,
};

impl RelationalPinnedRead {
    /// Traverse the ordered generation selected when this capability was pinned.
    pub fn execute_bounded_related_entity_ordered_lookup(
        &self,
        request: BoundedRelatedEntityOrderedLookupRequest,
        parity_mode: BoundedIndexParityMode,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BoundedRelatedEntityOrderedLookupOutcome, BoundedRelatedEntityOrderedLookupDenial>
    {
        self.performance_access().count_query_index_attempt();
        execution.check_live().map_err(|stop| {
            denial(
                BoundedRelatedEntityOrderedLookupDenialKind::Execution(
                    crate::execution::denial::kernel_stop(stop),
                ),
                &request,
            )
        })?;
        self.lookup_pinned_related_ordering(request, parity_mode, execution)
    }
}
impl RelationalPinnedRead {
    pub(super) fn lookup_pinned_related_ordering(
        &self,
        request: BoundedRelatedEntityOrderedLookupRequest,
        parity_mode: BoundedIndexParityMode,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BoundedRelatedEntityOrderedLookupOutcome, BoundedRelatedEntityOrderedLookupDenial>
    {
        let snapshot = request.snapshot();
        let projection = self.project_snapshot(snapshot, execution).ok_or_else(|| {
            denial(
                BoundedRelatedEntityOrderedLookupDenialKind::SnapshotUnavailable,
                &request,
            )
        })?;
        let source =
            super::super::projected_field_values::IndexProjectionSource::exact(&projection)
                .expect("resolved snapshot projection must carry an exact basis");
        let prepared = prepare_related_lookup(self, &request)?;
        let page = select_page(
            &prepared.parent_entries(),
            prepared.contract.ordering(),
            &request,
        )?;
        verify_entries(execution, &source, &prepared.contract, &page.rows)?;
        let outcome = outcome_from_page(prepared.generation_id, page, parity_mode);
        if parity_mode == BoundedIndexParityMode::Certification {
            certify_storage_parity(execution, &source, &prepared.contract, &outcome)?;
            self.performance_access()
                .count_query_index_parity_verification();
        }
        self.performance_access().count_query_index_path();
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
    read: &RelationalPinnedRead,
    request: &'request BoundedRelatedEntityOrderedLookupRequest,
) -> Result<PreparedRelatedLookup<'request>, BoundedRelatedEntityOrderedLookupDenial> {
    let contract = resolve_related_lookup_contract(read, request)?;
    let generation = read
        .selected_index(request.index_id())
        .and_then(|index| index.generation.clone())
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
    read: &RelationalPinnedRead,
    request: &'request BoundedRelatedEntityOrderedLookupRequest,
) -> Result<RelatedLookupContract<'request>, BoundedRelatedEntityOrderedLookupDenial> {
    let definition = read
        .selected_index(request.index_id())
        .map(|index| index.definition.clone())
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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    projection: &super::super::projected_field_values::IndexProjectionSource<'_>,
    contract: &RelatedLookupContract<'_>,
    entries: &[RelatedEntityOrderingEntry],
) -> Result<(), BoundedRelatedEntityOrderedLookupDenial> {
    for entry in entries {
        let relation = projection
            .with_relation(execution, entry.relation_id(), Clone::clone)
            .map_err(|cause| {
                denial(
                    BoundedRelatedEntityOrderedLookupDenialKind::Execution(cause),
                    contract.request,
                )
            })?
            .ok_or_else(|| corrupt(contract.request))?;
        let (observed_parent, observed_child) = match contract.parent_endpoint {
            RelatedEntityEndpoint::SourceParent => (relation.source, relation.target),
            RelatedEntityEndpoint::TargetParent => (relation.target, relation.source),
        };
        if relation.kind.kind_id != contract.relation_kind
            || observed_parent != contract.parent
            || observed_child != entry.child_entity_id()
            || projection
                .with_entity(execution, observed_child, |record| record.kind.kind_id)
                .map_err(|cause| {
                    denial(
                        BoundedRelatedEntityOrderedLookupDenialKind::Execution(cause),
                        contract.request,
                    )
                })?
                != Some(contract.child_kind)
        {
            return Err(corrupt(contract.request));
        }
        let values = contract
            .ordering()
            .iter()
            .map(|field| {
                super::super::projected_field_values::entity_aspect_field_ordering_value(
                    execution,
                    projection,
                    observed_child,
                    field.locator(),
                )
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|cause| {
                denial(
                    BoundedRelatedEntityOrderedLookupDenialKind::Execution(cause),
                    contract.request,
                )
            })?
            .into_iter()
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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    projection: &super::super::projected_field_values::IndexProjectionSource<'_>,
    contract: &RelatedLookupContract<'_>,
    indexed: &BoundedRelatedEntityOrderedLookupOutcome,
) -> Result<(), BoundedRelatedEntityOrderedLookupDenial> {
    let storage = build_related_entity_ordering_index(
        execution,
        projection,
        &RelatedEntityOrderingProjection::new(
            contract.relation_kind,
            contract.parent_endpoint,
            contract.child_kind,
            contract.ordering(),
        ),
    )
    .map_err(|cause| {
        denial(
            BoundedRelatedEntityOrderedLookupDenialKind::Execution(cause),
            contract.request,
        )
    })?;
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

pub(super) fn denial(
    kind: BoundedRelatedEntityOrderedLookupDenialKind,
    request: &BoundedRelatedEntityOrderedLookupRequest,
) -> BoundedRelatedEntityOrderedLookupDenial {
    BoundedRelatedEntityOrderedLookupDenial::new(kind, request.index_id())
}
