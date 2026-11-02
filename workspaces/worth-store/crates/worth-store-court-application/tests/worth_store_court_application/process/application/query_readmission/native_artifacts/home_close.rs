//! D11 crosses the production Host, original Query and actual physical Home.
use super::*;
use worth_execution::{CancellationSource, MapKernelStop, SerialMemoryBudget, SerialRequest};
use worth_query_host::facade::WorthQueryApplicationHome;
use worth_runtime_world::facade::{RuntimeWorldCloseDenial, RuntimeWorldServiceDenial};
mod killed_writer;
mod source_retirement;
type HostHome = WorthQueryApplicationHome<HomeIdentitySchema, HomeProgram>;
fn open(
    path: &std::path::Path,
    execution: ExecutionRequest<'_, '_>,
    initialized: Arc<AtomicUsize>,
    contacts: Arc<AtomicUsize>,
) -> HostHome {
    let request = fixture::request(path.to_path_buf(), build(), initialized, contacts);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("actual Host admission: {refused:?}"));
    home
}
fn serial(bytes: Option<u64>, deadline: Option<std::time::Instant>) -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(bytes),
        CancellationSource::new().token(),
        deadline,
    )
}
fn physical_media_admission() -> worth_store::physical_runtime::FilesystemMediaAdmission {
    use worth_store::physical_runtime::{FilesystemAccessPosture, FilesystemMediaAdmission};
    FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount)
}
fn assert_physical_lease_released(path: &std::path::Path, _: &ExecutionRequest<'_, '_>) {
    // This owner port acquires the actual exclusive mutation lease, without
    // running the separately governed Native/World recovery journey.
    let held = physical_media_admission()
        .qualify_home(path)
        .unwrap_or_else(|cause| {
            panic!("physical lease must be released while retired owners remain alive: {cause:?}")
        });
    drop(held);
}
#[test]
fn fresh_read_host_physically_closes_with_original_retired_allocations_alive() {
    let path = tempfile::tempdir().unwrap();
    let live = serial(None, None);
    let execution = ExecutionRequest::serial(&live);
    let initialized = Arc::new(AtomicUsize::new(0));
    let contacts = Arc::new(AtomicUsize::new(0));
    let home = open(
        path.path(),
        execution,
        initialized.clone(),
        contacts.clone(),
    );
    assert!(
        matches!(
        physical_media_admission().qualify_home(path.path()),
        Err(worth_store::physical_runtime::RecoveryFilesystemQualificationError::OwnershipContended)
    ),
        "the independent probe must refuse while this same Home serves"
    );
    assert_eq!(balance(home.runtime()), 7);
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    let closed = home
        .close(&execution)
        .unwrap_or_else(|refused| panic!("same read Home must physically close: {refused:?}"));
    assert!(matches!(
        closed.outcome(),
        ApplicationHomeCloseOutcome::Closed {
            checkpoint: Ok(_),
            ..
        }
    ));
    assert_eq!(
        contacts.load(Ordering::SeqCst),
        0,
        "closing never invokes business"
    );
    assert_physical_lease_released(path.path(), &execution);
    assert!(closed.is_closed());
    closed.reclaim();
}
#[test]
fn expired_host_close_preserves_the_same_serving_home_and_a_live_retry_physically_closes() {
    let path = tempfile::tempdir().unwrap();
    let initialized = Arc::new(AtomicUsize::new(0));
    let contacts = Arc::new(AtomicUsize::new(0));
    let live = serial(None, None);
    let execution = ExecutionRequest::serial(&live);
    let home = open(
        path.path(),
        execution,
        initialized.clone(),
        contacts.clone(),
    );
    assert!(matches!(
        set(home.runtime(), 8),
        WorthQueryApplicationMutationOutcome::Committed { result: 8, .. }
    ));
    let expired = serial(
        None,
        Some(std::time::Instant::now() - std::time::Duration::from_millis(1)),
    );
    let refused = home
        .close(&ExecutionRequest::serial(&expired))
        .err()
        .expect("original deadline refuses before seal");
    assert!(
        matches!(
            refused.directory_denial(),
            Some(ApplicationHomeCloseDenial::World(
                RuntimeWorldServiceDenial::Denied(RuntimeWorldCloseDenial::Stopped(
                    MapKernelStop::DeadlineElapsed
                ))
            ))
        ),
        "{refused:?}"
    );
    let home = refused
        .resume()
        .unwrap_or_else(|refused| panic!("pre-seal refusal retains a serving Home: {refused:?}"));
    assert_eq!(balance(home.runtime()), 8);
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    let closed = home
        .close(&execution)
        .unwrap_or_else(|refused| panic!("same Home closes after live retry: {refused:?}"));
    assert!(matches!(
        closed.outcome(),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
    assert_physical_lease_released(path.path(), &execution);
    // Physical lease release is observed while ALL retired allocations remain alive.
    assert!(closed.is_closed());
    closed.reclaim();
}
#[test]
fn host_checkpoint_memory_failure_is_reported_while_original_retired_owners_remain_physically_closed(
) {
    let path = tempfile::tempdir().unwrap();
    let live = serial(None, None);
    let execution = ExecutionRequest::serial(&live);
    let home = open(
        path.path(),
        execution,
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
    );
    // A real acknowledged journal tail makes capture larger than the fixed
    // retirement cursor. No checkpoint or handler result is fabricated.
    for ordinal in 0..32 {
        assert!(matches!(
            operations::set_key(home.runtime(), 0xd110 + ordinal, 8),
            WorthQueryApplicationMutationOutcome::Committed { result: 8, .. }
        ));
    }
    assert_eq!(balance(home.runtime()), 8);
    let bounded = serial(Some(256 * 1024), None);
    let closed = home
        .close(&ExecutionRequest::serial(&bounded))
        .unwrap_or_else(|refused| {
            panic!("a failed checkpoint is reported without withholding close: {refused:?}")
        });
    match closed.outcome() {
        ApplicationHomeCloseOutcome::Closed {
            checkpoint: Err(cause),
            ..
        } => {
            assert_checkpoint_memory_failure(cause);
        }
        ApplicationHomeCloseOutcome::Closed {
            checkpoint: Ok(_), ..
        } => panic!("this declared memory ceiling must exercise the real capture failure"),
        ApplicationHomeCloseOutcome::CloseRefused { reason, .. } => {
            panic!("checkpoint failure cannot veto physical Closed: {reason:?}")
        }
    }
    assert_physical_lease_released(path.path(), &execution);
    closed.reclaim();
}

fn assert_checkpoint_memory_failure(cause: &ApplicationHomeCheckpointDenial) {
    use worth_execution::{
        ExecutionAllocationDenialClass as Allocation, ExecutionRequestArrayDenial as Array,
        LeaseDenial, WorkCeilingDenial,
    };
    use worth_relational::facade::{
        durability::RelationalNativeCheckpointCaptureDenial as Capture,
        transactions::RelationalExecutionDenialCause as Request,
    };
    use worth_store_relational::RelationalSectionCheckpointDenial as Section;
    use ApplicationHomeCheckpointDenial::{Execution, Native, Payload};
    let memory = |work: &WorkCeilingDenial| {
        matches!(
            work,
            WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(_))
        )
    };
    let matched = matches!(
        cause,
        Native(Section::Capture(Capture::Request(
            Request::PolicyMemoryExhausted { .. }
                | Request::ProcessMemoryExhausted { .. }
                | Request::DeclaredMemoryExhausted { .. }
        )))
    ) || matches!(cause,
        Execution(work) | Native(Section::Execution(work)) | Payload(Array::Request(work))
            if memory(work)
    ) || matches!(cause,
        Native(Section::Capture(Capture::Allocation(denial))) | Payload(Array::Allocation(denial))
            if matches!(denial.classify(), Allocation::Request(work) if memory(&work))
    );
    assert!(
        matched,
        "expected the actual memory ceiling refusal: {cause:?}"
    );
}
