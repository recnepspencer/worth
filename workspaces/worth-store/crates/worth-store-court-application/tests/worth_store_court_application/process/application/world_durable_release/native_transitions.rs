//! A real Native fork after released-source restore survives an uncheckpointed tail.
use super::*;
use worth_relational::facade::history::BranchId;

#[test]
fn a_native_fork_from_a_released_predecessor_reopens_before_another_checkpoint() {
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
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open(&location, &execution);
    let page = fixture
        .home
        .world()
        .inspection_port()
        .recovery_page(None, maximum())
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    let recovery = fixture.home.world().recovery_port();
    let claim = recovery
        .inspect_recovered_effects(page.rows()[0].handle())
        .unwrap();
    let expected_product = claim.expected_head().selected_commit().ordinal();
    let released_receipt = claim.native_receipt().unwrap();
    let mut outcome = recovery.release_recovered_effects(&execution, claim);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let released = loop {
        match outcome {
            WorldRecoveredReleaseOutcome::Released(released) => break released,
            WorldRecoveredReleaseOutcome::Pending(claim) => {
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(1));
                outcome = recovery.release_recovered_effects(&execution, claim);
            }
            other => panic!("actual original release: {other:?}"),
        }
    };
    assert_eq!(
        released.original().native_receipt(),
        Some(&released_receipt)
    );
    drop((released, page));
    close(fixture.home, &execution);

    let fixture = open(&location, &execution);
    let native = fixture.home.relational().runtime();
    let main = native.main_branch_identity();
    let (before, basis) = native.observe_branch(&main).unwrap();
    let target = before
        .reference()
        .target()
        .as_basis()
        .expect("actual published predecessor");
    let expected_commit = target.selected_commit_id();
    let expected_roots = target.roots().clone();
    let expected_schema = before.schema_commitment();
    assert_ne!(expected_commit, released_receipt.commit_id.0);
    drop(basis);
    let (_, source) = native.observe_fork_source(main.branch_id()).unwrap();
    let child = BranchId("released-source-native-fork".into());
    let forked = native
        .fork_branch(&execution, child.clone(), source)
        .unwrap();
    let identity = native.branch_identity(&child).unwrap();
    let (fork, fork_basis) = native.observe_branch(&identity).unwrap();
    assert_eq!(
        fork.reference()
            .target()
            .as_basis()
            .unwrap()
            .selected_commit_id(),
        expected_commit
    );
    assert_eq!(fork.reference().generation().get(), 0);
    drop((forked, fork_basis));
    // No second Home checkpoint: the next receiver must replay this genuine
    // positioned transition from the earlier released-successor image.
    drop(fixture);

    let fixture = open(&location, &execution);
    let native = fixture.home.relational().runtime();
    let identity = native.branch_identity(&child).unwrap();
    let (fork, basis) = native.observe_branch(&identity).unwrap();
    let selected = fork.reference().target().as_basis().unwrap();
    assert_eq!(selected.selected_commit_id(), expected_commit);
    assert_eq!(selected.roots(), &expected_roots);
    assert_eq!(fork.schema_commitment(), expected_schema);
    assert_eq!(fork.reference().generation().get(), 0);
    assert_eq!(fork.truth_version().as_u64(), 0);
    assert_eq!(native.history().immutable_commit_count(), 3);
    let current = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(current.selected_commit().ordinal(), expected_product);
    let originals = fixture
        .home
        .world()
        .recovery_port()
        .released_original_page(&execution, None, maximum())
        .unwrap();
    assert_eq!(originals.rows().len(), 1);
    assert_eq!(
        originals.rows()[0].native_receipt(),
        Some(&released_receipt)
    );
    drop((basis, current, originals));
    close(fixture.home, &execution);
    // The next Complete image admits the same ordinary Native sibling through
    // the normal image path, without adding a World branch or a replay door.
    let fixture = open(&location, &execution);
    let native = fixture.home.relational().runtime();
    let (fork, basis) = native
        .observe_branch(&native.branch_identity(&child).unwrap())
        .unwrap();
    assert_eq!(
        fork.reference()
            .target()
            .as_basis()
            .unwrap()
            .selected_commit_id(),
        expected_commit
    );
    drop(basis);
    close(fixture.home, &execution);
}
