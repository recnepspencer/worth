//! Interruption after physical drain returns the one running close and lease.
use worth_execution::{
    CancellationSource, ExecutionRequest, MapKernelStop, SerialMemoryBudget, SerialRequest,
    WorkCeilingDenial,
};
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_store::physical_runtime::PhysicalStoreClosePhase;
use worth_store_application::{
    ApplicationHomeCloseDenial, ApplicationHomeCloseOutcome, ApplicationHomeClosePosture,
};
use worth_store_testkit_application_oracle::facade::home_opening;

#[test]
fn cancellation_after_drain_retains_the_running_close_and_resume_releases_its_lease() {
    interrupted_close(false);
}

#[test]
fn deadline_after_drain_retains_the_running_close_and_resume_releases_its_lease() {
    interrupted_close(true);
}

fn interrupted_close(deadline: bool) {
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
    let identity = home.home_identity();
    let cancellation = CancellationSource::new();
    let closing = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        cancellation.token(),
        deadline.then(|| {
            worth_execution::MonotonicClock::system()
                .deadline_at(std::time::Instant::now() + std::time::Duration::from_secs(5))
        }),
    );
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let (outcome, gate) = std::thread::scope(|scope| {
        let cancellation = &cancellation;
        let controller = scope.spawn(move || {
            let gate: worth_store::physical_runtime::certification::CertificationPhysicalClosePauseGate =
                receiver.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
            assert!(gate.await_arrival(), "the actual Store drain started before interruption");
            if !deadline { cancellation.cancel(); }
            gate
        });
        let outcome = home.close_with_physical_observer_for_test(
            &ExecutionRequest::serial(&closing),
            move |plan| {
                let gate = plan.certification_pause_at(PhysicalStoreClosePhase::SignalDisposed);
                sender.send(gate).unwrap();
            },
        );
        (outcome, controller.join().unwrap())
    });
    let custody = match outcome {
        ApplicationHomeCloseOutcome::CloseRefused {
            reason: ApplicationHomeCloseDenial::Execution(WorkCeilingDenial::Stopped(cause)),
            posture: ApplicationHomeClosePosture::RecoveryRequired(custody),
        } => {
            assert_eq!(
                cause,
                if deadline {
                    MapKernelStop::DeadlineElapsed
                } else {
                    MapKernelStop::Cancelled
                }
            );
            custody
        }
        ApplicationHomeCloseOutcome::Closed { .. }
        | ApplicationHomeCloseOutcome::CloseRefused { .. } => {
            panic!("interruption must return the running close, never a serving owner")
        }
    };
    assert_eq!(custody.home_identity(), identity);
    assert_eq!(custody.location(), directory.path());
    let progress = custody.physical_close_observation().unwrap();
    assert!(progress.reached(PhysicalStoreClosePhase::SignalDisposed));
    assert!(!progress.reached(PhysicalStoreClosePhase::MediaReleased));
    crate::incarnation::assert_contended(directory.path());
    gate.release();
    assert!(matches!(
        custody.try_release(&execution),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
    assert!(
        progress.reached(PhysicalStoreClosePhase::MediaReleased),
        "resume completed the observed original task"
    );
    let reopened = home_opening::open(
        home_opening::request(directory.path(), RelationalRuntimeApi::builder()),
        &execution,
    )
    .unwrap();
    assert_eq!(reopened.home_identity(), identity);
    assert!(matches!(
        reopened.close(&execution),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
}

mod checkpoint;
