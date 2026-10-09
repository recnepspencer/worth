use worth_proof::TransitionOutcome;

use crate::facade::*;

fn fork(
    runtime: &mut SignalRuntime<(), (), (), (), ()>,
    name: impl Into<String>,
    parent: SignalBranchId,
) -> SignalBranchHandle {
    match runtime.fork_branch(SignalBranchForkRequest::from_parent_branch_head(
        name, parent,
    )) {
        TransitionOutcome::Success(receipt) => receipt.created_branch().clone(),
        other => panic!("expected branch fork, got {other:?}"),
    }
}

fn retirement_plan(
    runtime: &mut SignalRuntime<(), (), (), (), ()>,
    branch: SignalBranchHandle,
    reason: SignalBranchRetirementReason,
) -> PlannedSignalBranchRetirement {
    let basis = runtime
        .observe_signal_branch_basis(branch.clone())
        .expect("retirement branch should be observable");
    match runtime.plan_signal_branch_retirement(branch, basis, reason) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("expected retirement plan, got {other:?}"),
    }
}

#[test]
fn retirement_reclaims_heavy_state_and_retains_compact_closeout_proof() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let canonical = runtime.current_branch();
    let branch = fork(&mut runtime, "retire-with-snapshots", canonical.id);
    runtime.switch_branch(branch.clone()).unwrap();
    runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    runtime.switch_branch(canonical.clone()).unwrap();

    let plan = retirement_plan(
        &mut runtime,
        branch.clone(),
        SignalBranchRetirementReason::Rejected,
    );
    let receipt = match runtime.retire_branch(plan) {
        TransitionOutcome::Success(receipt) => receipt,
        other => panic!("expected retirement success, got {other:?}"),
    };

    assert_eq!(receipt.retired_branch().id, branch.id);
    assert_eq!(receipt.retired_branch().name, branch.name);
    assert_eq!(
        receipt.retired_branch().head_snapshot_id,
        receipt.terminal_head_snapshot_id()
    );
    assert_eq!(receipt.parent_branch_id(), canonical.id);
    assert_eq!(receipt.reclaimed_branch_state_count(), 1);
    assert_eq!(receipt.reclaimed_snapshot_state_count(), 2);
    assert_eq!(receipt.reclaimed_runtime_meta_count(), 1);
    assert_eq!(receipt.retained_proof_record_count(), 1);
    assert!(!receipt.closeout_digest().is_empty());
    assert!(runtime.branch_handle(branch.id).is_none());
    assert_eq!(runtime.known_branches(), vec![canonical]);
    assert_eq!(
        runtime
            .branch_retirement_receipt(branch.id)
            .expect("retirement proof must remain readable")
            .closeout_digest(),
        receipt.closeout_digest()
    );
    assert!(runtime
        .replay_for_branch(runtime.current_branch().id)
        .frames
        .iter()
        .any(|event| event.kind == ReplayEventKind::BranchRetired));
}

#[test]
fn retirement_denies_current_and_parent_branches_with_live_native_children() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let canonical = runtime.current_branch();
    let canonical_basis = runtime
        .observe_signal_branch_basis(canonical.clone())
        .expect("canonical branch should be observable");
    let current_denial = runtime.plan_signal_branch_retirement(
        canonical.clone(),
        canonical_basis,
        SignalBranchRetirementReason::Superseded,
    );
    assert!(matches!(
        current_denial,
        TransitionOutcome::Denied(SignalBranchRetirementDenial::CurrentBranch { .. })
    ));

    let parent = fork(&mut runtime, "parent", canonical.id);
    let child = fork(&mut runtime, "child", parent.id);
    let parent_basis = runtime
        .observe_signal_branch_basis(parent.clone())
        .expect("parent branch should be observable");
    let denial = runtime.plan_signal_branch_retirement(
        parent.clone(),
        parent_basis,
        SignalBranchRetirementReason::DependencyCancellation,
    );
    assert!(matches!(
        denial,
        TransitionOutcome::Denied(SignalBranchRetirementDenial::LiveChildren {
            child_branch_ids,
            ..
        }) if child_branch_ids == vec![child.id]
    ));
    let child_plan = retirement_plan(
        &mut runtime,
        child,
        SignalBranchRetirementReason::DependencyCancellation,
    );
    assert!(matches!(
        runtime.retire_branch(child_plan),
        TransitionOutcome::Success(_)
    ));
    let parent_plan = retirement_plan(
        &mut runtime,
        parent,
        SignalBranchRetirementReason::DependencyCancellation,
    );
    assert!(matches!(
        runtime.retire_branch(parent_plan),
        TransitionOutcome::Success(_)
    ));
}

