//! The actual weak operation refuses its caller policy without changing truth.
use crate::durability::data::{
    RelationalCheckpointArtifactPolicy, RelationalNativeCheckpointCaptureDenial as Denial,
};
use crate::tests::support::*;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
#[test]
fn complete_original_request_denial_and_cancel_preserve_original_empty_source() {
    let runtime = runtime_with_test_schema();
    let identity = runtime.main_branch_identity();
    let original = runtime
        .history
        .branch_cell(identity.branch_id())
        .unwrap()
        .root()
        .unwrap();
    let service = runtime.complete_checkpoint_service();
    let budget = SerialMemoryBudget::new(Some(0));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let execution = ExecutionRequest::serial(&host);
    assert!(matches!(
        service.prepare(&execution, RelationalCheckpointArtifactPolicy::Embedded),
        Err(Denial::Request(_))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
    let cancellation = CancellationSource::new();
    cancellation.cancel();
    let stopped = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(16 * 1024 * 1024)),
        cancellation.token(),
        None,
    );
    assert!(matches!(
        service.prepare(
            &ExecutionRequest::serial(&stopped),
            RelationalCheckpointArtifactPolicy::Embedded
        ),
        Err(Denial::Request(_))
    ));
    let actual = runtime
        .history
        .branch_cell(identity.branch_id())
        .unwrap()
        .root()
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(&original, &actual));
    let retry = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(16 * 1024 * 1024)),
        CancellationSource::new().token(),
        None,
    );
    let result = service
        .prepare(
            &ExecutionRequest::serial(&retry),
            RelationalCheckpointArtifactPolicy::Embedded,
        )
        .unwrap();
    assert!(result.checkpoint().captured_sections().unwrap().total > 0);
}
