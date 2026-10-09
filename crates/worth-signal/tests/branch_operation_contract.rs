use worth_foundational::FoundationalBranchReferenceMismatchAxis;
use worth_signal::facade::branch::{
    SignalBranchForkOperationDenial, SignalBranchMergeDenial, SignalBranchRestoreDenial,
    SignalBranchSnapshotCaptureDenial,
};
use worth_signal::facade::{
    Aspect, AspectVersion, DependencyEdge, NodeEvaluationResult, SignalGraph, SignalRuntime,
};

fn runtime() -> SignalRuntime<(), (), (), (), ()> {
    SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build()
}

#[test]
fn documented_advance_then_read_flow_compiles_through_the_public_facade() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            worth_signal::facade::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    const PRICE: Aspect = Aspect::new(0);
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let derived = graph.node().on_demand().build();
    graph
        .set_dependencies(derived, [DependencyEdge::new(source, PRICE)])
        .unwrap();
    let mut runtime = SignalRuntime::build_for::<()>(graph);
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();

    let _next_basis = runtime
        .advance_signal_branch(request_execution, &mut (), &basis, |tx| {
            tx.mark_changed(source, PRICE)?;
            tx.target(derived).run(&|view| {
                let result = if view.node() == source {
                    view.finish(NodeEvaluationResult::from_version(
                        AspectVersion::from_updates([(PRICE, 1)]),
                    ))
                } else {
                    let version = view.read_aspect_version(source, PRICE)?;
                    view.finish(NodeEvaluationResult::from_version(version))
                };
                Ok(result)
            })?;
            Ok(())
        })
        .unwrap()
        .into_basis();

    let read_result = runtime
        .target(derived)
        .read(&(), &|view| {
            let version = view.read_aspect_version(source, PRICE)?;
            Ok(view.finish(NodeEvaluationResult::from_version(version)))
        })
        .unwrap();
    assert_eq!(read_result, AspectVersion::from_updates([(PRICE, 1)]));
}

#[test]
fn invalid_fork_identity_is_denied_before_catalog_or_head_movement() {
    let mut runtime = runtime();
    let branch = runtime.current_branch();
    let basis = runtime
        .observe_signal_branch_basis(branch.clone())
        .expect("owner observation should succeed");
    let known_before = runtime.known_branches();

    assert!(matches!(
        runtime.fork_signal_branch("   ", &basis),
        Err(SignalBranchForkOperationDenial::InvalidIdentity { .. })
    ));
    assert_eq!(runtime.known_branches(), known_before);
    let after = runtime
        .observe_signal_branch_basis(branch)
        .expect("invalid fork must preserve the source head");
    assert_eq!(after.observation(), basis.observation());
}

#[test]
fn cross_branch_snapshot_restore_is_typed_and_moves_neither_branch() {
    let mut runtime = runtime();
    let main = runtime.current_branch();
    let main_basis = runtime
        .observe_signal_branch_basis(main.clone())
        .expect("owner observation should succeed");
    let (snapshot, captured_main) = runtime
        .capture_signal_branch_snapshot(&main_basis)
        .expect("snapshot should succeed through the owner basis")
        .into_parts();
    drop(main_basis);

    let (fork, fork_basis) = runtime
        .fork_signal_branch("cross-restore", &captured_main)
        .expect("owner fork should succeed")
        .into_parts();
    assert!(matches!(
        runtime.restore_signal_branch(&fork_basis, &snapshot),
        Err(SignalBranchRestoreDenial::CrossBranchSnapshot {
            branch_id,
            snapshot_branch_id,
        }) if branch_id == fork.id && snapshot_branch_id == snapshot.snapshot().meta.branch_id
    ));
    let after_denial = runtime
        .observe_signal_branch_basis(fork)
        .expect("denied restore must preserve the fork");
    assert_eq!(after_denial.observation(), fork_basis.observation());
    let main_after_denial = runtime
        .observe_signal_branch_basis(main)
        .expect("cross-branch denial must preserve the source branch");
    assert_eq!(main_after_denial.observation(), captured_main.observation());
}

