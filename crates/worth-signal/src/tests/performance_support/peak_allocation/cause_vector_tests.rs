//! Real allocator evidence for selected fork-page request growth.
use super::measure;
use crate::data::persistent_vector::{PersistentVector, RetainedVectorMutationOutcome};
use crate::data::retained_storage::RetainedStoragePreparation as Work;

#[test]
fn selected_cause_pages_cover_real_new_allocation_without_unselected_history() {
    let small_history = measured_selected_edits(64, 8, 4_096, 4);
    let large_history = measured_selected_edits(1_024, 8, 4_096, 4);
    assert_eq!(small_history, large_history);
    measured_selected_edits(4_096, 4_096, 16, 32);
    measured_distinct_pages(4_096);
}

fn measured_distinct_pages(page_count: usize) {
    const PAGE_LEN: usize = 32;
    let mut source: PersistentVector<Vec<u8>> = (0..page_count * PAGE_LEN)
        .map(|index| {
            if index % PAGE_LEN == 0 {
                vec![7]
            } else {
                Vec::new()
            }
        })
        .collect();
    source
        .prepare_retained_charge(&mut Work::new(usize::MAX))
        .unwrap();
    let mut fork = source.fork_persistent();
    let selected: Vec<_> = (0..page_count).map(|page| page * PAGE_LEN).collect();
    let forecast = fork
        .selected_batch_request_growth_bound(&selected, false, &mut Work::new(usize::MAX))
        .unwrap()
        .bytes();
    let (draft, peak) = measure(|| {
        let mut draft = fork.fork_persistent();
        let mut work = Work::new(usize::MAX);
        for &index in &selected {
            draft
                .edit_with_retained_charge(index, &mut work, |value| value[0] ^= 1)
                .unwrap();
        }
        draft
    });
    assert!(peak.expect("tracked distinct page allocations") as u64 <= forecast);
    assert_eq!(draft.len(), page_count * PAGE_LEN);
    assert_eq!(source[0][0], 7);
    assert_eq!(fork[0][0], 7);
}

fn measured_selected_edits(
    base_len: usize,
    selected_existing: usize,
    payload_bytes: usize,
    appended: usize,
) -> u64 {
    let mut source: PersistentVector<Vec<u8>> =
        (0..base_len).map(|_| vec![7_u8; payload_bytes]).collect();
    source
        .prepare_retained_charge(&mut Work::new(usize::MAX))
        .unwrap();
    let mut first = source.fork_persistent();
    let edited = first
        .edit_with_retained_charge(0, &mut Work::new(usize::MAX), |value| value[0] = 9)
        .unwrap();
    assert!(matches!(
        edited,
        RetainedVectorMutationOutcome::Accounted { .. }
    ));
    let sibling = first.fork_persistent();
    let selected: Vec<_> = (0..selected_existing)
        .chain(base_len..base_len + appended)
        .collect();
    let forecast = first
        .selected_batch_request_growth_bound(&selected, true, &mut Work::new(usize::MAX))
        .unwrap()
        .bytes();
    let before = first.page_identities();
    let mut append = (0..appended)
        .map(|_| vec![3_u8; payload_bytes])
        .collect::<Vec<_>>();
    let (draft, peak) = measure(|| {
        let mut draft = first.fork_persistent();
        let mut work = Work::new(usize::MAX);
        for index in 0..selected_existing {
            let outcome = draft
                .edit_with_retained_charge(index, &mut work, |value| value[0] ^= 1)
                .unwrap();
            assert!(matches!(
                outcome,
                RetainedVectorMutationOutcome::Accounted { .. }
            ));
        }
        for value in append.drain(..) {
            let outcome = draft.push_with_retained_charge(value, &mut work).unwrap();
            assert!(matches!(
                outcome,
                RetainedVectorMutationOutcome::Accounted { .. }
            ));
        }
        draft
    });
    assert!(peak.expect("tracked selected page allocations") as u64 <= forecast);
    assert_eq!(draft.len(), base_len + appended);
    assert_eq!(sibling[0][0], 9);
    assert_eq!(first.page_identities(), before);
    forecast
}
