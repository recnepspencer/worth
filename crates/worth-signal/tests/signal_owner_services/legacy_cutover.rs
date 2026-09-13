use worth_proof::TransitionOutcome;
use worth_signal::facade::branch::{
    SignalBranchBasisReadmissionDenial, SignalBranchRetentionReleaseOutcome,
    SignalBranchRetirementBatchDenial,
};
use worth_signal::facade::history::{RuntimeBranch, RuntimeBranchId};
use worth_signal::facade::runtime::SignalBranchRetirementDenial;
use worth_signal::facade::{SignalBranchRetirementReason, SignalGraph, SignalRuntime};

type Runtime = SignalRuntime<(), (), (), (), ()>;

fn runtime() -> Runtime {
    SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build()
}

#[test]
fn legacy_root_calls_cross_issuance_without_a_second_branch_state_lane() {
    let mut runtime = runtime();
    let main = runtime.current_branch();
    let initial = runtime
        .observe_signal_branch_basis(main.clone())
        .expect("pre-issuance observation succeeds");
    let descriptor = initial.descriptor().clone();
    let pre_issuance_readmitted = runtime
        .readmit_signal_branch_basis(descriptor.clone())
        .expect("pre-issuance descriptor readmission succeeds");
    assert_eq!(pre_issuance_readmitted.observation(), initial.observation());
    drop(pre_issuance_readmitted);

    let pre_issuance_lease = runtime
        .retain_signal_component_basis(&initial)
        .expect("pre-issuance retention succeeds");
    assert!(matches!(
        runtime.release_signal_component_basis(pre_issuance_lease),
        SignalBranchRetentionReleaseOutcome::Released(_)
    ));
    let pre_issuance_terminals = runtime.signal_component_retention_terminal_counts();

    let _services = runtime
        .owner_component_services()
        .expect("the canonical partition seals once");

    let observed = runtime
        .observe_signal_branch_basis(main)
        .expect("the legacy observer delegates to the sealed owner");
    assert_eq!(observed.observation(), initial.observation());
    let readmitted = runtime
        .readmit_signal_branch_basis(descriptor)
        .expect("descriptor-only compatibility readmission delegates to the sealed owner");
    assert_eq!(readmitted.observation(), initial.observation());

    let mut unavailable =
        serde_json::to_value(initial.descriptor()).expect("the descriptor serializes");
    unavailable["observation"]["target"]["Basis"]["snapshot_id"] =
        serde_json::Value::from(u64::MAX);
    let unavailable =
        serde_json::from_value(unavailable).expect("the hostile descriptor remains structural");
    assert!(matches!(
        runtime.readmit_signal_branch_basis(unavailable),
        Err(SignalBranchBasisReadmissionDenial::UnavailableSnapshot { .. })
    ));

    let advanced = runtime
        .advance_signal_branch(&mut (), &initial, |_| Ok(()))
        .expect("the legacy mutation delegates to the canonical target cell");
    assert!(matches!(
        runtime.readmit_signal_branch_basis(initial.descriptor().clone()),
        Err(SignalBranchBasisReadmissionDenial::ReferenceMismatch { .. })
    ));

    let post_issuance_lease = runtime
        .retain_signal_component_basis(advanced.advanced_basis())
        .expect("post-issuance retention uses the sealed retention owner");
    assert!(matches!(
        runtime.release_signal_component_basis(post_issuance_lease),
        SignalBranchRetentionReleaseOutcome::Released(_)
    ));
    let post_issuance_terminals = runtime.signal_component_retention_terminal_counts();
    assert_eq!(
        post_issuance_terminals.explicit_releases(),
        pre_issuance_terminals.explicit_releases() + 1
    );
}