#[test]
fn stale_restore_is_typed_and_preserves_the_current_generation() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            worth_signal::facade::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = runtime();
    let branch = runtime.current_branch();
    let initial = runtime
        .observe_signal_branch_basis(branch.clone())
        .expect("owner observation should succeed");
    let (snapshot, captured) = runtime
        .capture_signal_branch_snapshot(&initial)
        .expect("snapshot should succeed through the owner basis")
        .into_parts();
    let current = runtime
        .advance_signal_branch(request_execution, &mut (), &captured, |_| Ok(()))
        .expect("advance should move the branch")
        .into_basis();

    assert!(matches!(
        runtime.restore_signal_branch(&captured, &snapshot),
        Err(SignalBranchRestoreDenial::BasisMismatch { ref axes })
            if axes == &[FoundationalBranchReferenceMismatchAxis::ReferenceGeneration]
    ));
    let after_denial = runtime
        .observe_signal_branch_basis(branch)
        .expect("denied restore must preserve the branch");
    assert_eq!(after_denial.observation(), current.observation());
}

#[test]
fn stale_snapshot_capture_is_typed_and_preserves_the_current_generation() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            worth_signal::facade::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = runtime();
    let branch = runtime.current_branch();
    let stale = runtime
        .observe_signal_branch_basis(branch.clone())
        .expect("owner observation should succeed");
    let current = runtime
        .advance_signal_branch(request_execution, &mut (), &stale, |_| Ok(()))
        .expect("advance should move the branch")
        .into_basis();

    assert!(matches!(
        runtime.capture_signal_branch_snapshot(&stale),
        Err(SignalBranchSnapshotCaptureDenial::BasisMismatch { ref axes })
            if axes == &[FoundationalBranchReferenceMismatchAxis::ReferenceGeneration]
    ));
    let after_denial = runtime
        .observe_signal_branch_basis(branch)
        .expect("denied capture must preserve the branch");
    assert_eq!(after_denial.observation(), current.observation());
}

#[test]
fn snapshot_authority_from_another_runtime_cannot_restore_matching_branch_ids() {
    let mut source = runtime();
    let source_basis = source
        .observe_signal_branch_basis(source.current_branch())
        .expect("source owner observation should succeed");
    let (foreign_snapshot, _) = source
        .capture_signal_branch_snapshot(&source_basis)
        .expect("source capture should succeed")
        .into_parts();

    let mut target = runtime();
    let target_branch = target.current_branch();
    let target_basis = target
        .observe_signal_branch_basis(target_branch.clone())
        .expect("target owner observation should succeed");
    assert!(matches!(
        target.restore_signal_branch(&target_basis, &foreign_snapshot),
        Err(SignalBranchRestoreDenial::ForeignSnapshotOwner { .. })
    ));
    let after = target
        .observe_signal_branch_basis(target_branch)
        .expect("foreign snapshot denial must preserve the target");
    assert_eq!(after.observation(), target_basis.observation());
}

#[test]
fn merge_requires_exact_bases_and_issues_the_advanced_target_basis() {
    let mut runtime = runtime();
    let main = runtime.current_branch();
    let main_basis = runtime
        .observe_signal_branch_basis(main.clone())
        .expect("main owner observation should succeed");
    let (_, feature_basis) = runtime
        .fork_signal_branch("canonical-merge", &main_basis)
        .expect("canonical fork should succeed")
        .into_parts();

    let outcome = runtime
        .merge()
        .from(&feature_basis)
        .into(&main_basis)
        .run()
        .expect("exact source and target bases should admit the merge");
    let target_basis = outcome.target_basis().clone();
    assert_ne!(target_basis.observation(), main_basis.observation());
    assert_eq!(
        runtime
            .observe_signal_branch_basis(main.clone())
            .expect("merged target should remain observable")
            .observation(),
        target_basis.observation()
    );

    let stale_denial = runtime
        .merge_branch(&feature_basis, &main_basis)
        .expect_err("the old target basis must be rejected");
    assert!(
        matches!(
        &stale_denial,
        SignalBranchMergeDenial::TargetBasisMismatch { axes }
            if axes.contains(&FoundationalBranchReferenceMismatchAxis::ReferenceGeneration)
        ),
        "unexpected stale merge denial: {stale_denial:?}"
    );
    assert_eq!(
        runtime
            .observe_signal_branch_basis(main)
            .expect("denied stale merge must preserve the target")
            .observation(),
        target_basis.observation()
    );
}
