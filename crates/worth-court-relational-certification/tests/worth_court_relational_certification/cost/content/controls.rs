use crate::content::{
    fixture::{format, grid},
    original,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{content::*, transactions::RelationalExecutionDenialCause};

#[test]
fn content_equality_shortcuts_obey_original_cancel_deadline_and_work() {
    let producer_cancel = CancellationSource::new();
    let producer = original(&producer_cancel);
    let execution = ExecutionRequest::serial(&producer);
    let format = format("sample-grid", 1, false);
    let frame = RelationalContentFrame::encode(&execution, &format, &grid()).unwrap();
    let mut changed = grid();
    changed.samples[0] = 5;
    let other = RelationalContentFrame::encode(&execution, &format, &changed).unwrap();
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(8192), Some(64)),
    );
    for (request, expected) in {
        let cancel = CancellationSource::new();
        let cancelled = original(&cancel);
        cancel.cancel();
        let limited = SerialRequest::from_policy(&policy, producer_cancel.token(), None);
        let expired = original(&producer_cancel).with_deadline(Some(std::time::Instant::now()));
        [
            (cancelled, RelationalExecutionDenialCause::Cancelled),
            (limited, RelationalExecutionDenialCause::WorkExhausted),
            (expired, RelationalExecutionDenialCause::DeadlineElapsed),
        ]
    } {
        for right in [&frame, &other] {
            assert_eq!(
                frame.same_content(&ExecutionRequest::serial(&request), right),
                Err(RelationalContentDenial::Execution(expected))
            );
        }
        assert_eq!(request.memory().unwrap().charged_memory_bytes(), 0);
    }
}