#[test]
fn sealed_legacy_retirement_preserves_unknown_before_basis_mismatch() {
    let mut runtime = runtime();
    let main = runtime.current_branch();
    let main_basis = runtime
        .observe_signal_branch_basis(main.clone())
        .expect("the bootstrap basis is admitted");
    let (child, child_basis) = runtime
        .fork_signal_branch("legacy-retirement-child", &main_basis)
        .expect("the owner creates a real retirement target")
        .into_parts();
    let child_descriptor = child_basis.descriptor().clone();
    let unknown = RuntimeBranch {
        id: RuntimeBranchId(u64::MAX),
        name: "unknown".to_owned(),
        parent_branch_id: None,
        head_snapshot_id: None,
    };
    assert!(matches!(
        runtime.plan_signal_branch_retirement(
            unknown.clone(),
            child_basis.clone(),
            SignalBranchRetirementReason::Superseded,
        ),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::UnknownBranch { .. })
    ));
    assert!(matches!(
        runtime.plan_signal_branch_retirement(
            main.clone(),
            child_basis.clone(),
            SignalBranchRetirementReason::Superseded,
        ),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::CanonicalBasisMismatch)
    ));
    assert!(matches!(
        runtime.plan_signal_branch_retirement_releasing_snapshots(
            main.clone(),
            child_basis.clone(),
            &[],
            SignalBranchRetirementReason::Superseded,
        ),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::CanonicalBasisMismatch)
    ));

    let _services = runtime
        .owner_component_services()
        .expect("the canonical partition seals once");

    assert!(matches!(
        runtime.plan_signal_branch_retirement(
            unknown,
            child_basis.clone(),
            SignalBranchRetirementReason::Superseded,
        ),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::UnknownBranch { .. })
    ));
    assert!(matches!(
        runtime.plan_signal_branch_retirement(
            main.clone(),
            child_basis.clone(),
            SignalBranchRetirementReason::Superseded,
        ),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::CanonicalBasisMismatch)
    ));
    assert!(matches!(
        runtime.plan_signal_branch_retirement_releasing_snapshots(
            main,
            child_basis.clone(),
            &[],
            SignalBranchRetirementReason::Superseded,
        ),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::CanonicalBasisMismatch)
    ));

    let plan = runtime
        .plan_signal_branch_retirement_releasing_snapshots(
            child,
            child_basis,
            &[],
            SignalBranchRetirementReason::Superseded,
        )
        .into_result()
        .expect("the exact sealed target remains retireable");
    assert!(matches!(
        runtime.retire_signal_branch(plan),
        TransitionOutcome::Success(_)
    ));
    assert!(matches!(
        runtime.readmit_signal_branch_basis(child_descriptor),
        Err(SignalBranchBasisReadmissionDenial::RetiredBranch { .. })
    ));
}

