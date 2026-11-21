//! A killed later attempt retains its actual published predecessor, not root.
use super::*;

#[test]
fn killed_native_after_two_publications_recovers_exact_unpublished_predecessor() {
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let markers = directory.path().join("markers");
    std::fs::create_dir(&markers).unwrap();
    assert_eq!(
        crate::killed_child::run_and_kill(
            CHILD,
            &[
                (LOCATION, location.as_path()),
                (MARKERS, markers.as_path()),
                (PREFIX, std::path::Path::new("two"))
            ],
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
    let inspection = fixture.home.world().inspection_port();
    assert_eq!(
        inspection.history_snapshot().unwrap().installed_commits(),
        3
    );
    assert_eq!(
        fixture
            .home
            .relational()
            .runtime()
            .history()
            .immutable_commit_count(),
        3
    );
    let page = inspection.recovery_page(None, maximum()).unwrap();
    assert_eq!(page.rows().len(), 1);
    let recovery = fixture.home.world().recovery_port();
    let claim = recovery
        .inspect_recovered_effects(page.rows()[0].handle())
        .unwrap();
    assert_eq!(claim.attempt_identity().ordinal(), number(0));
    assert_eq!(claim.reserved_commit_identity().ordinal(), number(1));
    assert_eq!(claim.identity().ordinal(), number(2));
    assert_eq!(claim.expected_head().selected_commit().ordinal(), number(3));
    assert_eq!(claim.expected_head().reference_generation().get(), 2);
    assert_eq!(
        claim.expected_head().branch_identity().name().as_str(),
        fields[5]
    );
    let current = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(current, *claim.expected_head());
    let description = ProductBranchReferenceDescription::from_observation(&current);
    let receipt = claim.native_receipt().unwrap();
    let mut outcome = recovery.release_recovered_effects(&execution, claim);
    let released = loop {
        match outcome {
            WorldRecoveredReleaseOutcome::Released(released) => break released,
            WorldRecoveredReleaseOutcome::Pending(claim) => {
                outcome = recovery.release_recovered_effects(&execution, claim)
            }
            other => panic!("original later unpublished release refused: {other:?}"),
        }
    };
    assert_eq!(released.original().native_receipt().unwrap(), &receipt);
    assert_eq!(
        released
            .original()
            .native_semantic_result()
            .unwrap()
            .created_entities()
            .len(),
        1
    );
    drop((released, current, page));
    close(fixture.home, &execution);
    let fixture = open(&location, &execution);
    assert!(fixture
        .home
        .world()
        .inspection_port()
        .recovery_page(None, maximum())
        .unwrap()
        .rows()
        .is_empty());
    assert_eq!(
        fixture
            .home
            .relational()
            .runtime()
            .history()
            .immutable_commit_count(),
        3
    );
    let current = fixture
        .home
        .world()
        .observation_port()
        .readmit_product_branch_reference(&description)
        .unwrap();
    assert_eq!(current.reference_generation().get(), 2);
    assert_eq!(current.selected_commit().ordinal(), number(3));
    assert_eq!(
        std::fs::read_to_string(markers.join("handler-runs")).unwrap(),
        "1"
    );
    drop(current);
    close(fixture.home, &execution);
}
