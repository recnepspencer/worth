use super::fixture::*;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::transactions::data::{CommitExecutionDenialKind, TransactionCommitError};
use worth_execution::{CancellationSource, CancellationToken, ExecutionRequest};

#[test]
fn native_validation_preserves_original_cancellation_and_deadline() {
    for cancelled in [true, false] {
        let fixture = DagFixture::new(8);
        let before = fixture.head();
        let cancellation = CancellationSource::new();
        let lease = request(
            1024 * 1024,
            4_000_000,
            cancellation.token(),
            (!cancelled).then(std::time::Instant::now),
        );
        if cancelled {
            cancellation.cancel();
        }
        let failure = fixture
            .validate(&ExecutionRequest::leased(&lease), fixture.chain_batch(2))
            .expect_err("the original stopped request must not issue a candidate");
        match failure {
            TransactionCommitError::Execution { denial, .. } => {
                assert!(
                    matches!(denial.kind, CommitExecutionDenialKind::Cause(cause)
                    if cause == if cancelled { Cause::Cancelled } else { Cause::DeadlineElapsed })
                );
            }
            other => panic!("expected original request interruption, got {other:?}"),
        }
        assert_eq!(fixture.head(), before);
        assert_eq!(lease.charged_memory_bytes(), 0);
        let funded = request(1024 * 1024, 4_000_000, CancellationToken::new(), None);
        let candidate = fixture
            .validate(&ExecutionRequest::leased(&funded), fixture.chain_batch(2))
            .expect("the uninterrupted counterpart validates the same DAG");
        drop(candidate);
        assert_eq!(fixture.head(), before);
        assert_eq!(funded.charged_memory_bytes(), 0);
    }
}
