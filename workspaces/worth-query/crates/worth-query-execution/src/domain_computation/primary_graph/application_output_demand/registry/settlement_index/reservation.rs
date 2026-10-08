//! A prepared posting reuses the selected ordered-index search where possible.

use std::collections::btree_map::Entry;
use std::sync::atomic::Ordering;

use super::*;

struct AdmittedGrowth {
    retained: usize,
    required: usize,
}

/// Borrowed, charged forecast only. Reclamation can change both tree roots;
/// the eventual insertion must perform its own search and admission.
pub(super) fn quote(
    state: &mut DemandRegistryState,
    identity: &RecordedSettlementIdentity,
    admission: &mut InvalidationEditAdmission,
) -> Result<usize, WorthQueryOutputDemandDenial> {
    state.drain_cancelled_settlement_vacancies(admission)?;
    let sources = &state.settlement_keys.sources;
    admission
        .charge_ordered_operations(
            1,
            tree_lookup_work::<SemanticSource>(sources.len()).ok_or_else(work_denial)?,
        )
        .map_err(|_| work_denial())?;
    let (addresses, new_source) = if let Some(source) = sources.get(identity.source()) {
        admission
            .charge_ordered_operations(
                1,
                tree_lookup_work::<Address>(source.len()).ok_or_else(work_denial)?,
            )
            .map_err(|_| work_denial())?;
        if source.contains_key(&identity.address()) {
            return Err(WorthQueryOutputDemandDenial::new(
                Kind::SchedulingDeferred,
                "another attempt owns the exact settlement address",
            ));
        }
        (source.len(), false)
    } else {
        (0, true)
    };
    retained_growth(sources.len(), addresses, new_source)
}

pub(super) fn reserve(
    state: &mut DemandRegistryState,
    identity: &Arc<RecordedSettlementIdentity>,
    admission: &mut InvalidationEditAdmission,
) -> Result<(Posting, Box<PendingVacancyCleanup>), WorthQueryOutputDemandDenial> {
    state.drain_cancelled_settlement_vacancies(admission)?;
    let reserved = state.required_reserved_bytes;
    let shared_custody = state
        .required_custody_retained_bytes
        .load(Ordering::Acquire);
    let budget = state.required_budget_bytes;
    let sources = &mut state.settlement_keys.sources;
    let source_count = sources.len();
    admission
        .charge_ordered_operations(
            1,
            tree_lookup_work::<SemanticSource>(source_count).ok_or_else(work_denial)?,
        )
        .map_err(|_| work_denial())?;

    let (posting, cleanup, growth) = if let Some(source) = sources.get_mut(identity.source()) {
        let address_count = source.len();
        admission
            .charge_ordered_operations(
                1,
                tree_lookup_work::<Address>(address_count).ok_or_else(work_denial)?,
            )
            .map_err(|_| work_denial())?;
        let Entry::Vacant(vacant) = source.entry(identity.address()) else {
            return Err(WorthQueryOutputDemandDenial::new(
                Kind::SchedulingDeferred,
                "another attempt owns the exact settlement address",
            ));
        };
        let growth = admit_growth(
            source_count,
            address_count,
            false,
            reserved,
            shared_custody,
            budget,
            admission,
        )?;
        let (posting, cleanup) = new_vacancy(identity);
        // VacantEntry carries the already admitted search into the insertion.
        vacant.insert(Arc::clone(&posting));
        (posting, cleanup, growth)
    } else {
        // This source is absent. Its new inner map has one empty-root insertion;
        // insertion into the outer map performs a second admitted source search.
        let extra_work = tree_lookup_work::<SemanticSource>(source_count)
            .and_then(|outer| outer.checked_add(tree_lookup_work::<Address>(0)?))
            .ok_or_else(work_denial)?;
        admission
            .charge_ordered_operations(1, extra_work)
            .map_err(|_| work_denial())?;
        let growth = admit_growth(
            source_count,
            0,
            true,
            reserved,
            shared_custody,
            budget,
            admission,
        )?;
        let (posting, cleanup) = new_vacancy(identity);
        let mut source = SourcePostings::new();
        source.insert(identity.address(), Arc::clone(&posting));
        sources.insert(identity.source().clone(), source);
        (posting, cleanup, growth)
    };
    state.required_reserved_bytes = growth.required;
    state.settlement_keys.retained_bytes += growth.retained;
    Ok((posting, cleanup))
}

#[allow(clippy::too_many_arguments)]
fn admit_growth(
    source_count: usize,
    address_count: usize,
    new_source: bool,
    reserved: usize,
    shared_custody: usize,
    budget: usize,
    admission: &mut InvalidationEditAdmission,
) -> Result<AdmittedGrowth, WorthQueryOutputDemandDenial> {
    let retained = retained_growth(source_count, address_count, new_source)?;
    let required = reserved.checked_add(retained).ok_or_else(capacity_denial)?;
    if required
        .checked_add(shared_custody)
        .is_none_or(|total| total > budget)
    {
        return Err(capacity_denial());
    }
    let insertion = tree_insert_bytes::<Address, Posting>(address_count)
        .and_then(|bytes| {
            if new_source {
                bytes.checked_add(tree_insert_bytes::<SemanticSource, SourcePostings>(
                    source_count,
                )?)
            } else {
                Some(bytes)
            }
        })
        .ok_or_else(capacity_denial)?;
    // Existing sources clone only the identity and posting Arc. A newly
    // inserted source also clones its fixed structural key.
    admission
        .charge_external_work(if new_source { 3 } else { 2 })
        .map_err(|_| work_denial())?;
    admission
        .admit_read_scratch(
            insertion
                .checked_add(posting_cell_bytes() as u64)
                .and_then(|bytes| bytes.checked_add(size_of::<PendingVacancyCleanup>() as u64))
                .ok_or_else(capacity_denial)?,
        )
        .map_err(|_| capacity_denial())?;
    Ok(AdmittedGrowth { retained, required })
}

