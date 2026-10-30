use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn every_case_runs_once_and_results_keep_input_order() {
    let visits = (0..17).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();
    let result = super::run((0..17).collect(), |index| {
        visits[index].fetch_add(1, Ordering::SeqCst);
        index * 3
    });
    assert_eq!(result, (0..17).map(|index| index * 3).collect::<Vec<_>>());
    assert!(visits
        .iter()
        .all(|visits| visits.load(Ordering::SeqCst) == 1));
}

#[test]
fn equal_inputs_are_independent_cases_and_are_never_deduplicated() {
    let visits = AtomicUsize::new(0);
    let result = super::run(vec![7, 7, 7], |value| {
        visits.fetch_add(1, Ordering::SeqCst);
        value
    });
    assert_eq!(result, vec![7, 7, 7]);
    assert_eq!(visits.load(Ordering::SeqCst), 3);
}

#[test]
fn a_failed_case_propagates_after_all_other_cases_join() {
    let visits = AtomicUsize::new(0);
    let result = std::panic::catch_unwind(|| {
        super::run((0..17).collect(), |index| {
            visits.fetch_add(1, Ordering::SeqCst);
            assert_ne!(index, 3, "case three is the original failure");
            index
        });
    });
    let failure = result.unwrap_err();
    let message = failure.downcast_ref::<String>().unwrap();
    assert!(message.contains("case three is the original failure"));
    assert_eq!(visits.load(Ordering::SeqCst), 17);
}
