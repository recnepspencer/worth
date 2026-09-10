use super::*;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial,
};

#[derive(Debug, PartialEq, Eq)]
struct UnclonableFrame(String);

#[test]
fn history_fork_append_and_eviction_share_frames_without_cloning_them() {
    let mut original = DiagnosticHistory::new();
    for index in 0..1024 {
        original
            .push_back(UnclonableFrame(index.to_string()))
            .unwrap();
    }
    let mut fork = original.clone();
    assert!(original.entries.ptr_eq(&fork.entries));
    let removed = fork.pop_front().unwrap();
    assert!(Arc::ptr_eq(&removed, original.entries.get(&0).unwrap()));
    fork.push_back(UnclonableFrame("new".into())).unwrap();
    assert_eq!(original.front().unwrap().0, "0");
    assert_eq!(fork.front().unwrap().0, "1");
    assert_eq!(fork.back().unwrap().0, "new");
    for position in [1, 511, 1023] {
        assert!(Arc::ptr_eq(
            original.entries.get(&position).unwrap(),
            fork.entries.get(&position).unwrap()
        ));
    }
}

#[test]
fn sequence_wire_and_iteration_preserve_order_after_middle_removal() {
    let mut history: DiagnosticHistory<_> = [10, 20, 30].into_iter().collect();
    assert_eq!(*history.remove(1).unwrap(), 20);
    let wire = serde_json::to_string(&history).unwrap();
    assert_eq!(wire, "[10,30]");
    let restored: DiagnosticHistory<i32> = serde_json::from_str(&wire).unwrap();
    assert_eq!(restored, history);
    assert_eq!(history.iter().rev().copied().collect::<Vec<_>>(), [30, 10]);
}

#[test]
fn exhausted_position_denies_before_changing_retained_roots() {
    let mut history: DiagnosticHistory<_> = [10].into_iter().collect();
    history.next_position = None;
    let original = history.clone();
    assert_eq!(
        history.push_back(20),
        Err(DiagnosticHistoryPositionExhausted)
    );
    assert!(history.entries.ptr_eq(&original.entries));
    assert_eq!(history, original);
}

#[test]
fn retained_append_admits_exact_bytes_and_denies_short_bytes_or_work_without_installation() {
    let original: DiagnosticHistory<_> = ["old".to_string()].into_iter().collect();
    original
        .prepare_retained_charge(&mut Work::new(100))
        .unwrap();
    let mut expected = original.clone();
    expected
        .push_back("new retained frame".to_string())
        .unwrap();
    let exact = expected.retained_heap_charge(&mut Work::new(100)).unwrap();
    let mut admitted = original.clone();
    let mut work = Work::new(100);
    admitted
        .append_with_retained_capacity("new retained frame".into(), exact, &mut work)
        .unwrap();
    assert_eq!(admitted, expected);
    assert_eq!(admitted.prepared_retained_charge().unwrap(), exact);
    let visits = work.visits();
    let mut exact_work = original.clone();
    exact_work
        .append_with_retained_capacity("new retained frame".into(), exact, &mut Work::new(visits))
        .unwrap();
    assert_eq!(exact_work, expected);
    for (maximum, limit) in [
        (
            exact
                .checked_sub(Charge::capacity::<u8>(1).unwrap())
                .unwrap(),
            100,
        ),
        (exact, visits - 1),
    ] {
        let mut rejected = original.clone();
        let (value, denial) = rejected
            .append_with_retained_capacity(
                "new retained frame".into(),
                maximum,
                &mut Work::new(limit),
            )
            .unwrap_err();
        assert_eq!(value, "new retained frame");
        let expected_denial = if limit == 100 {
            DiagnosticHistoryEditDenial::CapacityExhausted {
                maximum,
                required: exact,
            }
        } else {
            DiagnosticHistoryEditDenial::Accounting(
                RetainedStoragePreparationDenial::WorkExhausted {
                    maximum_visits: limit,
                },
            )
        };
        assert_eq!(denial, expected_denial);
        assert!(rejected.entries.ptr_eq(&original.entries));
        assert_eq!(
            rejected.prepared_retained_charge(),
            original.prepared_retained_charge()
        );
        assert_eq!(rejected.next_position, original.next_position);
    }
}

#[test]
fn retained_eviction_updates_only_this_root_and_raw_edits_require_readmission() {
    let mut history: DiagnosticHistory<_> = ["first".to_string(), "second".to_string()]
        .into_iter()
        .collect();
    history
        .prepare_retained_charge(&mut Work::new(100))
        .unwrap();
    let original = history.clone();
    let removed = history
        .evict_front_with_retained_charge(&mut Work::new(100))
        .unwrap()
        .unwrap();
    assert!(Arc::ptr_eq(&removed, original.entries.get(&0).unwrap()));
    assert_eq!(
        history.prepared_retained_charge().unwrap(),
        history.retained_heap_charge(&mut Work::new(100)).unwrap()
    );
    assert_eq!(original.len(), 2);
    assert_eq!(history.len(), 1);
    history.push_back("raw".into()).unwrap();
    assert_eq!(
        history.prepared_retained_charge(),
        Err(DiagnosticHistoryEditDenial::PreparationRequired)
    );
    assert!(matches!(
        history.append_with_retained_capacity("denied".into(), Charge::ZERO, &mut Work::new(0)),
        Err((_, DiagnosticHistoryEditDenial::PreparationRequired))
    ));
}
