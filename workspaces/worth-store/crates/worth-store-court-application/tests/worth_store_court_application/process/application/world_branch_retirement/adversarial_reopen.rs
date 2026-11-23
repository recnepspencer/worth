//! Adversarial cold-reopen courts for parent retirement (B1), released custody
//! after predecessor disposal (B5), and horizon history fidelity (B6).
use super::{create_branch, open_home, publish_round, quiescent_kill};
use std::num::NonZeroUsize;
use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::*;

fn maximum() -> NonZeroUsize {
    NonZeroUsize::new(16).unwrap()
}

#[test]
fn retiring_parent_keeps_surviving_child_and_reused_parent_name_reopenable() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open_home(directory.path(), &execution);
    let bootstrap = fixture.bootstrap_root(&execution);
    let root1 = publish_round(&fixture, &execution, bootstrap.product_branch().clone(), 0);
    drop(bootstrap);

    // Create parent-a, fork child-b from parent-a at gen 0, advance parent-a to gen 1,
    // retire parent-a and dispose its unshared gen 1 commit while gen 0 stays for child-b.
    let a0 = create_branch(&fixture, &execution, &root1, "parent-a", true, true);
    let old_incarnation = a0.lifecycle_incarnation().ordinal();
    let child_b0 = create_branch(&fixture, &execution, &a0, "child-b", true, true);
    let child_b1 = publish_round(&fixture, &execution, child_b0, 1);
    let child_ordinal = child_b1.selected_commit().ordinal();
    let child_native = child_b1
        .basis()
        .relational_basis()
        .descriptor()
        .branch_id()
        .0
        .clone();
    let child_signal = child_b1.basis().signal_basis().branch_id();

    let a1 = publish_round(&fixture, &execution, a0, 2);
    let a1_ordinal = a1.selected_commit().ordinal();
    let report = fixture
        .home
        .world()
        .branch_port()
        .retire_product_branch(&a1)
        .unwrap();
    drop((a1, child_b1));
    let released = fixture
        .home
        .world()
        .recovery_port()
        .release_retired_branch_history(report.retired_head())
        .unwrap();
    assert_eq!(released.reclaimed_commits().len(), 1);
    assert_eq!(released.reclaimed_commits()[0].ordinal(), a1_ordinal);
    drop(report);

    // Reuse the retired parent's name under a new incarnation (with distinct owner fork targets).
    let reused_a = super::create_branch_with_fork_prefix(
        &fixture,
        &execution,
        &root1,
        "parent-a",
        "parent-a-v2",
        true,
        true,
    );
    let new_incarnation = reused_a.lifecycle_incarnation().ordinal();
    assert_ne!(old_incarnation, new_incarnation);
    drop((root1, reused_a));

    crate::process::application::world_occurrence_readmission::close(fixture.home, &execution);

    // Cold reopen must succeed with child-b live, new parent-a live, and old parent-a retired.
    let reopened = open_home(directory.path(), &execution);
    let installed = reopened
        .home
        .world()
        .observation_port()
        .observe_installed_product_branches(&execution)
        .unwrap();
    assert_eq!(installed.branches().len(), 3);
    let child = installed
        .branches()
        .iter()
        .find(|b| b.branch_identity().name().as_str() == "child-b")
        .expect("child-b must survive parent-a retirement");
    assert_eq!(child.selected_commit().ordinal(), child_ordinal);
    assert_eq!(
        child
            .basis()
            .relational_basis()
            .descriptor()
            .branch_id()
            .0,
        child_native
    );
    assert_eq!(child.basis().signal_basis().branch_id(), child_signal);
    let reused = installed
        .branches()
        .iter()
        .find(|b| b.branch_identity().name().as_str() == "parent-a")
        .expect("reused parent-a must reopen");
    assert_eq!(reused.lifecycle_incarnation().ordinal(), new_incarnation);
    drop(installed);
    crate::process::application::world_occurrence_readmission::close(reopened.home, &execution);
}

