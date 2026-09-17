use worth_signal::facade::history::{RuntimeBranch, RuntimeSnapshot};

use crate::expression::model::SignalValue;
use crate::runtime::tests::worker_runtime::fixtures::portable_counter_graph::{
    set_counter, worker_shell_with_counter_graph,
};
use crate::runtime::worker_host::{
    WorkerApplyTransactionToBranchRequest, WorkerBranchBasisReceipt, WorkerBranchRetirementReason,
    WorkerCloseoutEffectBranchRequest, WorkerForkBranchReceipt, WorkerForkBranchRequest,
    WorkerRetireBranchRequest, WorkerRetireBranchesRequest, WorkerRuntimeShell,
};

#[test]
fn denied_worker_retirement_preserves_snapshot_authority() {
    let mut shell = worker_shell_with_counter_graph();
    let main = shell.current_branch();
    let main_basis = shell.worker_branch_basis(main.id.0).unwrap();
    let parent = shell
        .fork_worker_branch(WorkerForkBranchRequest {
            name: "snapshot-parent".to_owned(),
            parent_branch_id: main.id.0,
            expected_parent_basis: main_basis,
        })
        .unwrap();
    let retained_snapshot = shell.branch_snapshot(parent.branch.id.0).unwrap();
    let parent_basis = shell.worker_branch_basis(parent.branch.id.0).unwrap();
    shell
        .fork_worker_branch(WorkerForkBranchRequest {
            name: "live-child".to_owned(),
            parent_branch_id: parent.branch.id.0,
            expected_parent_basis: parent_basis.clone(),
        })
        .unwrap();

    let denial = shell
        .retire_worker_branch(WorkerRetireBranchRequest {
            branch_id: parent.branch.id.0,
            expected_basis: parent_basis,
            reason: WorkerBranchRetirementReason::Rejected,
        })
        .unwrap_err();

    assert!(denial.message.contains("LiveChildren"));
    shell
        .restore_branch_snapshot(parent.branch.id.0, retained_snapshot)
        .expect("denied retirement must preserve snapshot authority");
}

#[test]
fn worker_batch_retirement_releases_owned_snapshots_after_planning() {
    let mut shell = worker_shell_with_counter_graph();
    let main = shell.current_branch();
    let main_basis = shell.worker_branch_basis(main.id.0).unwrap();
    let parent = shell
        .fork_worker_branch(WorkerForkBranchRequest {
            name: "batch-parent".to_owned(),
            parent_branch_id: main.id.0,
            expected_parent_basis: main_basis,
        })
        .unwrap();
    let child = shell
        .fork_worker_branch(WorkerForkBranchRequest {
            name: "batch-child".to_owned(),
            parent_branch_id: parent.branch.id.0,
            expected_parent_basis: parent.created_basis,
        })
        .unwrap();
    shell.branch_snapshot(parent.branch.id.0).unwrap();
    shell.branch_snapshot(child.branch.id.0).unwrap();

    let receipt = shell
        .retire_worker_branches(WorkerRetireBranchesRequest {
            retirements: vec![
                WorkerRetireBranchRequest {
                    branch_id: child.branch.id.0,
                    expected_basis: shell.worker_branch_basis(child.branch.id.0).unwrap(),
                    reason: WorkerBranchRetirementReason::Rejected,
                },
                WorkerRetireBranchRequest {
                    branch_id: parent.branch.id.0,
                    expected_basis: shell.worker_branch_basis(parent.branch.id.0).unwrap(),
                    reason: WorkerBranchRetirementReason::DependencyCancellation,
                },
            ],
        })
        .unwrap();

    assert_eq!(receipt.retirements.len(), 2);
    assert_eq!(shell.branches(), vec![main]);
}

struct EffectCloseoutFixture {
    shell: WorkerRuntimeShell,
    /// The branch the effect settles into: the root, or (for
    /// `effect_closeout_fixture_under_branch_head`) a child of the root.
    main: RuntimeBranch,
    dependency: WorkerForkBranchReceipt,
    effect: WorkerForkBranchReceipt,
    coordinator: WorkerForkBranchReceipt,
    effect_snapshot: RuntimeSnapshot,
    dependency_snapshot: RuntimeSnapshot,
}

fn effect_closeout_fixture() -> EffectCloseoutFixture {
    let shell = worker_shell_with_counter_graph();
    let main = shell.current_branch();
    effect_closeout_fixture_on(shell, main)
}

