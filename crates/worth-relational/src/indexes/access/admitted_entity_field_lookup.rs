use worth_foundational::facade::{
    AspectFieldLocator, AspectValue, AuthoritativeRecordAspectState,
    ContractValidatedAspectValueView,
};

use crate::branch::RelationalPartitionVisit;
use crate::identity::data::{EntityId, KindId};
use crate::indexes::data::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenial,
    BoundedEntityFieldLookupDenialKind as Denial, BoundedEntityFieldLookupOutcome,
    BoundedIndexParityMode, DerivedIndexEntries, DerivedIndexId, MAX_BOUNDED_INDEX_CANDIDATES,
};
use crate::runtime::{ExactLookupInputs, VisibilityProjectionView};
use crate::storage::data::AuthoritativeFieldComparisonKey;
use crate::storage::overlay::PartitionAccess;

use super::IndexAccess;

mod navigation;
use navigation::{btree_navigation_work, ordered_navigation_work};

impl IndexAccess<'_> {
    /// The exact selected-basis variant of the ordinary bounded lookup. The
    /// issued view supplies snapshot and schema authority; each index read,
    /// native candidate comparison and Certification storage visit uses the
    /// same caller admission before its work or allocation.
    pub fn execute_bounded_entity_field_lookup_admitted<E>(
        &self,
        view: &VisibilityProjectionView<'_>,
        index: DerivedIndexId,
        kind: KindId,
        locator: &AspectFieldLocator,
        value: &AspectValue,
        limit: usize,
        parity: BoundedIndexParityMode,
        mut prepare: impl FnMut(u64, u64) -> Result<(), E>,
    ) -> Result<BoundedEntityFieldLookupOutcome, Stop<E>> {
        if !view.is_exact_basis() || !view.is_from_runtime(self.runtime) {
            return Err(Stop::ExactBasisRequired);
        }
        if limit == 0 || limit > MAX_BOUNDED_INDEX_CANDIDATES {
            return Err(denied(Denial::InvalidCandidateLimit, index));
        }
        let schema = view
            .selected_schema_authority()
            .ok_or(Stop::ExactBasisRequired)?
            .schema_version();
        self.runtime
            .performance_access()
            .count_query_index_attempt();
        let selected = self
            .runtime
            .indexes
            .exact_lookup_inputs_admitted(
                index,
                locator,
                view.selected_branch_id(),
                view.version_id(),
                schema,
                &mut prepare,
            )
            .map_err(|stop| match stop {
                crate::indexes::data::SelectedIndexGenerationAdmissionStop::Admission(stop) => {
                    Stop::Admission(stop)
                }
                crate::indexes::data::SelectedIndexGenerationAdmissionStop::AccountingOverflow => {
                    Stop::AccountingOverflow
                }
            })?;
        let (definition, generation) = match selected {
            ExactLookupInputs::MissingDefinition => {
                return Err(denied(Denial::IndexNotInstalled, index))
            }
            ExactLookupInputs::MissingGeneration => {
                return Err(denied(Denial::ExactGenerationUnavailable, index))
            }
            ExactLookupInputs::WrongKind => return Err(denied(Denial::WrongIndexKind, index)),
            ExactLookupInputs::Ready {
                definition,
                generation,
            } => (definition, generation),
        };
        let DerivedIndexEntries::EntityField(entries) = &generation.entries else {
            return Err(denied(Denial::WrongIndexKind, index));
        };

        // The codec's length probe inspects the variant and at most two
        // borrowed text lengths. Actual initialized bytes are paid by the
        // subsequent encoding claim.
        charge(&mut prepare, 3, 0)?;
        let key_bytes = AuthoritativeFieldComparisonKey::required_encoded_capacity_bytes(value)
            .ok_or(Stop::AccountingOverflow)?;
        charge(
            &mut prepare,
            key_bytes.checked_add(1).ok_or(Stop::AccountingOverflow)?,
            key_bytes,
        )?;
        let key = AuthoritativeFieldComparisonKey::from_aspect_value(value);
        let seek = if entries.is_empty() {
            1
        } else {
            ordered_navigation_work(entries.len())?
                .checked_mul(key_bytes.checked_add(1).ok_or(Stop::AccountingOverflow)?)
                .ok_or(Stop::AccountingOverflow)?
        };
        charge(&mut prepare, seek, 0)?;
        let rows = entries.get(&key);
        let mut read = AdmittedCollectionRead {
            view,
            locator,
            value,
            prepare: &mut prepare,
        };
        let (candidates, examined, overflowed) =
            super::entity_field_collection::collect(rows, index, kind, limit, &mut read)?;
        let outcome = BoundedEntityFieldLookupOutcome::new(
            definition,
            generation.generation_id,
            candidates,
            examined,
            overflowed,
            parity,
        );
        if parity == BoundedIndexParityMode::Certification {
            certify_storage(
                view,
                kind,
                locator,
                value,
                index,
                limit,
                &outcome,
                &mut prepare,
            )?;
            self.runtime
                .performance_access()
                .count_query_index_parity_verification();
        }
        self.runtime.performance_access().count_query_index_path();
        Ok(outcome)
    }
}

