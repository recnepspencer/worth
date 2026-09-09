use super::*;
use crate::data::retained_storage::{arc_allocation_charge, ordered_index_charge};
use crate::diagnostics::replay::ReplayEventDetail;
use crate::diagnostics::state::{DiagnosticHistory, DiagnosticHistoryEditDenial};
use crate::tests::emitted_merge_replay_event;
use std::sync::Arc;

#[test]
fn retained_merge_replay_admits_exact_budget_and_preserves_denied_evidence() {
    let event = emitted_merge_replay_event();
    assert!(matches!(
        event.detail,
        Some(ReplayEventDetail::BranchMergeSummary { .. })
    ));
    let original: DiagnosticHistory<_> = [event.clone()].into_iter().collect();
    let old_charge = original
        .prepare_retained_charge(&mut Work::new(10000))
        .unwrap();
    let candidate = event.clone();
    let exact = appended_charge(old_charge, &candidate);
    let surviving_heap = candidate
        .retained_heap_charge(&mut Work::new(10000))
        .unwrap();
    let mut admitted = original.clone();
    let mut work = Work::new(10000);
    admitted
        .append_with_retained_capacity(candidate, exact, &mut work)
        .unwrap();
    let visits = work.visits();
    for (limit, byte_short) in [(visits, false), (visits, true), (visits - 1, false)] {
        let candidate = event.clone();
        let required = appended_charge(old_charge, &candidate);
        let message_allocation = candidate
            .detail
            .as_ref()
            .unwrap()
            .as_message()
            .unwrap()
            .as_ptr();
        let maximum = if byte_short {
            required
                .checked_sub(Charge::capacity::<u8>(1).unwrap())
                .unwrap()
        } else {
            required
        };
        let mut draft = original.clone();
        let result = draft.append_with_retained_capacity(candidate, maximum, &mut Work::new(limit));
        if !byte_short && limit == visits {
            result.unwrap();
            assert_eq!(draft, admitted);
            assert_eq!(draft.prepared_retained_charge().unwrap(), required);
            continue;
        }
        let (returned, denial) = result.unwrap_err();
        assert_eq!(returned, event);
        assert_eq!(
            returned
                .detail
                .as_ref()
                .unwrap()
                .as_message()
                .unwrap()
                .as_ptr(),
            message_allocation
        );
        assert_eq!(draft, original);
        assert!(std::ptr::eq(
            draft.front().unwrap(),
            original.front().unwrap()
        ));
        assert_eq!(draft.prepared_retained_charge().unwrap(), old_charge);
        let expected = if byte_short {
            DiagnosticHistoryEditDenial::CapacityExhausted { maximum, required }
        } else {
            DiagnosticHistoryEditDenial::Accounting(Denial::WorkExhausted {
                maximum_visits: limit,
            })
        };
        assert_eq!(denial, expected);
    }
    let retained = admitted
        .evict_front_with_retained_charge(&mut Work::new(10000))
        .unwrap()
        .unwrap();
    assert!(std::ptr::eq(
        Arc::as_ptr(&retained),
        original.front().unwrap()
    ));
    assert_eq!(admitted.len(), 1);
    assert_eq!(original.len(), 1);
    let survivor_charge = ordered_index_charge::<u64, Arc<ReplayEvent>>(1)
        .unwrap()
        .checked_add(arc_allocation_charge::<ReplayEvent>().unwrap())
        .unwrap()
        .checked_add(surviving_heap)
        .unwrap();
    assert_eq!(
        admitted.prepared_retained_charge().unwrap(),
        survivor_charge
    );
}

#[test]
fn replay_message_capacity_is_additional_to_complete_merge_witness_payloads() {
    let mut event = emitted_merge_replay_event();
    let original = event.clone();
    let before = event.retained_heap_charge(&mut Work::new(10000)).unwrap();
    let Some(ReplayEventDetail::BranchMergeSummary { message, .. }) = &mut event.detail else {
        panic!("merge fixture")
    };
    let old_capacity = message.capacity();
    message.reserve_exact(8192);
    let delta = message.capacity() - old_capacity;
    assert_eq!(
        event
            .retained_heap_charge(&mut Work::new(10000))
            .unwrap()
            .bytes()
            - before.bytes(),
        delta as u64
    );
    assert_eq!(event, original);
    let mut message = String::with_capacity(2048);
    message.push_str("retained failure detail");
    let expected = message.capacity();
    assert_eq!(
        ReplayEventDetail::Message(message)
            .retained_heap_charge(&mut Work::new(2))
            .unwrap()
            .bytes(),
        expected as u64
    );
    assert_eq!(
        ReplayEventDetail::TaskOutcome(crate::logic::planner::TaskExecutionOutcome::Recomputed)
            .retained_heap_charge(&mut Work::new(1))
            .unwrap(),
        Charge::ZERO
    );
}
fn appended_charge(old: Charge, candidate: &ReplayEvent) -> Charge {
    old.checked_sub(ordered_index_charge::<u64, Arc<ReplayEvent>>(1).unwrap())
        .unwrap()
        .checked_add(ordered_index_charge::<u64, Arc<ReplayEvent>>(2).unwrap())
        .unwrap()
        .checked_add(arc_allocation_charge::<ReplayEvent>().unwrap())
        .unwrap()
        .checked_add(
            candidate
                .retained_heap_charge(&mut Work::new(10000))
                .unwrap(),
        )
        .unwrap()
}
