//! Durable history disposal court: retired and inherited occurrences are
//! disposed once, their pins/envelopes are released and counted, and reopen
//! does not resurrect them.
use super::{create_branch, open_home, publish_round};
use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::*;

#[test]
fn durable_history_disposal_releases_retired_and_inherited_occurrences_once_without_resurrection() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open_home(directory.path(), &execution);
    let bootstrap = fixture.bootstrap_root(&execution);
    let root0 = bootstrap.product_branch().clone();
    let root0_commit = root0.selected_commit().ordinal();
    drop(bootstrap);
    let root1 = publish_round(&fixture, &execution, root0, 0);
    let root1_commit = root1.selected_commit().ordinal();

    let sibling0 = create_branch(&fixture, &execution, &root1, "disposed-sibling", true, true);
    let sibling1 = publish_round(&fixture, &execution, sibling0, 1);
    let sibling1_commit = sibling1.selected_commit().clone();
    let sibling1_ordinal = sibling1_commit.ordinal();

    let root2 = publish_round(&fixture, &execution, root1, 2);
    let root2_ordinal = root2.selected_commit().ordinal();

    let report = fixture
        .home
        .world()
        .branch_port()
        .retire_product_branch(&sibling1)
        .unwrap();
    drop((sibling1, root2));

    let before_history = fixture
        .home
        .world()
        .inspection_port()
        .history_snapshot()
        .unwrap()
        .installed_commits();
    let before_pins = fixture
        .home
        .world()
        .inspection_port()
        .retention_snapshot()
        .unwrap()
        .unique_pins();
    assert_eq!(before_history, 5);

    // 1. Dispose retired branch occurrences (sibling1 and sibling0).
    let branch_release = fixture
        .home
        .world()
        .recovery_port()
        .release_retired_branch_history(report.retired_head())
        .unwrap();
    assert_eq!(branch_release.reclaimed_commits().len(), 2);
    assert!(branch_release.held_commit().is_none());
    // Re-running does not dispose twice.
    assert!(fixture
        .home
        .world()
        .recovery_port()
        .release_retired_branch_history(report.retired_head())
        .unwrap()
        .reclaimed_commits()
        .is_empty());

    // 2. Dispose inherited unprotected ancestors (root0 and root1).
    let interior_retired = fixture
        .home
        .world()
        .recovery_port()
        .retire_changed_history()
        .unwrap();
    assert_eq!(interior_retired.len(), 2);
    assert!(interior_retired.iter().any(|c| c.ordinal() == root0_commit));
    assert!(interior_retired.iter().any(|c| c.ordinal() == root1_commit));
    // Re-running does not dispose twice.
    assert!(fixture
        .home
        .world()
        .recovery_port()
        .retire_changed_history()
        .unwrap()
        .is_empty());

    let after_history = fixture
        .home
        .world()
        .inspection_port()
        .history_snapshot()
        .unwrap()
        .installed_commits();
    let after_pins = fixture
        .home
        .world()
        .inspection_port()
        .retention_snapshot()
        .unwrap()
        .unique_pins();
    assert_eq!(after_history, 1);
    assert!(after_pins < before_pins);

    drop(report);
    crate::process::application::world_occurrence_readmission::close(fixture.home, &execution);

    // Reopen must NOT resurrect disposed commits.
    let reopened = open_home(directory.path(), &execution);
    assert_eq!(
        reopened
            .home
            .world()
            .inspection_port()
            .history_snapshot()
            .unwrap()
            .installed_commits(),
        1
    );
    assert_eq!(
        reopened
            .home
            .world()
            .inspection_port()
            .retention_snapshot()
            .unwrap()
            .unique_pins(),
        after_pins
    );
    let durable_owner = reopened.home.world().owner_identity().durable().unwrap();
    for disposed in [root0_commit, root1_commit, sibling1_ordinal] {
        let id = worth_foundational::facade::DurableCompositeCommitIdentity::from_recorded(
            durable_owner,
            disposed,
        );
        assert!(matches!(
            reopened
                .home
                .world()
                .observation_port()
                .readmitted_publication(&id),
            Err(RuntimeWorldServiceDenial::Denied(
                WorldReadmittedPublicationDenial::NotReadmitted
            ))
        ));
    }
    let live = reopened
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(live.selected_commit().ordinal(), root2_ordinal);
    drop(live);
    crate::process::application::world_occurrence_readmission::close(reopened.home, &execution);
}
