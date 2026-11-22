//! Horizon history fidelity court: exact publication/fork facts, parent,
//! splice distance, and retirement boundary survive interior/source disposal (B6).
use super::{create_branch, open_home, publish_round};
use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::*;

#[test]
fn horizon_replay_preserves_exact_publication_facts_splice_distance_and_fork_boundary() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open_home(directory.path(), &execution);
    let bootstrap = fixture.bootstrap_root(&execution);
    let root0 = bootstrap.product_branch().clone();
    drop(bootstrap);
    // Advance root0 -> P so P has a non-empty Native basis before forking pin-p.
    let p = publish_round(&fixture, &execution, root0, 0);
    let p_ordinal = p.selected_commit().ordinal();

    // Pin P with a sibling so P survives while interior Q (P -> Q -> R) is disposed.
    let pin_p = create_branch(&fixture, &execution, &p, "pin-p", true, true);
    let q = publish_round(&fixture, &execution, p, 1);
    let q_ordinal = q.selected_commit().ordinal();
    let r = publish_round(&fixture, &execution, q, 2);
    let r_ordinal = r.selected_commit().ordinal();

    // Also create a horizon-created child whose source branch commits are disposed.
    let src0 = create_branch(&fixture, &execution, &r, "src-branch", true, true);
    let src1 = publish_round(&fixture, &execution, src0, 3);
    let src1_ordinal = src1.selected_commit().ordinal();
    let hchild = create_branch(&fixture, &execution, &src1, "horizon-child", true, true);
    let hchild_ordinal = hchild.selected_commit().ordinal();
    let src_report = fixture
        .home
        .world()
        .branch_port()
        .retire_product_branch(&src1)
        .unwrap();
    drop((src1, src_report, pin_p));

    // Dispose interior Q (between P and R) and src0/src1 (between R and horizon-child).
    let disposed = fixture
        .home
        .world()
        .recovery_port()
        .retire_changed_history()
        .unwrap();
    assert!(disposed.iter().any(|c| c.ordinal() == q_ordinal));
    assert!(disposed.iter().any(|c| c.ordinal() == src1_ordinal));

    // Record exact live commit facts & spliced traversal BEFORE close.
    let max = std::num::NonZeroUsize::new(10).unwrap();
    let root_trace_before = fixture
        .home
        .world()
        .inspection_port()
        .trace_ancestry(r.selected_commit().clone(), max)
        .unwrap();
    let r_before = root_trace_before.commits().next().unwrap();
    assert_eq!(r_before.identity().ordinal(), r_ordinal);
    assert_eq!(
        r_before.relational_change(),
        CompositeComponentChangePosture::Published
    );
    assert_eq!(
        r_before.signal_change(),
        CompositeComponentChangePosture::RetainExact
    );
    let r_rel_commit = r_before
        .relational_publication_identity()
        .unwrap()
        .commit_id();
    assert!(r_before.signal_publication_identity().is_none());
    assert!(root_trace_before.crossed_retired_history());
    assert_eq!(root_trace_before.commits().len(), 2);

    let child_trace_before = fixture
        .home
        .world()
        .inspection_port()
        .trace_ancestry(hchild.selected_commit().clone(), max)
        .unwrap();
    let hc_before = child_trace_before.commits().next().unwrap();
    assert_eq!(hc_before.identity().ordinal(), hchild_ordinal);
    assert_eq!(
        hc_before.relational_change(),
        CompositeComponentChangePosture::Published
    );
    assert_eq!(
        hc_before.signal_change(),
        CompositeComponentChangePosture::Published
    );
    assert!(hc_before.relational_fork_target_identity().is_some());
    assert!(matches!(
        hc_before.signal_publication_identity(),
        Some(CompositeSignalPublicationIdentity::Forked(_))
    ));
    assert!(child_trace_before.crossed_retired_history());
    drop((root_trace_before, child_trace_before, r, hchild));

    crate::process::application::world_occurrence_readmission::close(fixture.home, &execution);

    // Reopen and verify exact publication/fork facts, parent, splice distance, and boundary!
    let reopened = open_home(directory.path(), &execution);
    let root_obs = reopened
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(root_obs.selected_commit().ordinal(), r_ordinal);
    let root_trace_after = reopened
        .home
        .world()
        .inspection_port()
        .trace_ancestry(root_obs.selected_commit().clone(), max)
        .unwrap();
    assert!(root_trace_after.crossed_retired_history());
    // Bounded branch ancestry (branch depth = gen 2 + 1 = 3) must ALSO reach P at exact spliced distance 2.
    let root_branch_trace = reopened
        .home
        .world()
        .observation_port()
        .trace_product_branch_ancestry(&execution, root_obs.lifecycle_incarnation(), None)
        .unwrap();
    assert_eq!(root_branch_trace.commits().len(), 2);
    drop(root_branch_trace);
    let root_commits: Vec<_> = root_trace_after.commits().collect();
    assert_eq!(root_commits.len(), 2);
    assert_eq!(root_commits[0].identity().ordinal(), r_ordinal);
    assert_eq!(root_commits[1].identity().ordinal(), p_ordinal);
    assert_eq!(
        root_commits[0].relational_change(),
        CompositeComponentChangePosture::Published
    );
    assert_eq!(
        root_commits[0].signal_change(),
        CompositeComponentChangePosture::RetainExact
    );
    assert_eq!(
        root_commits[0]
            .relational_publication_identity()
            .unwrap()
            .commit_id(),
        r_rel_commit
    );
    assert!(root_commits[0].signal_publication_identity().is_none());
    assert!(matches!(
        root_commits[0].parent(),
        CompositeCommitParent::Ordinary(parent) if parent.commit().ordinal() == q_ordinal
    ));

    let installed = reopened
        .home
        .world()
        .observation_port()
        .observe_installed_product_branches(&execution)
        .unwrap();
    let hc_obs = installed
        .branches()
        .iter()
        .find(|b| b.branch_identity().name().as_str() == "horizon-child")
        .unwrap()
        .clone();
    let child_trace_after = reopened
        .home
        .world()
        .inspection_port()
        .trace_ancestry(hc_obs.selected_commit().clone(), max)
        .unwrap();
    assert!(child_trace_after.crossed_retired_history());
    let hc_commits: Vec<_> = child_trace_after.commits().collect();
    assert_eq!(hc_commits.len(), 3);
    assert_eq!(hc_commits[0].identity().ordinal(), hchild_ordinal);
    assert_eq!(hc_commits[1].identity().ordinal(), r_ordinal);
    assert_eq!(hc_commits[2].identity().ordinal(), p_ordinal);
    let hc_after = hc_commits[0];
    assert_eq!(hc_after.identity().ordinal(), hchild_ordinal);
    assert_eq!(
        hc_after.relational_change(),
        CompositeComponentChangePosture::Published
    );
    assert_eq!(
        hc_after.signal_change(),
        CompositeComponentChangePosture::Published
    );
    assert!(hc_after.relational_fork_target_identity().is_some());
    assert!(matches!(
        hc_after.signal_publication_identity(),
        Some(CompositeSignalPublicationIdentity::Forked(_))
    ));
    assert!(matches!(
        hc_after.parent(),
        CompositeCommitParent::Ordinary(parent) if parent.commit().ordinal() == src1_ordinal
    ));
    drop((root_trace_after, child_trace_after, installed, root_obs));

    // Retiring horizon-child and releasing its history releases horizon-child and stops at R.
    let hc_report = reopened
        .home
        .world()
        .branch_port()
        .retire_product_branch(&hc_obs)
        .unwrap();
    drop(hc_obs);
    let hc_release = reopened
        .home
        .world()
        .recovery_port()
        .release_retired_branch_history(hc_report.retired_head())
        .unwrap();
    assert_eq!(hc_release.reclaimed_commits().len(), 1);
    assert_eq!(hc_release.reclaimed_commits()[0].ordinal(), hchild_ordinal);
    drop(hc_report);
    crate::process::application::world_occurrence_readmission::close(reopened.home, &execution);
}
