//! Durable historical-root forks, divergent equal-looking edits, archive and
//! delete across crash cuts, and permanent name retirement (Store M4.1, D3a).
use super::branch_history_publications::{create_entity_outcome_on_branch, fork};
use super::branch_history_recovery::{checkpoint, current_head, recover};
use worth_execution::ExecutionRequest;
use worth_relational::facade::branch::{
    RelationalBranchLifecyclePosture as Posture, RelationalBranchTransitionStop as Stop,
    RelationalForkDenial,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_testkit_relational::aspect_storage::publications::release_test_commit_snapshot;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::durable_runtime;

fn branch(name: &str) -> BranchId {
    BranchId(name.into())
}

fn commit(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
    on: &str,
) -> worth_relational::facade::history::CommitId {
    let created = create_entity_outcome_on_branch(execution, runtime, key, branch(on));
    let commit = created.commit.commit_id;
    release_test_commit_snapshot(runtime, &created);
    commit
}

fn heads(
    runtime: &RelationalRuntime,
) -> [Option<worth_relational::facade::history::RelationalCommitReceipt>; 2] {
    [
        current_head(runtime, &branch("b1")),
        current_head(runtime, &branch("b2")),
    ]
}

#[test]
fn durable_historical_fork_keeps_its_exact_source_across_archive_delete_and_crash_cuts() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let medium = TestDurableMedium::new();
    let mut runtime = durable_runtime(&medium);
    commit(execution, &runtime, "seed", "main");
    fork(execution, &runtime, "b1");
    let c0 = commit(execution, &runtime, "c0", "b1");
    let services = runtime.owner_component_services();
    let b1 = runtime.branch_identity(&branch("b1")).unwrap();
    let retained = services.basis_port().admit_branch_basis(&b1).unwrap();
    // b1 moves past c0 before the historical fork is taken.
    commit(execution, &runtime, "b1-after-c0", "b1");
    let source = services
        .fork_port()
        .admit_retained_fork_source(&retained)
        .unwrap();
    let (forked, b2_basis) = services
        .fork_port()
        .fork_retained_branch(execution, branch("b2"), source)
        .unwrap();
    assert_eq!(forked.shared_commit_id(), Some(c0));
    assert_eq!(b2_basis.observation().commit_id(), Some(c0));
    drop((b2_basis, retained, services));
    // Divergent edits with equal-looking values never collapse the histories.
    let b1_same = commit(execution, &runtime, "same-value", "b1");
    let b2_same = commit(execution, &runtime, "same-value", "b2");
    assert_ne!(b1_same, b2_same);
    let live = heads(&runtime);
    assert_ne!(live[0], live[1]);
    assert_eq!(
        live[1].as_ref().unwrap().parents,
        vec![c0],
        "b2 is exactly c0 plus its own delta"
    );

    // Crash cut 1: the raw tail replays the durable historical fork.
    let replayed = recover(execution, &medium, durable_runtime(&medium)).1;
    assert_eq!(heads(&replayed), live);

    // Crash cut 2: archive b1; b2 keeps its own head and parentage.
    runtime.archive_branch(execution, &b1).unwrap();
    let archived = recover(execution, &medium, durable_runtime(&medium)).1;
    assert_eq!(
        archived
            .branch_reference_state(&branch("b1"))
            .unwrap()
            .lifecycle_posture(),
        Posture::Archived
    );
    let archived_head = archived
        .branch_reference_state(&branch("b1"))
        .unwrap()
        .observation()
        .target()
        .as_basis()
        .map(|head| head.selected_commit_id());
    assert_eq!(archived_head, live[0].as_ref().map(|head| head.commit_id.0));
    assert_eq!(current_head(&archived, &branch("b2")), live[1]);

    // Crash cut 3: checkpoint, then delete b1 permanently; b2 is untouched.
    checkpoint(execution, &runtime, &medium);
    runtime.delete_branch(execution, &b1).unwrap();
    for cut in ["tail-after-checkpoint", "checkpoint-covered"] {
        if cut == "checkpoint-covered" {
            checkpoint(execution, &runtime, &medium);
        }
        let recovered = recover(execution, &medium, durable_runtime(&medium)).1;
        assert!(recovered.branch_identity(&branch("b1")).is_err(), "{cut}");
        assert_eq!(current_head(&recovered, &branch("b2")), live[1], "{cut}");
        let (_, main) = recovered.observe_fork_source(&branch("main")).unwrap();
        assert!(
            matches!(
                recovered.fork_branch(execution, branch("b1"), main),
                Err(Stop::Denied(RelationalForkDenial::RetiredTarget))
            ),
            "{cut}: a retired name is never recreated"
        );
    }
    let continued = recover(execution, &medium, durable_runtime(&medium)).1;
    let next = commit(execution, &continued, "after-recovery", "b2");
    let again = recover(execution, &medium, durable_runtime(&medium)).1;
    assert_eq!(current_head(&again, &branch("b2")).unwrap().commit_id, next);
}

#[test]
fn a_historical_fork_taken_after_a_checkpoint_rebuilds_its_source_from_the_checkpoint() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let c0 = commit(execution, &runtime, "c0", "main");
    let services = runtime.owner_component_services();
    let retained = services
        .basis_port()
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let held = runtime.retain_component_basis(&retained).unwrap();
    commit(execution, &runtime, "c1", "main");
    // The checkpoint covers c0 only as a retained component image.
    checkpoint(execution, &runtime, &medium);
    let source = services
        .fork_port()
        .admit_retained_fork_source(&retained)
        .unwrap();
    services
        .fork_port()
        .fork_retained_branch(execution, branch("from-c0"), source)
        .unwrap();
    runtime.release_component_basis(held).unwrap();
    let recovered = recover(execution, &medium, durable_runtime(&medium)).1;
    let head = current_head(&recovered, &branch("from-c0")).unwrap();
    assert_eq!(head.commit_id, c0);
    assert_eq!(
        current_head(&recovered, &branch("main")),
        current_head(&runtime, &branch("main"))
    );
}