#[test]
fn released_unpublished_with_native_effect_reopens_after_predecessor_disposal() {
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let markers = directory.path().join("markers");
    std::fs::create_dir(&markers).unwrap();
    assert_eq!(
        crate::killed_child::run_and_kill(
            "process::application::world_durable_release::child::original_world_attempt_child",
            &[
                ("WORTH_WORLD_RELEASE_HOME", location.as_path()),
                ("WORTH_WORLD_RELEASE_MARKERS", markers.as_path()),
                (
                    "WORTH_WORLD_RELEASE_PUBLISHED_PREFIX",
                    std::path::Path::new("two"),
                ),
            ],
            &markers.join("ready"),
            &markers.join("finished"),
        ),
        crate::killed_child::ChildEnd::KilledAtPark
    );
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open_home(&location, &execution);
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
    let expected_receipt = claim.native_receipt().unwrap();
    let expected_attempt = claim.attempt_identity().ordinal();
    let mut outcome = recovery.release_recovered_effects(&execution, claim);
    let released = loop {
        match outcome {
            WorldRecoveredReleaseOutcome::Released(released) => break released,
            WorldRecoveredReleaseOutcome::Pending(claim) => {
                outcome = recovery.release_recovered_effects(&execution, claim)
            }
            other => panic!("unexpected release outcome: {other:?}"),
        }
    };
    assert_eq!(
        released.original().native_receipt().unwrap(),
        &expected_receipt
    );
    drop((released, page));
    // Graceful reopen selects the released predecessor head in Native before a fresh edit.
    crate::process::application::world_occurrence_readmission::close(fixture.home, &execution);
    let fixture = open_home(&location, &execution);

    // Advance root past the released original's predecessor and dispose that predecessor!
    let head = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let predecessor_ordinal = head.selected_commit().ordinal();
    let next = publish_round(&fixture, &execution, head, 3);
    let next_ordinal = next.selected_commit().ordinal();
    drop(next);
    let disposed = fixture
        .home
        .world()
        .recovery_port()
        .retire_changed_history()
        .unwrap();
    assert!(disposed.iter().any(|c| c.ordinal() == predecessor_ordinal));
    crate::process::application::world_occurrence_readmission::close(fixture.home, &execution);

    // Reopen must succeed with predecessor disposed AND Track H released original intact.
    let reopened = open_home(&location, &execution);
    let live = reopened
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(live.selected_commit().ordinal(), next_ordinal);
    let released_page = reopened
        .home
        .world()
        .recovery_port()
        .released_original_page(&execution, None, maximum())
        .unwrap();
    assert_eq!(released_page.rows().len(), 1);
    let row = &released_page.rows()[0];
    assert_eq!(row.attempt_identity().ordinal(), expected_attempt);
    assert_eq!(row.native_receipt().unwrap(), &expected_receipt);
    assert_eq!(
        row.native_semantic_result().unwrap().created_entities().len(),
        1
    );
    drop((live, released_page));
    crate::process::application::world_occurrence_readmission::close(reopened.home, &execution);
}

#[test]
fn released_unpublished_no_effect_reopens_after_predecessor_disposal() {
    let root = quiescent_kill::run_killed("open-no-effect");
    let fields: Vec<u64> = std::fs::read_to_string(root.path().join("markers/open-no-effect"))
        .unwrap()
        .lines()
        .map(|s| s.parse::<u64>().unwrap())
        .collect();
    let (expected_attempt, predecessor_ordinal) = (fields[0], fields[1]);
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open_home(&root.path().join("home"), &execution);
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
    assert!(claim.native_receipt().is_none());
    let mut outcome = recovery.release_recovered_effects(&execution, claim);
    let released = loop {
        match outcome {
            WorldRecoveredReleaseOutcome::Released(released) => break released,
            WorldRecoveredReleaseOutcome::Pending(claim) => {
                outcome = recovery.release_recovered_effects(&execution, claim)
            }
            other => panic!("unexpected no-effect release outcome: {other:?}"),
        }
    };
    drop((released, page));

    let head = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let next = publish_round(&fixture, &execution, head, 2);
    let next_ordinal = next.selected_commit().ordinal();
    drop(next);
    let disposed = fixture
        .home
        .world()
        .recovery_port()
        .retire_changed_history()
        .unwrap();
    assert!(disposed.iter().any(|c| c.ordinal() == predecessor_ordinal));
    crate::process::application::world_occurrence_readmission::close(fixture.home, &execution);

    let reopened = open_home(&root.path().join("home"), &execution);
    let live = reopened
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(live.selected_commit().ordinal(), next_ordinal);
    let released_page = reopened
        .home
        .world()
        .recovery_port()
        .released_original_page(&execution, None, maximum())
        .unwrap();
    assert_eq!(released_page.rows().len(), 1);
    assert_eq!(
        released_page.rows()[0].attempt_identity().ordinal(),
        expected_attempt
    );
    assert!(released_page.rows()[0].native_receipt().is_none());
    drop((live, released_page));
    crate::process::application::world_occurrence_readmission::close(reopened.home, &execution);
}
