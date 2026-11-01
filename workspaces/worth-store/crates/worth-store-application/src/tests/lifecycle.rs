use crate::facade::{
    admit_home, readmit_relational, recover_home, ApplicationHomeCloseOutcome,
    ApplicationHomeClosePosture, ReadmittedApplicationHome,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

fn open(path: &std::path::Path, execution: &ExecutionRequest<'_, '_>) -> ReadmittedApplicationHome {
    let recovered = recover_home(super::program::request(path), execution).unwrap();
    let admitted = admit_home(recovered, execution).unwrap();
    readmit_relational(admitted, execution)
        .unwrap()
        .consume(|home| home)
}

#[test]
fn public_native_progression_closes_reopens_and_continues_without_a_partial_home_checkpoint() {
    let directory = tempfile::tempdir().unwrap();
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let home = open(directory.path(), &execution);
    let identity = home.home_identity();
    let incarnation = home.incarnation();
    super::native_schema::commit(home.relational().runtime(), &execution, "before");
    let before = super::native_schema::artifacts(&execution, home.relational().runtime());
    assert_eq!(before.len(), 1);
    assert!(matches!(
        home.checkpoint(&execution),
        Err(crate::ApplicationHomeCheckpointDenial::WorldSectionRequired)
    ));
    match home.close(&execution) {
        ApplicationHomeCloseOutcome::Closed { checkpoint, .. } => {
            assert!(matches!(
                checkpoint,
                Err(crate::ApplicationHomeCheckpointDenial::WorldSectionRequired)
            ));
        }
        ApplicationHomeCloseOutcome::CloseRefused { .. } => panic!("quiescent owner must close"),
    }
    let home = open(directory.path(), &execution);
    assert_eq!(home.home_identity(), identity);
    assert_eq!(home.incarnation(), incarnation.successor().unwrap());
    assert_eq!(
        super::native_schema::artifacts(&execution, home.relational().runtime()),
        before
    );
    super::native_schema::commit(home.relational().runtime(), &execution, "after");
    let after = super::native_schema::artifacts(&execution, home.relational().runtime());
    assert_eq!(after.len(), 2);
    assert_eq!(&after[..1], before.as_slice());
    assert!(matches!(
        home.close(&execution),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
}

#[test]
fn cancelled_close_returns_the_same_usable_serving_home_and_does_not_seal_admission() {
    let directory = tempfile::tempdir().unwrap();
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let home = open(directory.path(), &execution);
    let identity = home.home_identity();
    let cancellation = CancellationSource::new();
    let closing =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    cancellation.cancel();
    let home = match home.close(&ExecutionRequest::serial(&closing)) {
        ApplicationHomeCloseOutcome::CloseRefused {
            posture: ApplicationHomeClosePosture::Serving(home),
            ..
        } => home,
        ApplicationHomeCloseOutcome::Closed { .. }
        | ApplicationHomeCloseOutcome::CloseRefused {
            posture: ApplicationHomeClosePosture::RecoveryRequired(_),
            ..
        } => panic!("a cancelled close must preserve serving custody"),
    };
    assert_eq!(home.home_identity(), identity);
    super::native_schema::commit(home.relational().runtime(), &execution, "after-refusal");
    assert_eq!(
        super::native_schema::artifacts(&execution, home.relational().runtime()).len(),
        1
    );
    assert!(matches!(
        home.close(&execution),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
}

#[test]
fn retained_native_read_pin_refuses_close_and_the_same_home_can_release_it_and_continue() {
    let directory = tempfile::tempdir().unwrap();
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let home = open(directory.path(), &execution);
    let runtime = home.relational().runtime();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let snapshot = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    drop(basis);
    let home = match home.close(&execution) {
        ApplicationHomeCloseOutcome::CloseRefused {
            posture: ApplicationHomeClosePosture::Serving(home),
            ..
        } => home,
        ApplicationHomeCloseOutcome::Closed { .. }
        | ApplicationHomeCloseOutcome::CloseRefused {
            posture: ApplicationHomeClosePosture::RecoveryRequired(_),
            ..
        } => panic!("live read custody cannot be silently discarded"),
    };
    home.relational()
        .runtime()
        .snapshots()
        .release_snapshot(&snapshot)
        .unwrap();
    super::native_schema::commit(home.relational().runtime(), &execution, "after-pin-release");
    assert!(matches!(
        home.close(&execution),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
}
