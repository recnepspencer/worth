//! One counted oracle for actual request-backed Query doors.
use std::fmt::Debug;
use worth_execution::{
    with_request_consult_observer, CancellationSource, ExecutionRequest, ExecutionWorkCeiling,
    SerialMemoryBudget, SerialRequest,
};
pub(crate) fn every_consult<T: Eq + Debug, E: Eq + Debug>(
    door: impl Fn(&ExecutionRequest<'_, '_>) -> Result<T, E>,
    cancelled: E,
    exhausted: E,
) {
    let run = |k: Option<usize>, ceiling: u64| {
        let source = CancellationSource::new();
        let cancel = source.clone();
        let host = SerialRequest::from_memory(
            SerialMemoryBudget::new(64 * 1024 * 1024),
            source.token(),
            None,
        );
        let request = ExecutionRequest::serial(&host);
        let ((answer, report), count) = with_request_consult_observer(
            move |at| {
                if Some(at) == k {
                    cancel.cancel();
                }
            },
            || {
                request
                    .run(ExecutionWorkCeiling::new(ceiling), |_| door(&request))
                    .unwrap()
            },
        );
        (answer, report.charged_work(), count)
    };
    let (healthy, work, count) = run(None, 100_000);
    assert_ne!(
        healthy.as_ref().err(),
        Some(&cancelled),
        "healthy door must not report cancellation"
    );
    assert_ne!(
        healthy.as_ref().err(),
        Some(&exhausted),
        "healthy door must not exhaust work"
    );
    assert!(count > 0);
    eprintln!("door consults={count}, work={work}");
    for k in 1..=count + 1 {
        let (answer, _, _) = run(Some(k), 100_000);
        if k > count {
            assert_eq!(answer, healthy, "past consult {count}");
        } else {
            assert_eq!(answer.as_ref().err(), Some(&cancelled), "consult {k}");
        }
    }
    if work > 0 {
        assert_eq!(
            run(None, work - 1).0.err(),
            Some(exhausted),
            "one unit short"
        );
    }
}