#[test]
fn sealed_legacy_batch_fences_child_before_parent_and_retains_receipts() {
    let mut runtime = runtime();
    let main = runtime.current_branch();
    let main_basis = runtime
        .observe_signal_branch_basis(main.clone())
        .expect("the bootstrap basis is admitted");
    let (parent, parent_basis) = runtime
        .fork_signal_branch("batch-parent", &main_basis)
        .expect("the owner creates the batch parent")
        .into_parts();
    let (child, child_basis) = runtime
        .fork_signal_branch("batch-child", &parent_basis)
        .expect("the owner creates the batch child")
        .into_parts();
    let unknown = RuntimeBranch {
        id: RuntimeBranchId(u64::MAX),
        name: "batch-unknown".to_owned(),
        parent_branch_id: None,
        head_snapshot_id: None,
    };

    assert!(matches!(
        runtime.plan_signal_branch_retirement_batch(Vec::new()),
        TransitionOutcome::Denied(SignalBranchRetirementBatchDenial::Empty)
    ));
    assert!(matches!(
        runtime.plan_signal_branch_retirement_batch_releasing_snapshots(Vec::new()),
        TransitionOutcome::Denied(SignalBranchRetirementBatchDenial::Empty)
    ));
    assert!(matches!(
        runtime.plan_signal_branch_retirement_batch(vec![(
            unknown.clone(),
            child_basis.clone(),
            SignalBranchRetirementReason::Superseded,
        )]),
        TransitionOutcome::Denied(SignalBranchRetirementBatchDenial::Retirement {
            position: 0,
            denial: SignalBranchRetirementDenial::UnknownBranch { .. },
        })
    ));
    assert!(matches!(
        runtime.plan_signal_branch_retirement_batch(vec![(
            main.clone(),
            child_basis.clone(),
            SignalBranchRetirementReason::Superseded,
        )]),
        TransitionOutcome::Denied(SignalBranchRetirementBatchDenial::Retirement {
            position: 0,
            denial: SignalBranchRetirementDenial::CanonicalBasisMismatch,
        })
    ));
    assert!(matches!(
        runtime.plan_signal_branch_retirement_batch(vec![(
            parent.clone(),
            parent_basis.clone(),
            SignalBranchRetirementReason::Superseded,
        )]),
        TransitionOutcome::Denied(SignalBranchRetirementBatchDenial::Retirement {
            position: 0,
            denial: SignalBranchRetirementDenial::LiveChildren { .. },
        })
    ));

    let _services = runtime
        .owner_component_services()
        .expect("the canonical partition seals once");

    assert!(matches!(
        runtime.plan_signal_branch_retirement_batch(Vec::new()),
        TransitionOutcome::Denied(SignalBranchRetirementBatchDenial::Empty)
    ));
    assert!(matches!(
        runtime.plan_signal_branch_retirement_batch_releasing_snapshots(Vec::new()),
        TransitionOutcome::Denied(SignalBranchRetirementBatchDenial::Empty)
    ));
    assert!(matches!(
        runtime.plan_signal_branch_retirement_batch(vec![(
            unknown,
            child_basis.clone(),
            SignalBranchRetirementReason::Superseded,
        )]),
        TransitionOutcome::Denied(SignalBranchRetirementBatchDenial::Retirement {
            position: 0,
            denial: SignalBranchRetirementDenial::UnknownBranch { .. },
        })
    ));
    assert!(matches!(
        runtime.plan_signal_branch_retirement_batch(vec![(
            main,
            child_basis.clone(),
            SignalBranchRetirementReason::Superseded,
        )]),
        TransitionOutcome::Denied(SignalBranchRetirementBatchDenial::Retirement {
            position: 0,
            denial: SignalBranchRetirementDenial::CanonicalBasisMismatch,
        })
    ));
    assert!(matches!(
        runtime.plan_signal_branch_retirement_batch(vec![(
            parent.clone(),
            parent_basis.clone(),
            SignalBranchRetirementReason::Superseded,
        )]),
        TransitionOutcome::Denied(SignalBranchRetirementBatchDenial::Retirement {
            position: 0,
            denial: SignalBranchRetirementDenial::LiveChildren { .. },
        })
    ));

    let plan = runtime
        .plan_signal_branch_retirement_batch(vec![
            (
                child.clone(),
                child_basis,
                SignalBranchRetirementReason::Merged,
            ),
            (
                parent.clone(),
                parent_basis,
                SignalBranchRetirementReason::DependencyCancellation,
            ),
        ])
        .into_result()
        .expect("the sealed owner plans the complete child-before-parent batch");
    let receipt = runtime
        .retire_signal_branch_batch(plan)
        .into_result()
        .expect("the sealed owner executes the fully fenced batch");
    assert_eq!(receipt.receipts()[0].retired_branch().id, child.id);
    assert_eq!(receipt.receipts()[1].retired_branch().id, parent.id);
    assert_eq!(
        runtime
            .branch_retirement_receipt(child.id)
            .expect("the canonical owner retains the child receipt")
            .retired_branch()
            .id,
        child.id
    );
    assert_eq!(
        runtime
            .branch_retirement_receipt(parent.id)
            .expect("the canonical owner retains the parent receipt")
            .retired_branch()
            .id,
        parent.id
    );
}