fn retained_growth(
    source_count: usize,
    address_count: usize,
    new_source: bool,
) -> Result<usize, WorthQueryOutputDemandDenial> {
    let old_outer = tree_retained_bytes::<SemanticSource, SourcePostings>(source_count)
        .ok_or_else(capacity_denial)?;
    let new_outer = tree_retained_bytes::<SemanticSource, SourcePostings>(
        source_count
            .checked_add(usize::from(new_source))
            .ok_or_else(capacity_denial)?,
    )
    .ok_or_else(capacity_denial)?;
    let old_inner =
        tree_retained_bytes::<Address, Posting>(address_count).ok_or_else(capacity_denial)?;
    let new_inner = tree_retained_bytes::<Address, Posting>(
        address_count.checked_add(1).ok_or_else(capacity_denial)?,
    )
    .ok_or_else(capacity_denial)?;
    new_outer
        .checked_sub(old_outer)
        .and_then(|bytes| bytes.checked_add(new_inner.checked_sub(old_inner)?))
        .and_then(|bytes| bytes.checked_add(posting_cell_bytes()))
        .and_then(|bytes| bytes.checked_add(size_of::<PendingVacancyCleanup>()))
        .ok_or_else(capacity_denial)
}

fn new_vacancy(
    identity: &Arc<RecordedSettlementIdentity>,
) -> (Posting, Box<PendingVacancyCleanup>) {
    let posting = Arc::new(Mutex::new(None));
    let cleanup = Box::new(PendingVacancyCleanup {
        identity: Arc::clone(identity),
        next: None,
    });
    (posting, cleanup)
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_relational::facade::mvcc::CompanionPreflightBudget;

    fn admission(work: u64, bytes: u64) -> InvalidationEditAdmission {
        InvalidationEditAdmission::new(CompanionPreflightBudget {
            maximum_work_visits: work,
            maximum_preparation_bytes: bytes,
        })
    }

    fn first_posting(identity: &Arc<RecordedSettlementIdentity>) -> DemandRegistryState {
        let mut state = DemandRegistryState::default();
        state
            .reserve_settlement_vacancy(identity, &mut admission(1_000_000, 8 * 1024 * 1024))
            .expect("the first actual lineage address is admitted");
        state
    }

    #[test]
    fn quote_rechecks_a_retired_source_root_before_reserving_its_successor() {
        let (_lineage, [first, second]) = crate::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement_pair();
        let mut state = DemandRegistryState::default();
        let mut work = admission(1_000_000, 8 * 1024 * 1024);
        let (posting, cleanup) = state.reserve_settlement_vacancy(&first, &mut work).unwrap();
        let prior = state.required_reserved_bytes;
        let existing = quote(&mut state, &second, &mut work).unwrap();
        assert_eq!(
            state.required_reserved_bytes, prior,
            "a quote reserves nothing"
        );
        assert_eq!(state.settlement_keys.sources[first.source()].len(), 1);
        state.defer_cancelled_settlement_vacancy(cleanup);
        drop(posting);
        state
            .drain_cancelled_settlement_vacancies(&mut work)
            .unwrap();
        assert_eq!(state.required_reserved_bytes, 0);
        let absent = quote(&mut state, &second, &mut work).unwrap();
        assert!(
            absent > existing,
            "retirement removed independently rooted trees"
        );
        let (_posting, _cleanup) = state
            .reserve_settlement_vacancy(&second, &mut work)
            .unwrap();
        assert_eq!(state.required_reserved_bytes, absent);
        assert_eq!(state.settlement_keys.sources[second.source()].len(), 1);
    }

    #[test]
    fn held_address_entry_denies_before_insertion_and_retries_at_exact_capacity() {
        let (_lineage, [first, second]) = crate::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement_pair();
        assert!(first.source() == second.source());
        assert!(first.address() != second.address());

        let mut measured_state = first_posting(&first);
        let mut measured = admission(1_000_000, 8 * 1024 * 1024);
        measured_state
            .reserve_settlement_vacancy(&second, &mut measured)
            .expect("the second address under the same source is admissible");
        let exact_work = measured.charged_work();
        let exact_bytes = measured.charged_bytes();
        assert!(exact_work > 1);
        assert!(exact_bytes > 0);

        let mut state = first_posting(&first);
        let retained_before = state.required_reserved_bytes;
        let source = state.settlement_keys.sources.get(first.source()).unwrap();
        assert_eq!(source.len(), 1);
        let denial = state
            .reserve_settlement_vacancy(&second, &mut admission(exact_work, exact_bytes - 1))
            .err()
            .expect("one short scratch byte rejects the vacant entry");
        assert_eq!(denial.kind(), Kind::RetentionBudgetExceeded);
        assert_eq!(state.required_reserved_bytes, retained_before);
        assert_eq!(state.settlement_keys.sources[first.source()].len(), 1);

        let denial = state
            .reserve_settlement_vacancy(&second, &mut admission(exact_work - 1, exact_bytes))
            .err()
            .expect("one short work visit rejects before insertion");
        assert_eq!(denial.kind(), Kind::WorkBudgetExceeded);
        assert_eq!(state.required_reserved_bytes, retained_before);
        assert_eq!(state.settlement_keys.sources[first.source()].len(), 1);

        state
            .reserve_settlement_vacancy(&second, &mut admission(exact_work, exact_bytes))
            .expect("the same vacant entry succeeds at its measured budget");
        assert_eq!(state.settlement_keys.sources[first.source()].len(), 2);
        assert!(state.required_reserved_bytes > retained_before);
    }
}
