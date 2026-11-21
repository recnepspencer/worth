//! C1.3 on the filesystem: a damaged selected Native body refuses before any
//! World service. Store's physical recovery admits no damaged WAL at all, so
//! readmitting past a lost unrelated body is proved at the Native owner
//! (recorded_component_checkpoint_readmits_the_selected_subject_without_an_unrelated_body).
use super::*;
use crate::open_refusals::{flip_after, ARTIFACT_MAGIC};

#[test]
fn a_damaged_selected_native_body_refuses_before_any_world_service() {
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
    // The killed original's Native effect is the Home's first canonical body.
    let selected = 1_u64.to_le_bytes();
    assert!(
        flip_after(&location, ARTIFACT_MAGIC, 512, &|frame| {
            frame.get(10..18) == Some(&selected[..])
        }) > 0,
        "the selected body is on disk"
    );
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let refused = worth_store_testkit_application_oracle::world_home::open(
        worth_store_testkit_application_oracle::home_opening::request(
            &location,
            crate::workload::runtime_builder(crate::schema_transition::SOURCE),
        ),
        &execution,
    );
    use worth_store_application::{
        ApplicationHomeOpenRefusal as Open, ApplicationHomePhysicalRefusalKind as Kind,
        ApplicationHomePhysicalStage as Stage,
    };
    use worth_store_testkit_application_oracle::world_home::WorldHomeFixtureRefusal as Refused;
    let Err(Refused::Home(cause)) = refused else {
        panic!("a damaged selected body cannot reach service");
    };
    let Open::Physical(refusal) = *cause else {
        panic!("Store's physical recovery refuses first: {cause:?}");
    };
    assert_eq!(refusal.stage(), Stage::Recovery);
    assert_eq!(refusal.kind(), Kind::PhysicalStep);
    assert!(refusal.detail().contains("Damage("), "{}", refusal.detail());
}