/// The product forks resource effects from whatever branch it treats as
/// canonical; `createBranchHead` in the resource suite makes that a child of
/// the root. Closeout must settle into that branch, not into the root.
fn effect_closeout_fixture_under_branch_head() -> EffectCloseoutFixture {
    let mut shell = worker_shell_with_counter_graph();
    let root = shell.current_branch();
    let root_basis = shell.worker_branch_basis(root.id.0).unwrap();
    let head = shell
        .fork_worker_branch(WorkerForkBranchRequest {
            name: "canonical-head".to_owned(),
            parent_branch_id: root.id.0,
            expected_parent_basis: root_basis,
        })
        .unwrap();
    shell.switch_branch(head.branch.id.0).unwrap();
    let main = shell.current_branch();
    assert_eq!(main.id, head.branch.id);
    effect_closeout_fixture_on(shell, main)
}

fn effect_closeout_fixture_on(
    mut shell: WorkerRuntimeShell,
    main: RuntimeBranch,
) -> EffectCloseoutFixture {
    let main_basis = shell.worker_branch_basis(main.id.0).unwrap();
    let dependency = shell
        .fork_worker_branch(WorkerForkBranchRequest {
            name: "effect-dependency".to_owned(),
            parent_branch_id: main.id.0,
            expected_parent_basis: main_basis.clone(),
        })
        .unwrap();
    let effect = shell
        .fork_worker_branch(WorkerForkBranchRequest {
            name: "effect-work".to_owned(),
            parent_branch_id: dependency.branch.id.0,
            expected_parent_basis: dependency.created_basis.clone(),
        })
        .unwrap();
    let coordinator = shell
        .fork_worker_branch(WorkerForkBranchRequest {
            name: "closeout-coordinator".to_owned(),
            parent_branch_id: main.id.0,
            expected_parent_basis: main_basis,
        })
        .unwrap();
    shell.switch_branch(coordinator.branch.id.0).unwrap();
    let effect_snapshot = shell.branch_snapshot(effect.branch.id.0).unwrap();
    let dependency_snapshot = shell.branch_snapshot(dependency.branch.id.0).unwrap();
    EffectCloseoutFixture {
        shell,
        main,
        dependency,
        effect,
        coordinator,
        effect_snapshot,
        dependency_snapshot,
    }
}

fn closeout_request(
    fixture: &EffectCloseoutFixture,
    expected_main_basis: WorkerBranchBasisReceipt,
) -> WorkerCloseoutEffectBranchRequest {
    WorkerCloseoutEffectBranchRequest {
        canonical_transaction: WorkerApplyTransactionToBranchRequest {
            branch_id: fixture.main.id.0,
            expected_basis: expected_main_basis,
            transaction_ops: set_counter(23.0),
        },
        effect_retirement: WorkerRetireBranchRequest {
            branch_id: fixture.effect.branch.id.0,
            expected_basis: fixture
                .shell
                .worker_branch_basis(fixture.effect.branch.id.0)
                .unwrap(),
            reason: WorkerBranchRetirementReason::Rejected,
        },
        dependency_basis_retirement: Some(WorkerRetireBranchRequest {
            branch_id: fixture.dependency.branch.id.0,
            expected_basis: fixture
                .shell
                .worker_branch_basis(fixture.dependency.branch.id.0)
                .unwrap(),
            reason: WorkerBranchRetirementReason::DependencyCancellation,
        }),
    }
}

#[test]
fn worker_effect_closeout_releases_snapshots_only_after_canonical_commit() {
    let mut fixture = effect_closeout_fixture();
    let main_basis = fixture
        .shell
        .worker_branch_basis(fixture.main.id.0)
        .unwrap();
    let request = closeout_request(&fixture, main_basis);
    let receipt = fixture
        .shell
        .closeout_worker_effect_branch(request)
        .unwrap();

    assert_eq!(
        receipt.effect_retirement.retired_branch_id,
        fixture.effect.branch.id.0
    );
    assert_eq!(
        receipt
            .dependency_basis_retirement
            .expect("dependency retirement should be present")
            .retired_branch_id,
        fixture.dependency.branch.id.0
    );
    fixture.shell.switch_branch(fixture.main.id.0).unwrap();
    assert_eq!(
        fixture.shell.read_value("counter").unwrap(),
        SignalValue::Number(23.0)
    );
}

