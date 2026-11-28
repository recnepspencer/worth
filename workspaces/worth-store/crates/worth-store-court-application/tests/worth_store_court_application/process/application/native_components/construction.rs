//! Construction refusal reaches the caller without retaining the physical home.
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::runtime::{
    RelationalRuntimeApi, RelationalRuntimeConstructionDenial,
};
use worth_store_testkit_application_oracle::facade::home_opening::HomeOpeningRefused as OpenRefusal;

use worth_store_application::{ApplicationHomeCloseOutcome, ApplicationHomeOpenRefusal};

#[test]
fn invalid_invariant_refuses_open_with_its_cause_and_releases_the_home() {
    use worth_relational::facade::runtime::{
        InvariantCatalog, InvariantExecutionPoint, InvariantRegistration, InvariantRule,
    };

    let mut registration =
        InvariantRegistration::commit_boundary_blocking(InvariantRule::MaxMergedIntents(1));
    registration.execution_point = InvariantExecutionPoint::ProposedState;
    let builder = RelationalRuntimeApi::builder().invariant_catalog(InvariantCatalog {
        registrations: vec![registration],
    });
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let initialized = worth_store_testkit_application_oracle::facade::home_opening::open(
        worth_store_testkit_application_oracle::facade::home_opening::request(
            &location,
            RelationalRuntimeApi::builder(),
        ),
        &execution,
    )
    .unwrap();
    assert!(matches!(
        initialized.close(&execution),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
    let refused = worth_store_testkit_application_oracle::facade::home_opening::open(
        worth_store_testkit_application_oracle::facade::home_opening::request(&location, builder),
        &execution,
    );
    let refused = match refused {
        Err(OpenRefusal::Readmission(
            worth_store_application::RelationalHomeOpenRefused::Attachment(
                worth_store_application::RelationalAttachmentRefused::Construction(home),
            ),
        )) => home,
        other => panic!(
            "invalid invariant refused at wrong phase: {:?}",
            other.map(|_| ())
        ),
    };
    assert!(matches!(
        refused.reason(),
        RelationalRuntimeConstructionDenial::InvariantExecutionPoint {
            execution_point: InvariantExecutionPoint::ProposedState,
            ..
        }
    ));
    let (_, closed) = refused.close(&execution);
    assert!(matches!(closed, ApplicationHomeCloseOutcome::Closed { .. }));

    // The actual construction refusal retained its Store custody until the
    // explicit close above. A production reopen independently checks release.
    let home = worth_store_testkit_application_oracle::facade::home_opening::open(
        worth_store_testkit_application_oracle::facade::home_opening::request(
            &location,
            RelationalRuntimeApi::builder(),
        ),
        &execution,
    )
    .expect("the refused location remains usable by a valid runtime");
    assert!(matches!(
        home.close(&execution),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
}

#[test]
fn cancelled_attachment_retains_admitted_home_until_explicit_never_attached_close() {
    use worth_execution::{MapKernelStop, WorkCeilingDenial};
    use worth_store_application::{
        admit_home, attach_relational, recover_home, ApplicationHomeCheckpointDenial,
        ApplicationHomeCloseDenial, ApplicationHomeClosePosture, RelationalAttachmentRefused,
    };
    use worth_store_testkit_application_oracle::facade::home_opening;

    let directory = tempfile::tempdir().unwrap();
    let cancellation = CancellationSource::new();
    let original =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let original = ExecutionRequest::serial(&original);
    let initialized = home_opening::open(
        home_opening::request(directory.path(), RelationalRuntimeApi::builder()),
        &original,
    )
    .unwrap();
    assert!(matches!(
        initialized.close(&original),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
    let admitted = original
        .in_scope(|_| {
            let recovered = recover_home(
                home_opening::request(directory.path(), RelationalRuntimeApi::builder()),
                &original,
            )?;
            admit_home(recovered, &original)
        })
        .unwrap()
        .unwrap();
    cancellation.cancel();
    let home = match attach_relational(admitted, &original) {
        Err(RelationalAttachmentRefused::BeforeConstruction {
            cause: WorkCeilingDenial::Stopped(MapKernelStop::Cancelled),
            home,
        }) => home,
        _ => panic!("original cancellation must retain the genuine pre-construction Home"),
    };
    let fresh = crate::workload::serial_request();
    let fresh = ExecutionRequest::serial(&fresh);
    match home_opening::open(
        home_opening::request(directory.path(), RelationalRuntimeApi::builder()),
        &fresh,
    ) {
        Err(OpenRefusal::BeforeReadmission(ApplicationHomeOpenRefusal::Physical(refused))) => {
            assert_eq!(
                refused.kind(),
                worth_store_application::ApplicationHomePhysicalRefusalKind::MutationOwnerContended,
            )
        }
        _ => panic!("BeforeConstruction keeps the actual Home mutation lease"),
    }
    // Choosing close consumes the original definition; cancellation still
    // returns inspectable actual physical custody, without creating Native.
    let recovery = match home.consume(|home| home.close(&original)) {
        ApplicationHomeCloseOutcome::CloseRefused {
            reason:
                ApplicationHomeCloseDenial::Execution(WorkCeilingDenial::Stopped(
                    MapKernelStop::Cancelled,
                )),
            posture: ApplicationHomeClosePosture::RecoveryRequired(recovery),
        } => recovery,
        _ => panic!("cancelled cleanup must retain the same physical Home"),
    };
    assert!(recovery.world_close_report().is_none());
    assert!(matches!(
        recovery.checkpoint_result(),
        Err(ApplicationHomeCheckpointDenial::NativeSectionNeverAttached)
    ));
    assert!(matches!(
        recovery.try_release(&fresh),
        ApplicationHomeCloseOutcome::Closed {
            checkpoint: Err(ApplicationHomeCheckpointDenial::NativeSectionNeverAttached),
            ..
        }
    ));
    let reopened = home_opening::open(
        home_opening::request(directory.path(), RelationalRuntimeApi::builder()),
        &fresh,
    )
    .expect("actual physical close permits a genuine reopen");
    assert!(matches!(
        reopened.close(&fresh),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
}