struct AdmittedCollectionRead<'a, 'runtime, F> {
    view: &'a VisibilityProjectionView<'runtime>,
    locator: &'a AspectFieldLocator,
    value: &'a AspectValue,
    prepare: &'a mut F,
}
impl<E, F: FnMut(u64, u64) -> Result<(), E>>
    super::entity_field_collection::EntityFieldCollectionRead<E>
    for AdmittedCollectionRead<'_, '_, F>
{
    fn prepare_results(&mut self, examined: usize) -> Result<(), Stop<E>> {
        let bytes = width(examined)?
            .checked_mul(
                u64::try_from(std::mem::size_of::<EntityId>())
                    .map_err(|_| Stop::AccountingOverflow)?,
            )
            .and_then(|bytes| bytes.checked_mul(2))
            .ok_or(Stop::AccountingOverflow)?;
        charge(
            self.prepare,
            width(examined)?
                .checked_mul(2)
                .ok_or(Stop::AccountingOverflow)?,
            bytes,
        )
    }
    fn compare(
        &mut self,
        entity: EntityId,
        previous: usize,
    ) -> Result<Option<(KindId, bool)>, Stop<E>> {
        charge(self.prepare, width(previous + 1)?, 0)?;
        Ok(
            borrowed_matches(self.view, entity, self.locator, self.value, self.prepare)?
                .map(|(_, kind, matches)| (kind, matches)),
        )
    }
}

fn borrowed_matches<E>(
    view: &VisibilityProjectionView<'_>,
    entity: EntityId,
    locator: &AspectFieldLocator,
    expected: &AspectValue,
    prepare: &mut impl FnMut(u64, u64) -> Result<(), E>,
) -> Result<Option<(EntityId, KindId, bool)>, Stop<E>> {
    let read = view
        .exact_entity_state_read_work_bound()
        .ok_or(Stop::ExactBasisRequired)?;
    charge(prepare, read, 0)?;
    view.with_exact_entity_state(entity, |metadata, state| {
        let matches = state_matches(state, locator, expected, prepare)?;
        Ok((metadata.entity_id, metadata.kind_id, matches))
    })
    .map_err(|_| Stop::ExactBasisRequired)?
    .transpose()
}

fn state_matches<E>(
    state: Option<&AuthoritativeRecordAspectState>,
    locator: &AspectFieldLocator,
    expected: &AspectValue,
    prepare: &mut impl FnMut(u64, u64) -> Result<(), E>,
) -> Result<bool, Stop<E>> {
    let Some(state) = state else { return Ok(false) };
    let aspect = locator.aspect().aspect_key();
    charge(
        prepare,
        btree_navigation_work(state.aspects().len(), aspect.as_str().len())?,
        0,
    )?;
    let Some(validated) = state.get(aspect) else {
        return Ok(false);
    };
    let actual = match validated.view() {
        ContractValidatedAspectValueView::Scalar(value) => Some(value),
        ContractValidatedAspectValueView::Struct(value) => {
            let [field] = locator.field_path().fields() else {
                return Ok(false);
            };
            charge(
                prepare,
                btree_navigation_work(value.len(), field.as_str().len())?,
                0,
            )?;
            value.get(field)
        }
    };
    let Some(actual) = actual else {
        return Ok(false);
    };
    let comparison = actual
        .semantic_byte_width()
        .max(expected.semantic_byte_width());
    charge(
        prepare,
        width(comparison)?
            .checked_add(1)
            .ok_or(Stop::AccountingOverflow)?,
        0,
    )?;
    Ok(actual == expected)
}

