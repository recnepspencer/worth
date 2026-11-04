use super::*;
use std::sync::{atomic::AtomicUsize, Arc, Mutex};
use worth_query_host::facade::WorthQueryApplicationHomeLocation;
use worth_store_application::{
    ApplicationHomeOpeningEdge as Edge, ApplicationHomeOpeningStage as Stage,
};

pub(super) fn build() -> worth_foundational::facade::ApplicationBuildIdentity {
    worth_foundational::facade::ApplicationBuildIdentity::parse(env!(
        "WORTH_IMPLEMENTATION_BUILD_ID"
    ))
    .unwrap()
}
pub(super) fn serial() -> worth_execution::SerialRequest {
    worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(128 * 1024 * 1024)),
        worth_execution::CancellationToken::new(),
        None,
    )
}
#[test]
fn actual_host_opening_observes_every_owner_stage_without_granting_a_phase() {
    let path = tempfile::tempdir().unwrap();
    let request = serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let captured = seen.clone();
    let control = host_interruption::opening::control(
        move |arrival| captured.lock().unwrap().push(arrival),
        None,
        None,
    );
    let physical = control.physical_control_for_test();
    let seam = worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::BeforeWalAppend;
    assert!(matches!(
        physical.pause_mutation_for_test(seam),
        Err(worth_store_application::ApplicationHomePhysicalControlDenial::Unavailable)
    ));
    let home = host_observation::opening::open(
        WorthQueryApplicationHomeLocation::Directory(path.path().to_path_buf()),
        build(),
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
        Some(control),
        Default::default(),
        &execution,
    );
    let observations = seen.lock().unwrap().clone();
    let gate = physical
        .pause_mutation_for_test(seam)
        .expect("scheduler binds only to the actual recovered Store");
    gate.release();
    assert_eq!(observations.len(), 12);
    for stage in Stage::ALL {
        let before = observations
            .iter()
            .position(|a| a.stage == stage && a.edge == Edge::Before)
            .unwrap();
        let after = observations
            .iter()
            .position(|a| a.stage == stage && a.edge == Edge::After)
            .unwrap();
        assert!(
            before < after,
            "{stage:?} observes its actual predecessor before successor"
        );
    }
    let closed = home
        .close(&execution)
        .unwrap_or_else(|refused| panic!("actual close: {refused:?}"));
    assert!(closed.is_closed());
    closed.reclaim();
    assert!(
        matches!(
            physical.pause_mutation_for_test(seam),
            Err(worth_store_application::ApplicationHomePhysicalControlDenial::Unavailable)
        ),
        "the scheduler must not keep serving custody alive"
    );
}
