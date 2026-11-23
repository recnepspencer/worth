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

#[test]
fn exact_reuse_branch_and_signal_only_fork_preserve_horizon_splice_distance_and_facts() {
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let max = std::num::NonZeroUsize::new(10).unwrap();

    // Part 1 (B6a boundary): root1 -> src0 (pinned, gen 0) -> Q (gen 1) -> R_parent (gen 2, depth 3).
    // Creating an exact-reuse branch at Q adds NO commit edge; disposing Q keeps distance(R_parent, src0) == 2 (< 3).
    let dir1 = tempfile::tempdir().unwrap();
    let fx1 = open_home(dir1.path(), &execution);
    let boot1 = fx1.bootstrap_root(&execution);
    let root1 = publish_round(&fx1, &execution, boot1.product_branch().clone(), 0);
    drop(boot1);
    let src0 = create_branch(&fx1, &execution, &root1, "reuse-parent", true, true);
    let src0_ordinal = src0.selected_commit().ordinal();
    let pin_src0 = create_branch(&fx1, &execution, &src0, "pin-src0", true, true);
    let q = publish_round(&fx1, &execution, src0, 1);
    let q_ordinal = q.selected_commit().ordinal();
    let reused_at_q = create_branch(&fx1, &execution, &q, "reuse-at-q", false, false);
    assert_eq!(reused_at_q.selected_commit().ordinal(), q_ordinal);
    let reused_at_q_report = fx1.home.world().branch_port().retire_product_branch(&reused_at_q).unwrap();
    drop((reused_at_q, reused_at_q_report));
    let r_parent = publish_round(&fx1, &execution, q, 2);
    let r_parent_ordinal = r_parent.selected_commit().ordinal();
    let disposed1 = fx1.home.world().recovery_port().retire_changed_history().unwrap();
    assert!(disposed1.iter().any(|c| c.ordinal() == q_ordinal));
    let parent_before: Vec<u64> = fx1.home.world().observation_port()
        .trace_product_branch_ancestry(&execution, r_parent.lifecycle_incarnation(), None).unwrap()
        .commits().map(|c| c.identity().ordinal()).collect();
    assert_eq!(parent_before, vec![r_parent_ordinal, src0_ordinal]);
    drop((root1, pin_src0, r_parent));
    crate::process::application::world_occurrence_readmission::close(fx1.home, &execution);

    let re1 = open_home(dir1.path(), &execution);
    let inst1 = re1.home.world().observation_port().observe_installed_product_branches(&execution).unwrap();
    let parent_obs = inst1.branches().iter()
        .find(|b| b.branch_identity().name().as_str() == "reuse-parent").unwrap().clone();
    let parent_after: Vec<u64> = re1.home.world().observation_port()
        .trace_product_branch_ancestry(&execution, parent_obs.lifecycle_incarnation(), None).unwrap()
        .commits().map(|c| c.identity().ordinal()).collect();
    assert_eq!(parent_after, parent_before);
    drop((inst1, parent_obs));
    crate::process::application::world_occurrence_readmission::close(re1.home, &execution);

    // Part 2 (B6a live advanced exact-reuse branch): advance exact-reuse branch past disposed Q2.
    let dir2 = tempfile::tempdir().unwrap();
    let fx2 = open_home(dir2.path(), &execution);
    let boot2 = fx2.bootstrap_root(&execution);
    let root2 = publish_round(&fx2, &execution, boot2.product_branch().clone(), 0);
    drop(boot2);
    let src2_0 = create_branch(&fx2, &execution, &root2, "reuse-src2", true, true);
    let q2 = publish_round(&fx2, &execution, src2_0, 1);
    let q2_ordinal = q2.selected_commit().ordinal();
    let reused_live0 = create_branch(&fx2, &execution, &q2, "exact-reuse-live", false, false);
    assert_eq!(reused_live0.selected_commit().ordinal(), q2_ordinal);
    let src2_report = fx2.home.world().branch_port().retire_product_branch(&q2).unwrap();
    drop((q2, src2_report));
    let reused_live1 = publish_round(&fx2, &execution, reused_live0, 2);
    let reused_live1_ordinal = reused_live1.selected_commit().ordinal();
    let disposed2 = fx2.home.world().recovery_port().retire_changed_history().unwrap();
    assert!(disposed2.iter().any(|c| c.ordinal() == q2_ordinal));
    let reused_trace_before: Vec<u64> = fx2.home.world().inspection_port()
        .trace_ancestry(reused_live1.selected_commit().clone(), max).unwrap()
        .commits().map(|c| c.identity().ordinal()).collect();
    assert_eq!(reused_trace_before, vec![reused_live1_ordinal, root2.selected_commit().ordinal()]);
    let reused_branch_before: Vec<u64> = fx2.home.world().observation_port()
        .trace_product_branch_ancestry(&execution, reused_live1.lifecycle_incarnation(), None).unwrap()
        .commits().map(|c| c.identity().ordinal()).collect();
    assert_eq!(reused_branch_before, vec![reused_live1_ordinal]);
    drop((root2, reused_live1));
    crate::process::application::world_occurrence_readmission::close(fx2.home, &execution);

    let re2 = open_home(dir2.path(), &execution);
    let inst2 = re2.home.world().observation_port().observe_installed_product_branches(&execution).unwrap();
    let reused_obs = inst2.branches().iter()
        .find(|b| b.branch_identity().name().as_str() == "exact-reuse-live").unwrap().clone();
    let reused_trace_after: Vec<u64> = re2.home.world().inspection_port()
        .trace_ancestry(reused_obs.selected_commit().clone(), max).unwrap()
        .commits().map(|c| c.identity().ordinal()).collect();
    assert_eq!(reused_trace_after, reused_trace_before);
    let reused_branch_after: Vec<u64> = re2.home.world().observation_port()
        .trace_product_branch_ancestry(&execution, reused_obs.lifecycle_incarnation(), None).unwrap()
        .commits().map(|c| c.identity().ordinal()).collect();
    assert_eq!(reused_branch_after, reused_branch_before);
    drop((inst2, reused_obs));
    crate::process::application::world_occurrence_readmission::close(re2.home, &execution);

    // Part 3 (B6b positive): Signal-only horizon fork whose source branch commits are disposed.
    let sig_dir = tempfile::tempdir().unwrap();
    let sig_fx = open_home(sig_dir.path(), &execution);
    let sig_boot = sig_fx.bootstrap_root(&execution);
    let sig_root1 = publish_round(&sig_fx, &execution, sig_boot.product_branch().clone(), 0);
    drop(sig_boot);
    let sig_src0 = create_branch(&sig_fx, &execution, &sig_root1, "sig-src", true, true);
    let sig_src1 = publish_round(&sig_fx, &execution, sig_src0, 1);
    let sig_src1_ordinal = sig_src1.selected_commit().ordinal();
    let sig_only = create_branch(&sig_fx, &execution, &sig_src1, "sig-only-child", false, true);
    let sig_only_ordinal = sig_only.selected_commit().ordinal();
    let sig_src_report = sig_fx.home.world().branch_port().retire_product_branch(&sig_src1).unwrap();
    drop((sig_src1, sig_src_report));
    let sig_disposed = sig_fx.home.world().recovery_port().retire_changed_history().unwrap();
    assert!(sig_disposed.iter().any(|c| c.ordinal() == sig_src1_ordinal));
    drop((sig_root1, sig_only));
    crate::process::application::world_occurrence_readmission::close(sig_fx.home, &execution);

    let sig_reopened = open_home(sig_dir.path(), &execution);
    let sig_installed = sig_reopened
        .home
        .world()
        .observation_port()
        .observe_installed_product_branches(&execution)
        .unwrap();
    let sig_obs = sig_installed
        .branches()
        .iter()
        .find(|b| b.branch_identity().name().as_str() == "sig-only-child")
        .unwrap()
        .clone();
    let sig_trace = sig_reopened
        .home
        .world()
        .inspection_port()
        .trace_ancestry(sig_obs.selected_commit().clone(), max)
        .unwrap();
    let sig_commit = sig_trace.commits().next().unwrap();
    assert_eq!(sig_commit.identity().ordinal(), sig_only_ordinal);
    assert_eq!(sig_commit.relational_change(), CompositeComponentChangePosture::RetainExact);
    assert_eq!(sig_commit.signal_change(), CompositeComponentChangePosture::Published);
    assert!(sig_commit.relational_fork_target_identity().is_none());
    assert!(matches!(
        sig_commit.signal_publication_identity(),
        Some(CompositeSignalPublicationIdentity::Forked(_))
    ));
    assert!(matches!(
        sig_commit.parent(),
        CompositeCommitParent::Ordinary(parent) if parent.commit().ordinal() == sig_src1_ordinal
    ));
    drop((sig_trace, sig_installed, sig_obs));
    crate::process::application::world_occurrence_readmission::close(sig_reopened.home, &execution);
}
