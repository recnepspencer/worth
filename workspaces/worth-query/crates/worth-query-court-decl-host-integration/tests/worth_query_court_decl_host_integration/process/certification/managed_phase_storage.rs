use std::cell::Cell;
use worth_execution::ExecutionRequest;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitOutcome,
    WorthQueryManagedPhaseStorageForTest as PhaseStorage,
};
#[test]
fn managed_phase_storage_refuses_the_original_grant_before_start() {
    let budget = SerialMemoryBudget::new(Some(0));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let Err(WorthQueryApplicationCommitOutcome::Denied(denial)) =
        PhaseStorage::<u64>::prepare(&ExecutionRequest::serial(&host))
    else {
        panic!("the original zero grant must refuse before start");
    };
    assert!(matches!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ExecutionResource { .. }
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
}
#[test]
fn managed_phase_storage_retains_the_actual_value_and_releases_its_grant() {
    let bytes = std::mem::size_of::<Cell<Option<u64>>>() as u64;
    let budget = SerialMemoryBudget::new(Some(65_536));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let actual = PhaseStorage::prepare(&ExecutionRequest::serial(&host))
        .unwrap()
        .fill(37_u64);
    assert_eq!(budget.charged_memory_bytes(), bytes);
    assert_eq!(actual.consume(), 37);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
#[test]
fn cancelled_completion_preflight_is_cancelled_without_indeterminate_business() {
    let budget = SerialMemoryBudget::new(Some(65_536));
    let cancellation = CancellationSource::new();
    cancellation.cancel();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    assert!(matches!(
        PhaseStorage::<u64>::prepare(&ExecutionRequest::serial(&host)),
        Err(WorthQueryApplicationCommitOutcome::Cancelled)
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
}