fn certify_storage<E>(
    view: &VisibilityProjectionView<'_>,
    kind: KindId,
    locator: &AspectFieldLocator,
    value: &AspectValue,
    index: DerivedIndexId,
    limit: usize,
    outcome: &BoundedEntityFieldLookupOutcome,
    prepare: &mut impl FnMut(u64, u64) -> Result<(), E>,
) -> Result<(), Stop<E>> {
    let root = view.selected_root().ok_or(Stop::ExactBasisRequired)?;
    let slots = root.entity_slot_count();
    let bytes = width(slots)?
        .checked_mul(
            u64::try_from(std::mem::size_of::<EntityId>()).map_err(|_| Stop::AccountingOverflow)?,
        )
        .ok_or(Stop::AccountingOverflow)?;
    // The selected root supplies a finite ID inventory before the scan. This
    // also pays live-bitset iteration, including nonmatching live slots.
    charge(prepare, width(slots)?, bytes)?;
    let mut storage_ids = Vec::with_capacity(slots);
    root.try_visit_partitions(|event| {
        let RelationalPartitionVisit::Partition(partition_id) = event else {
            return charge(prepare, 1, 0);
        };
        charge(prepare, 33, 0)?;
        let Some(partition) = root.get_partition(partition_id) else {
            return Ok(());
        };
        for slot in partition.entity_arena.live_bitset.iter_set_slots() {
            let entity = EntityId::new(partition_id, slot as u64, 0);
            let Some((actual_id, actual_kind, matches)) =
                borrowed_matches(view, entity, locator, value, prepare)?
            else {
                continue;
            };
            if actual_kind == kind && matches {
                storage_ids.push(actual_id);
            }
        }
        Ok(())
    })?;
    let count = storage_ids.len();
    sort_fixed_ids(&mut storage_ids, prepare)?;
    charge(prepare, width(count.min(limit))?, 0)?;
    let overflowed = count > limit;
    storage_ids.truncate(limit);
    if overflowed == outcome.overflowed() && storage_ids == outcome.candidate_entity_ids() {
        Ok(())
    } else {
        Err(Stop::Lookup(
            BoundedEntityFieldLookupDenial::new(Denial::StorageParityMismatch, index)
                .with_examined_entry_count(outcome.examined_entry_count()),
        ))
    }
}

/// Heap sort performs no allocation. Each fixed EntityId comparison and
/// initialized slot exchange is admitted immediately before it occurs.
fn sort_fixed_ids<E>(
    ids: &mut [EntityId],
    prepare: &mut impl FnMut(u64, u64) -> Result<(), E>,
) -> Result<(), Stop<E>> {
    for start in (0..ids.len() / 2).rev() {
        sift_down(ids, start, ids.len(), prepare)?;
    }
    for end in (1..ids.len()).rev() {
        charge(prepare, 2, 0)?;
        ids.swap(0, end);
        sift_down(ids, 0, end, prepare)?;
    }
    Ok(())
}

fn sift_down<E>(
    ids: &mut [EntityId],
    mut root: usize,
    end: usize,
    prepare: &mut impl FnMut(u64, u64) -> Result<(), E>,
) -> Result<(), Stop<E>> {
    loop {
        charge(prepare, 1, 0)?;
        let child = root
            .checked_mul(2)
            .and_then(|slot| slot.checked_add(1))
            .ok_or(Stop::AccountingOverflow)?;
        if child >= end {
            return Ok(());
        }
        let mut selected = child;
        if child + 1 < end {
            charge(prepare, 1, 0)?;
            if ids[child] < ids[child + 1] {
                selected = child + 1;
            }
        }
        charge(prepare, 1, 0)?;
        if ids[root] >= ids[selected] {
            return Ok(());
        }
        charge(prepare, 2, 0)?;
        ids.swap(root, selected);
        root = selected;
    }
}

fn charge<E>(
    prepare: &mut impl FnMut(u64, u64) -> Result<(), E>,
    work: u64,
    bytes: u64,
) -> Result<(), Stop<E>> {
    prepare(work, bytes).map_err(Stop::Admission)
}

fn width<E>(value: usize) -> Result<u64, Stop<E>> {
    u64::try_from(value).map_err(|_| Stop::AccountingOverflow)
}

fn denied<E>(kind: Denial, index: DerivedIndexId) -> Stop<E> {
    Stop::Lookup(BoundedEntityFieldLookupDenial::new(kind, index))
}
