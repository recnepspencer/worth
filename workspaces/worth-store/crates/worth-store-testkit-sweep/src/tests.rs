use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

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

#[test]
fn empty_input_never_calls_the_action_at_any_width() {
    for workers in [1, 4] {
        let result: Vec<()> = super::run_with_worker_limit(
            Vec::<()>::new(),
            |_| panic!("an empty sweep has no case"),
            Some(workers),
        );
        assert!(result.is_empty());
    }
}

#[test]
fn one_worker_runs_every_duplicate_on_the_calling_thread_in_order() {
    let caller = std::thread::current().id();
    let visits = Mutex::new(Vec::new());
    let inputs = vec![7, 7, 2, 7];
    let result = super::run_with_worker_limit(
        inputs.clone(),
        |value| {
            assert_eq!(std::thread::current().id(), caller);
            visits.lock().unwrap().push(value);
            value * 3
        },
        Some(1),
    );
    assert_eq!(visits.into_inner().unwrap(), inputs);
    assert_eq!(result, vec![21, 21, 6, 21]);
}

#[test]
fn serial_and_parallel_sweeps_drain_multiple_panics_then_resume_the_first_input_failure() {
    for workers in [1, 4] {
        let visits = (0..17).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();
        let result = std::panic::catch_unwind(|| {
            super::run_with_worker_limit(
                (0..17).collect(),
                |index| {
                    visits[index].fetch_add(1, Ordering::SeqCst);
                    if index == 2 || index == 7 {
                        std::panic::panic_any(index);
                    }
                    index
                },
                Some(workers),
            );
        });
        assert_eq!(*result.unwrap_err().downcast::<usize>().unwrap(), 2);
        assert!(visits
            .iter()
            .all(|visits| visits.load(Ordering::SeqCst) == 1));
    }
}

#[test]
fn earliest_input_panic_wins_even_when_a_later_input_panics_first() {
    let (send, receive) = std::sync::mpsc::channel();
    let receive = Mutex::new(receive);
    let visits = (0..5).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();
    let result = std::panic::catch_unwind(|| {
        super::run_with_worker_limit(
            (0..5).collect(),
            |index| {
                visits[index].fetch_add(1, Ordering::SeqCst);
                match index {
                    0 => {
                        receive
                            .lock()
                            .unwrap()
                            .recv_timeout(std::time::Duration::from_secs(10))
                            .expect("the worker must drain the later panic and reach case two");
                        std::panic::panic_any(index);
                    }
                    1 => std::panic::panic_any(index),
                    // With two workers, reaching case two means case one's
                    // panic was caught while the other worker still held zero.
                    2 => send.send(()).unwrap(),
                    3 | 4 => (),
                    _ => unreachable!("only five input indices were scheduled"),
                }
            },
            Some(2),
        );
    });
    assert_eq!(*result.unwrap_err().downcast::<usize>().unwrap(), 0);
    assert!(visits
        .iter()
        .all(|visits| visits.load(Ordering::SeqCst) == 1));
}
