//! Actual Home/Native/World custody survives process death and explicit release.
use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::*;
use worth_store_application::{ApplicationHomeCheckpointDenial, ApplicationHomeCloseOutcome};
mod child;
mod published_predecessor;
const LOCATION: &str = "WORTH_WORLD_RELEASE_HOME";
const MARKERS: &str = "WORTH_WORLD_RELEASE_MARKERS";
const PREFIX: &str = "WORTH_WORLD_RELEASE_PUBLISHED_PREFIX";
const CHILD: &str =
    "process::application::world_durable_release::child::original_world_attempt_child";

fn maximum() -> std::num::NonZeroUsize {
    std::num::NonZeroUsize::new(16).unwrap()
}
fn open(
    location: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
) -> worth_store_testkit_application_oracle::world_home::WorldHomeFixture {
    worth_store_testkit_application_oracle::world_home::open(
        worth_store_testkit_application_oracle::home_opening::request(
            location,
            crate::workload::runtime_builder(crate::schema_transition::SOURCE),
        ),
        execution,
    )
    .expect("actual filesystem Home owner readmission")
}

#[test]
fn killed_native_effect_reopens_original_world_custody_releases_and_reopens_again() {
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let markers = directory.path().join("markers");
    std::fs::create_dir(&markers).unwrap();
    assert_eq!(
        crate::killed_child::run_and_kill(
            CHILD,
            &[(LOCATION, location.as_path()), (MARKERS, markers.as_path())],
            &markers.join("ready"),
            &markers.join("finished")
        ),
        crate::killed_child::ChildEnd::KilledAtPark
    );
    let original = std::fs::read_to_string(markers.join("original")).unwrap();
    let fields: Vec<_> = original.lines().collect();
    let number = |index: usize| fields[index].parse::<u64>().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open(&location, &execution);
    assert_eq!(format!("{:?}", fixture.home.home_identity()), fields[6]);
    assert_ne!(format!("{:?}", fixture.home.incarnation()), fields[7]);
    let inspection = fixture.home.world().inspection_port();
    let page = inspection.recovery_page(None, maximum()).unwrap();
    assert_eq!(page.rows().len(), 1);
    let row = &page.rows()[0];
    assert_eq!(row.state(), RuntimeWorldRecoveryRecordState::Recovered);
    assert!(row.admitted_at().is_none());
    assert_eq!(row.handle().identity().ordinal(), number(2));
    let recovery = fixture.home.world().recovery_port();
    let claim = recovery.inspect_recovered_effects(row.handle()).unwrap();
    assert_eq!(claim.attempt_identity().ordinal(), number(0));
    assert_eq!(claim.reserved_commit_identity().ordinal(), number(1));
    assert_eq!(claim.identity().ordinal(), number(2));
    assert_eq!(claim.claim_incarnation(), fixture.home.incarnation());
    let expected = claim.expected_head();
    assert_eq!(expected.selected_commit().ordinal(), number(3));
    assert_eq!(expected.lifecycle_incarnation().ordinal(), number(4));
    assert_eq!(expected.branch_identity().name().as_str(), fields[5]);
    assert_eq!(expected.reference_generation().get(), 0);
    let description = ProductBranchReferenceDescription::from_observation(expected);
    let receipt = claim
        .native_receipt()
        .expect("the killed actual Native effect is retained");
    let old_handle = row.handle().clone();
    // Teardown must permit the very reopen needed to inspect terminal journal
    // uncertainty. The actual owner drops its pins; durable custody stays open.
    drop((claim, page));
    close(fixture.home, &execution);
    let fixture = open(&location, &execution);
    let inspection = fixture.home.world().inspection_port();
    let page = inspection.recovery_page(None, maximum()).unwrap();
    assert_eq!(page.rows().len(), 1);
    let recovery = fixture.home.world().recovery_port();
    assert!(matches!(
        recovery.inspect_recovered_effects(&old_handle),
        Err(RuntimeWorldRecoveryDenial::StaleIncarnation)
    ));
    let claim = recovery
        .inspect_recovered_effects(page.rows()[0].handle())
        .unwrap();
    assert_eq!(claim.identity().ordinal(), number(2));
    assert_eq!(claim.attempt_identity().ordinal(), number(0));
    assert_eq!(claim.reserved_commit_identity().ordinal(), number(1));
    assert_eq!(claim.native_receipt().as_ref(), Some(&receipt));
    let mut outcome = recovery.release_recovered_effects(&execution, claim);
    let backstop = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let released = loop {
        match outcome {
            WorldRecoveredReleaseOutcome::Released(released) => break released,
            WorldRecoveredReleaseOutcome::Pending(claim) => {
                assert!(std::time::Instant::now() < backstop);
                std::thread::sleep(std::time::Duration::from_millis(1));
                outcome = recovery.release_recovered_effects(&execution, claim);
            }
            other => panic!("original recovered release refused: {other:?}"),
        }
    };
    assert_eq!(released.identity().ordinal(), number(2));
    assert_eq!(
        released.native_effect().unwrap().subject().receipt(),
        &receipt
    );
    assert_eq!(
        released
            .native_effect()
            .unwrap()
            .semantic_result()
            .created_entities()
            .len(),
        1
    );
    assert!(inspection
        .recovery_page(None, maximum())
        .unwrap()
        .rows()
        .is_empty());
    let unchanged = fixture
        .home
        .world()
        .observation_port()
        .readmit_product_branch_reference(&description)
        .unwrap();
    assert_eq!(unchanged.reference_generation().get(), 0);
    assert_eq!(unchanged.selected_commit().ordinal(), number(3));
    drop((unchanged, released, page));
    close(fixture.home, &execution);
    let second = open(&location, &execution);
    assert_eq!(format!("{:?}", second.home.home_identity()), fields[6]);
    assert!(second
        .home
        .world()
        .inspection_port()
        .recovery_page(None, maximum())
        .unwrap()
        .rows()
        .is_empty());
    let head = second
        .home
        .world()
        .observation_port()
        .readmit_product_branch_reference(&description)
        .unwrap();
    assert_eq!(head.reference_generation().get(), 0);
    assert_eq!(head.selected_commit().ordinal(), number(3));
    assert_eq!(
        std::fs::read_to_string(markers.join("handler-runs")).unwrap(),
        "1"
    );
    drop(head);
    close(second.home, &execution);
}

fn close(
    home: worth_store_application::WorldReadmittedApplicationHome<(), (), (), (), ()>,
    execution: &ExecutionRequest<'_, '_>,
) {
    match home.close(execution) {
        Ok(ApplicationHomeCloseOutcome::Closed {
            checkpoint: Err(ApplicationHomeCheckpointDenial::WorldSectionRequired),
            ..
        }) => {}
        Err(refused) => panic!("World close refused: {:?}", refused.reason()),
        Ok(ApplicationHomeCloseOutcome::CloseRefused { reason, .. }) => {
            panic!("Native close refused: {reason:?}")
        }
        _ => panic!("full World checkpoint remains explicitly unsupported"),
    }
}