#[test]
fn failed_canonical_closeout_preserves_branches_and_snapshot_authority() {
    let mut fixture = effect_closeout_fixture();
    let stale_main_basis = fixture
        .shell
        .worker_branch_basis(fixture.main.id.0)
        .unwrap();
    fixture
        .shell
        .apply_transaction_to_worker_branch(WorkerApplyTransactionToBranchRequest {
            branch_id: fixture.main.id.0,
            expected_basis: stale_main_basis.clone(),
            transaction_ops: set_counter(7.0),
        })
        .unwrap();
    let request = closeout_request(&fixture, stale_main_basis);

    let denial = fixture
        .shell
        .closeout_worker_effect_branch(request)
        .unwrap_err();
    assert!(denial.message.contains("stale worker branch basis"));
    assert!(fixture
        .shell
        .branches()
        .iter()
        .any(|branch| branch.id == fixture.effect.branch.id));
    assert!(fixture
        .shell
        .branches()
        .iter()
        .any(|branch| branch.id == fixture.dependency.branch.id));
    fixture
        .shell
        .restore_branch_snapshot(fixture.effect.branch.id.0, fixture.effect_snapshot.clone())
        .expect("failed closeout must preserve effect snapshot authority");
    fixture
        .shell
        .restore_branch_snapshot(
            fixture.dependency.branch.id.0,
            fixture.dependency_snapshot.clone(),
        )
        .expect("failed closeout must preserve dependency snapshot authority");

    let main_basis = fixture
        .shell
        .worker_branch_basis(fixture.main.id.0)
        .unwrap();
    let retry = closeout_request(&fixture, main_basis);
    fixture
        .shell
        .closeout_worker_effect_branch(retry)
        .expect("closeout should succeed after the denied attempt");
}

#[test]
fn effect_closeout_rejects_a_retiring_branch_as_the_canonical_target() {
    let mut fixture = effect_closeout_fixture();
    let main_basis = fixture
        .shell
        .worker_branch_basis(fixture.main.id.0)
        .unwrap();
    let mut request = closeout_request(&fixture, main_basis);
    request.canonical_transaction.branch_id = fixture.effect.branch.id.0;
    request.canonical_transaction.expected_basis = fixture
        .shell
        .worker_branch_basis(fixture.effect.branch.id.0)
        .unwrap();

    let denial = fixture
        .shell
        .closeout_worker_effect_branch(request)
        .unwrap_err();
    assert!(
        denial.message.contains("is retired by this closeout"),
        "{}",
        denial.message
    );
    assert!(fixture
        .shell
        .branches()
        .iter()
        .any(|branch| branch.id == fixture.effect.branch.id));
}

#[test]
fn effect_closeout_settles_into_a_branch_head_below_the_root() {
    let mut fixture = effect_closeout_fixture_under_branch_head();
    assert!(
        fixture.main.parent_branch_id.is_some(),
        "the canonical head is a child of the root"
    );
    let head_basis = fixture
        .shell
        .worker_branch_basis(fixture.main.id.0)
        .unwrap();
    let request = closeout_request(&fixture, head_basis);

    let receipt = fixture
        .shell
        .closeout_worker_effect_branch(request)
        .expect("closeout settles into the canonical head the effect forked from");

    assert_eq!(
        receipt.effect_retirement.retired_branch_id,
        fixture.effect.branch.id.0
    );
    fixture.shell.switch_branch(fixture.main.id.0).unwrap();
    assert_eq!(
        fixture.shell.read_value("counter").unwrap(),
        SignalValue::Number(23.0)
    );
    let root = fixture
        .shell
        .branches()
        .into_iter()
        .find(|branch| branch.parent_branch_id.is_none())
        .expect("root branch survives");
    fixture.shell.switch_branch(root.id.0).unwrap();
    assert_ne!(
        fixture.shell.read_value("counter").unwrap(),
        SignalValue::Number(23.0),
        "the root does not receive a closeout addressed to the branch head"
    );
}

#[test]
fn effect_closeout_rejects_an_unrelated_live_branch_as_the_canonical_target() {
    let mut fixture = effect_closeout_fixture();
    let coordinator_id = fixture.coordinator.branch.id.0;
    let mut request = closeout_request(
        &fixture,
        fixture
            .shell
            .worker_branch_basis(fixture.main.id.0)
            .unwrap(),
    );
    request.canonical_transaction.branch_id = coordinator_id;
    request.canonical_transaction.expected_basis =
        fixture.shell.worker_branch_basis(coordinator_id).unwrap();

    let denial = fixture
        .shell
        .closeout_worker_effect_branch(request)
        .unwrap_err();
    assert!(
        denial
            .message
            .contains("must be the nearest surviving ancestor"),
        "{}",
        denial.message
    );
    assert!(fixture
        .shell
        .branches()
        .iter()
        .any(|branch| branch.id == fixture.effect.branch.id));
}

mod effect_parent;
