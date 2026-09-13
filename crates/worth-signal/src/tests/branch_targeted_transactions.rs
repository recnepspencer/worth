use worth_proof::TransitionOutcome;

use crate::facade::*;
use crate::logic::transaction::LoweredBranchTargetedTransactionPlan;

fn targeted_plan<E>(
    runtime: &mut SignalRuntime<(), (), E, (), ()>,
    branch: SignalBranchHandle,
) -> LoweredBranchTargetedTransactionPlan {
    let head = match runtime.branch_transaction_head(branch.clone()) {
        TransitionOutcome::Success(head) => head,
        other => panic!("expected branch head, got {other:?}"),
    };
    match runtime
        .plan_branch_targeted_transaction(BranchTargetedTransactionRequest::new(branch, head))
    {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("expected branch transaction plan, got {other:?}"),
    }
}

#[test]
fn ten_interleaved_branch_transactions_advance_only_their_owned_heads() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let canonical = runtime.current_branch();
    let branches = (0..10)
        .map(|ordinal| {
            runtime
                .create_branch(format!("effect-{ordinal}"))
                .expect("branch fork should succeed")
        })
        .collect::<Vec<_>>();

    for (ordinal, branch) in branches.iter().enumerate() {
        let plan = targeted_plan(&mut runtime, branch.clone());
        let receipt =
            match runtime.execute_branch_targeted_transaction(&mut (), plan, |transaction| {
                transaction.mark_dirty(
                    node,
                    Aspect::new((ordinal % crate::data::aspect::MAX_ASPECTS) as u8),
                )
            }) {
                TransitionOutcome::Success(receipt) => receipt,
                other => panic!("expected targeted transaction success, got {other:?}"),
            };
        assert_eq!(receipt.before_head().generation(), 0);
        assert_eq!(receipt.after_head().generation(), 1);
        assert_eq!(receipt.active_branch_before(), &canonical);
        assert_eq!(receipt.active_branch_after(), &canonical);
        for untouched in branches.iter().skip(ordinal + 1) {
            let head = match runtime.branch_transaction_head(untouched.clone()) {
                TransitionOutcome::Success(head) => head,
                other => panic!("expected untouched branch head, got {other:?}"),
            };
            assert_eq!(head.generation(), 0);
        }
    }

    assert_eq!(runtime.current_branch(), canonical);
    assert!(branches.iter().all(|branch| {
        matches!(
            runtime.branch_transaction_head(branch.clone()),
            TransitionOutcome::Success(head) if head.generation() == 1
        )
    }));
}

#[test]
fn stale_target_head_is_denied_without_running_the_transaction_closure() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let canonical = runtime.current_branch();
    let branch = runtime.create_branch("stale-effect").unwrap();
    let stale_plan = targeted_plan(&mut runtime, branch.clone());
    let current_plan = targeted_plan(&mut runtime, branch.clone());
    assert!(matches!(
        runtime.execute_branch_targeted_transaction(&mut (), current_plan, |transaction| {
            transaction.mark_dirty(node, Aspect::new(0))
        }),
        TransitionOutcome::Success(_)
    ));
    let mut closure_ran = false;
    let stale = runtime.execute_branch_targeted_transaction(&mut (), stale_plan, |_transaction| {
        closure_ran = true;
        Ok(())
    });
    assert!(matches!(
        stale,
        TransitionOutcome::Denied(BranchTargetedTransactionDenial::StaleTargetHead { .. })
    ));
    assert!(!closure_ran);
    assert_eq!(runtime.current_branch(), canonical);
}

#[test]
fn branch_local_event_publication_is_rejected_and_active_branch_is_restored() {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestEvent {
        Published,
    }

    let graph = SignalGraph::new();
    let mut runtime = SignalRuntime::builder(graph)
        .with_kernel_defaults()
        .with_events::<TestEvent>()
        .build();
    let canonical = runtime.current_branch();
    let branch = runtime.create_branch("event-effect").unwrap();
    let plan = targeted_plan(&mut runtime, branch.clone());
    let outcome = runtime.execute_branch_targeted_transaction(&mut (), plan, |transaction| {
        transaction.emit_event(TestEvent::Published);
        Ok(())
    });

    assert!(matches!(outcome, TransitionOutcome::Failed(_)));
    assert_eq!(runtime.current_branch(), canonical);
    let head = match runtime.branch_transaction_head(branch) {
        TransitionOutcome::Success(head) => head,
        other => panic!("expected live branch after denied publication, got {other:?}"),
    };
    assert_eq!(head.generation(), 0);
}

#[test]
fn branch_targeted_dependency_rewiring_is_atomic_and_branch_local() {
    let mut graph = SignalGraph::new();
    let source_a = graph.node().build();
    let source_b = graph.node().build();
    let derived = graph.node().build();
    graph
        .set_dependencies(derived, [DependencyEdge::new(source_a, Aspect::new(0))])
        .unwrap();
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let canonical = runtime.current_branch();
    let branch = runtime.create_branch("dynamic-dependencies").unwrap();

    let failed_plan = targeted_plan(&mut runtime, branch.clone());
    let failed = runtime.execute_branch_targeted_transaction(&mut (), failed_plan, |transaction| {
        transaction.set_dependencies(derived, [DependencyEdge::new(source_b, Aspect::new(0))])?;
        Err(SignalError::invalid_input("force rollback"))
    });
    assert!(matches!(failed, TransitionOutcome::Failed(_)));
    runtime.switch_branch(branch.clone()).unwrap();
    assert_eq!(
        runtime.graph().dependency_sources_of(derived).unwrap(),
        vec![source_a],
        "aborted destination must restore its inherited dependency truth"
    );
    runtime.switch_branch(canonical.clone()).unwrap();

    let committed_plan = targeted_plan(&mut runtime, branch.clone());
    assert!(matches!(
        runtime.execute_branch_targeted_transaction(&mut (), committed_plan, |transaction| {
            transaction.set_dependencies(derived, [DependencyEdge::new(source_b, Aspect::new(0))])
        },),
        TransitionOutcome::Success(_)
    ));
    assert_eq!(runtime.current_branch(), canonical);
    assert_eq!(
        runtime.graph().dependency_sources_of(derived).unwrap(),
        vec![source_a]
    );

    runtime.switch_branch(branch).unwrap();
    assert_eq!(
        runtime.graph().dependency_sources_of(derived).unwrap(),
        vec![source_b]
    );
}