#[test]
fn one_thousand_retired_siblings_leave_no_live_branch_residue() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let canonical = runtime.current_branch();
    let siblings = (0..1_000)
        .map(|ordinal| fork(&mut runtime, format!("sibling-{ordinal}"), canonical.id))
        .collect::<Vec<_>>();

    for sibling in &siblings {
        let plan = retirement_plan(
            &mut runtime,
            sibling.clone(),
            SignalBranchRetirementReason::ProjectionRebuild,
        );
        assert!(matches!(
            runtime.retire_branch(plan),
            TransitionOutcome::Success(_)
        ));
    }

    assert_eq!(runtime.known_branches(), vec![canonical]);
    assert!(siblings
        .iter()
        .all(|branch| runtime.branch_handle(branch.id).is_none()));
    assert!(siblings
        .iter()
        .all(|branch| runtime.branch_retirement_receipt(branch.id).is_some()));
    assert_eq!(
        runtime
            .telemetry()
            .transaction
            .branch_retirement_execution_count,
        1_000
    );
}

#[test]
fn retirement_plan_is_invalidated_by_a_snapshot_free_branch_transaction() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let canonical = runtime.current_branch();
    let branch = fork(&mut runtime, "moving-head", canonical.id);
    let retirement = retirement_plan(
        &mut runtime,
        branch.clone(),
        SignalBranchRetirementReason::Superseded,
    );
    let head = match runtime.branch_transaction_head(branch.clone()) {
        TransitionOutcome::Success(head) => head,
        other => panic!("expected branch head, got {other:?}"),
    };
    let transaction = match runtime
        .plan_branch_targeted_transaction(BranchTargetedTransactionRequest::new(branch, head))
    {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("expected targeted transaction plan, got {other:?}"),
    };
    assert!(matches!(
        runtime.execute_branch_targeted_transaction(
            request_execution,
            &mut (),
            transaction,
            |tx| { tx.mark_dirty(node, Aspect::new(0)) }
        ),
        TransitionOutcome::Success(_)
    ));

    assert!(matches!(
        runtime.retire_branch(retirement),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::StaleBranchHead { .. })
    ));
}

#[test]
fn ordered_retirement_batch_closes_child_then_parent_as_one_plan() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let canonical = runtime.current_branch();
    let parent = fork(&mut runtime, "derived-basis", canonical.id);
    let child = fork(&mut runtime, "effect", parent.id);
    let child_basis = runtime
        .observe_signal_branch_basis(child.clone())
        .expect("child should be observable");
    let parent_basis = runtime
        .observe_signal_branch_basis(parent.clone())
        .expect("parent should be observable");
    let plan = match runtime.plan_signal_branch_retirement_batch(vec![
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
    ]) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("expected ordered retirement plan, got {other:?}"),
    };
    assert_eq!(plan.breadth(), 2);
    let receipt = match runtime.retire_branch_batch(plan) {
        TransitionOutcome::Success(receipt) => receipt,
        other => panic!("expected ordered retirement, got {other:?}"),
    };
    assert_eq!(receipt.receipts().len(), 2);
    assert_eq!(receipt.receipts()[0].retired_branch().id, child.id);
    assert_eq!(receipt.receipts()[1].retired_branch().id, parent.id);
    assert_eq!(runtime.known_branches(), vec![canonical]);
}

#[test]
fn retirement_batch_denial_is_side_effect_free() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let canonical = runtime.current_branch();
    let parent = fork(&mut runtime, "parent-first", canonical.id);
    let child = fork(&mut runtime, "child-second", parent.id);
    let parent_basis = runtime
        .observe_signal_branch_basis(parent.clone())
        .expect("parent should be observable");
    let child_basis = runtime
        .observe_signal_branch_basis(child.clone())
        .expect("child should be observable");
    assert!(matches!(
        runtime.plan_signal_branch_retirement_batch(vec![
            (
                parent.clone(),
                parent_basis,
                SignalBranchRetirementReason::DependencyCancellation,
            ),
            (
                child.clone(),
                child_basis,
                SignalBranchRetirementReason::Rejected
            ),
        ]),
        TransitionOutcome::Denied(SignalBranchRetirementBatchDenial::Retirement {
            denial: SignalBranchRetirementDenial::LiveChildren { .. },
            ..
        })
    ));
    assert!(runtime.branch_handle(parent.id).is_some());
    assert!(runtime.branch_handle(child.id).is_some());
    assert!(runtime.branch_retirement_receipt(parent.id).is_none());
    assert!(runtime.branch_retirement_receipt(child.id).is_none());
}
