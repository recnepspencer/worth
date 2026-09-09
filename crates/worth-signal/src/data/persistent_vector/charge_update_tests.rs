use super::{PersistentVector, RetainedVectorMutationDenial, RetainedVectorMutationOutcome};
use crate::data::retained_storage::{
    RetainedStorageMeasurement, RetainedStoragePreparation as Work,
    RetainedStoragePreparationDenial,
};

fn accounted<R>(outcome: RetainedVectorMutationOutcome<R>) -> R {
    match outcome {
        RetainedVectorMutationOutcome::Accounted { output, .. } => output,
        RetainedVectorMutationOutcome::Unaccounted { denial, .. } => {
            panic!("unexpected unaccounted edit: {denial:?}")
        }
    }
}

fn assert_carried_matches_representation<T: Clone + RetainedStorageMeasurement>(
    values: &PersistentVector<T>,
) {
    let carried = values.prepared_retained_charge().unwrap();
    // Explicit representation traversal is independent of the local delta
    // update; it includes hidden bases, all overlays and nested payloads.
    let measured = values
        .retained_heap_charge(&mut Work::new(100_000))
        .unwrap();
    assert_eq!(carried, measured);
}

#[test]
fn charge_follows_capacity_fork_payload_edit_append_and_logical_removal() {
    let mut values: PersistentVector<String> =
        [String::with_capacity(4096), String::from("second")]
            .into_iter()
            .collect();
    values.prepare_retained_charge(&mut Work::new(100)).unwrap();
    values.reserve_exclusive(16);
    assert_carried_matches_representation(&values);
    accounted(
        values
            .edit_with_retained_charge(0, &mut Work::new(100), |value| {
                value.push_str("original");
            })
            .unwrap(),
    );
    assert_carried_matches_representation(&values);
    let mut child = values.fork_persistent();
    assert_carried_matches_representation(&values);
    assert_carried_matches_representation(&child);
    accounted(
        child
            .edit_with_retained_charge(0, &mut Work::new(100), |value| {
                *value = String::from("replacement");
            })
            .unwrap(),
    );
    assert_carried_matches_representation(&child);
    assert_eq!(values[0], "original");
    let retained = child.clone();
    accounted(
        child
            .edit_with_retained_charge(1, &mut Work::new(100), |value| value.reserve(8192))
            .unwrap(),
    );
    assert_carried_matches_representation(&retained);
    assert_carried_matches_representation(&child);
    accounted(
        child
            .push_with_retained_charge(String::with_capacity(16384), &mut Work::new(100))
            .unwrap(),
    );
    assert_carried_matches_representation(&child);
    while !child.is_empty() {
        accounted(child.pop_with_retained_charge(&mut Work::new(100)).unwrap());
        assert_carried_matches_representation(&child);
    }
    assert!(
        child.prepared_retained_charge().unwrap().bytes() >= 4096,
        "the emptied child still owns its original shared base"
    );
    child.clear();
    assert_eq!(child.prepared_retained_charge().unwrap().bytes(), 0);
}

#[test]
fn untracked_mutation_closes_accounting_instead_of_serving_a_stale_charge() {
    let mut values: PersistentVector<String> = [String::from("initial")].into_iter().collect();
    values.prepare_retained_charge(&mut Work::new(10)).unwrap();
    values[0].reserve(8192);
    assert_eq!(
        values.prepared_retained_charge(),
        Err(RetainedVectorMutationDenial::PreparationRequired)
    );
    let mut contacted = false;
    assert!(matches!(
        values.edit_with_retained_charge(0, &mut Work::new(10), |_| contacted = true),
        Err(RetainedVectorMutationDenial::PreparationRequired)
    ));
    assert!(!contacted);
    assert!(values.prepare_retained_charge(&mut Work::new(0)).is_err());
    assert_eq!(
        values.prepared_retained_charge(),
        Err(RetainedVectorMutationDenial::PreparationRequired)
    );
    values.prepare_retained_charge(&mut Work::new(10)).unwrap();
    assert_carried_matches_representation(&values);
    values.iter_mut().next().unwrap().push_str(" changed");
    assert!(values.prepared_retained_charge().is_err());
}

#[test]
fn pre_edit_denial_and_post_edit_accounting_failure_preserve_distinct_progress() {
    let mut values: PersistentVector<String> = [String::from("initial")].into_iter().collect();
    values.prepare_retained_charge(&mut Work::new(10)).unwrap();
    let before = values.prepared_retained_charge().unwrap();
    let denied = values.edit_with_retained_charge(0, &mut Work::new(0), |value| value.clear());
    assert!(matches!(
        denied,
        Err(RetainedVectorMutationDenial::Accounting(
            RetainedStoragePreparationDenial::WorkExhausted { .. }
        ))
    ));
    assert_eq!(values[0], "initial");
    assert_eq!(values.prepared_retained_charge().unwrap(), before);
    let completion = values
        .edit_with_retained_charge(0, &mut Work::new(2), |value| {
            value.push_str(" performed");
            Box::new("retained operation output")
        })
        .unwrap();
    match completion {
        RetainedVectorMutationOutcome::Unaccounted { output, denial } => {
            assert_eq!(*output, "retained operation output");
            assert!(matches!(
                denial,
                RetainedStoragePreparationDenial::WorkExhausted { .. }
            ));
        }
        _ => panic!("post-edit work exhaustion must retain the actual output"),
    }
    assert_eq!(values[0], "initial performed");
    assert!(values.prepared_retained_charge().is_err());
    values.prepare_retained_charge(&mut Work::new(10)).unwrap();
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = values.edit_with_retained_charge(0, &mut Work::new(10), |value| {
            value.reserve(32768);
            panic!("payload edit unwinds");
        });
    }));
    assert!(unwind.is_err());
    assert!(values.prepared_retained_charge().is_err());
}

#[test]
fn changed_payload_accounting_and_ready_reads_do_not_scan_untouched_population() {
    let mut visits = Vec::new();
    for population in [1, 8, 64] {
        let mut values: PersistentVector<Vec<String>> = (0..population)
            .map(|_| vec![String::from("fixed payload")])
            .collect();
        values
            .prepare_retained_charge(&mut Work::new(10_000))
            .unwrap();
        let mut child = values.fork_persistent();
        let mut work = Work::new(100);
        accounted(
            child
                .edit_with_retained_charge(0, &mut work, |value| value[0].push('!'))
                .unwrap(),
        );
        visits.push(work.visits());
        let mut no_traversal = Work::new(0);
        let ready = child.prepared_retained_charge().unwrap();
        for _ in 0..64 {
            assert_eq!(
                child.prepare_retained_charge(&mut no_traversal).unwrap(),
                ready
            );
        }
        assert_eq!(no_traversal.visits(), 0);
        assert_carried_matches_representation(&child);
        assert_carried_matches_representation(&values);
    }
    assert!(visits[0] > 0);
    assert!(visits.iter().all(|&count| count == visits[0]));
}
