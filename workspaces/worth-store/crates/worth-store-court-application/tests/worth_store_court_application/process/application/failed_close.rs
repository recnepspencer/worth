//! An accepted close failure returns Store's actual owning evidence.
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_store::physical_runtime::PhysicalStoreCloseFailureCause;
use worth_store_application::{
    ApplicationHomeCloseDenial, ApplicationHomeCloseOutcome, ApplicationHomeClosePosture,
};
use worth_store_testkit_application_oracle::facade::home_opening;

#[test]
fn physical_close_worker_failure_retains_actual_inspection_evidence() {
    let directory = tempfile::tempdir().unwrap();
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let home = home_opening::open(
        home_opening::request(directory.path(), RelationalRuntimeApi::builder()),
        &execution,
    )
    .unwrap();
    let home_identity = home.home_identity();
    let runtime_identity = home.physical_serving_for_test().runtime_identity();
    let outcome = home.close_with_physical_observer_for_test(&execution, |_| {
        panic!("fail the actual close worker before its plan executes");
    });
    let custody = match outcome {
        ApplicationHomeCloseOutcome::CloseRefused {
            reason:
                ApplicationHomeCloseDenial::PhysicalCloseFailed(
                    PhysicalStoreCloseFailureCause::WorkerPanicked,
                ),
            posture: ApplicationHomeClosePosture::RecoveryRequired(custody),
        } => custody,
        ApplicationHomeCloseOutcome::Closed { .. }
        | ApplicationHomeCloseOutcome::CloseRefused { .. } => {
            panic!("worker failure must return actual inspection custody")
        }
    };
    assert_eq!(custody.home_identity(), home_identity);
    let failure = custody.physical_close_failure().unwrap();
    assert_eq!(failure.runtime_identity(), runtime_identity);
    assert!(failure.actual_outcome().is_none());
    assert!(failure.observation().is_some());
    // Retrying failed evidence must neither start another task nor reconstruct
    // a serving owner. Its actual worker identity and failure survive.
    let custody = match custody.try_release(&execution) {
        ApplicationHomeCloseOutcome::CloseRefused {
            reason:
                ApplicationHomeCloseDenial::PhysicalCloseFailed(
                    PhysicalStoreCloseFailureCause::WorkerPanicked,
                ),
            posture: ApplicationHomeClosePosture::RecoveryRequired(custody),
        } => custody,
        ApplicationHomeCloseOutcome::Closed { .. }
        | ApplicationHomeCloseOutcome::CloseRefused { .. } => panic!("failed task is inspect-only"),
    };
    let failure = custody
        .into_physical_close_failure()
        .unwrap_or_else(|_| panic!("actual failure"));
    assert_eq!(failure.runtime_identity(), runtime_identity);
    let (cause, actual, observation) = failure.into_parts();
    assert_eq!(cause, PhysicalStoreCloseFailureCause::WorkerPanicked);
    assert!(actual.is_none());
    assert!(observation.is_some());
}
