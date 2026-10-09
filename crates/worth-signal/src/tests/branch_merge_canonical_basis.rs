use worth_foundational::facade::CanonicalizationRuleVersion;
use worth_proof::TransitionOutcome;

use crate::facade::*;
use crate::tests::support::{version_ab, ASPECT_A, ASPECT_B};

fn canonical_version() -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new("worth.signal.phase8.tests")
        .expect("phase 8 test canonicalization version should be valid")
}

fn build_scoped_canonical_runtime() -> (
    SignalRuntime<(), (), (), (), ()>,
    SignalBranchHandle,
    SignalBranchHandle,
    NodeId,
    NodeId,
) {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new();
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let support = runtime
        .graph_mut()
        .node()
        .reads_aspects([ASPECT_A])
        .produces_aspects([ASPECT_A])
        .build();
    let primary = runtime
        .graph_mut()
        .node()
        .reads_aspects([ASPECT_A])
        .produces_aspects([ASPECT_A])
        .build();
    runtime
        .graph_mut()
        .append_dependency(primary, support, ASPECT_A)
        .unwrap();
    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.read(support, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0))))
            })?;
            tx.read(primary, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(2, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.current_branch();
    let feature = runtime.create_branch("feature-phase8-canonical").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.mark_dirty(primary, ASPECT_A)?;
            tx.read(primary, &|view| {
                let _ = view.read_aspect_version(support, ASPECT_A)?;
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(22, 0))))
            })?;
            Ok(())
        })
        .unwrap();
    runtime.switch_branch(main.clone()).unwrap();

    (runtime, feature, main, support, primary)
}

#[test]
fn branch_basis_canonical_projection_is_stable_across_equivalent_producers() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new();
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let node = runtime
        .graph_mut()
        .node()
        .produces_aspects([ASPECT_A])
        .build();
    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.read(node, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0))))
            })?;
            Ok(())
        })
        .unwrap();
    let main = runtime.current_branch();
    let snapshot = runtime.capture_branch_snapshot(main.clone()).unwrap();
    let current = runtime.current_branch_basis_artifact();
    let explicit = match runtime.branch_basis_artifact(main.clone()) {
        TransitionOutcome::Success(artifact) => artifact,
        outcome => panic!("expected explicit branch basis artifact, got {outcome:?}"),
    };
    let snapshot_basis = match runtime.snapshot_branch_basis_artifact(main, &snapshot) {
        TransitionOutcome::Success(artifact) => artifact,
        outcome => panic!("expected snapshot-derived branch basis artifact, got {outcome:?}"),
    };

    let current_canonical = match current
        .payload()
        .prepare_canonical_basis(canonical_version())
    {
        TransitionOutcome::Success(ready) => ready,
        outcome => panic!("expected current branch basis canonicalization, got {outcome:?}"),
    };
    let explicit_canonical = match explicit
        .payload()
        .prepare_canonical_basis(canonical_version())
    {
        TransitionOutcome::Success(ready) => ready,
        outcome => panic!("expected explicit branch basis canonicalization, got {outcome:?}"),
    };
    let snapshot_canonical = match snapshot_basis
        .payload()
        .prepare_canonical_basis(canonical_version())
    {
        TransitionOutcome::Success(ready) => ready,
        outcome => panic!("expected snapshot branch basis canonicalization, got {outcome:?}"),
    };

    assert_eq!(
        current_canonical.payload().entries(),
        explicit_canonical.payload().entries()
    );
    assert_eq!(
        current_canonical.payload().entries(),
        snapshot_canonical.payload().entries()
    );

    let current_locator = match current
        .payload()
        .prepare_locator_for_canonical_basis(canonical_version())
    {
        TransitionOutcome::Success(ready) => ready,
        outcome => {
            panic!("expected current branch basis locator canonicalization, got {outcome:?}")
        }
    };
    let explicit_locator = match explicit
        .payload()
        .prepare_locator_for_canonical_basis(canonical_version())
    {
        TransitionOutcome::Success(ready) => ready,
        outcome => {
            panic!("expected explicit branch basis locator canonicalization, got {outcome:?}")
        }
    };
    assert_eq!(
        current_locator.payload().entries(),
        explicit_locator.payload().entries()
    );
    assert_eq!(
        current.payload().compact_explanation(),
        explicit.payload().compact_explanation()
    );
}

#[test]
fn scoped_merge_canonical_basis_keeps_requested_admitted_and_no_op_scope_separate() {
    let (mut runtime, feature, main, _support, primary) = build_scoped_canonical_runtime();
    let request_a = [
        SignalSelectedAspectRequestEntry::new(primary, ASPECT_B),
        SignalSelectedAspectRequestEntry::new(primary, ASPECT_A),
    ];
    let request_b = [
        SignalSelectedAspectRequestEntry::new(primary, ASPECT_A),
        SignalSelectedAspectRequestEntry::new(primary, ASPECT_B),
    ];

    let canonical_a = match runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .selected_aspects(request_a)
        .plan()
        .expect("selected-aspect merge should plan")
        .plan()
        .prepare_scoped_merge_canonical_basis_bundle(canonical_version())
    {
        TransitionOutcome::Success(ready) => ready,
        outcome => panic!("expected scoped canonical basis bundle, got {outcome:?}"),
    };
    let canonical_b = match runtime
        .merge_raw()
        .from(feature)
        .into(main)
        .selected_aspects(request_b)
        .plan()
        .expect("selected-aspect merge should plan regardless of producer order")
        .plan()
        .prepare_scoped_merge_canonical_basis_bundle(canonical_version())
    {
        TransitionOutcome::Success(ready) => ready,
        outcome => panic!("expected scoped canonical basis bundle, got {outcome:?}"),
    };

    assert_eq!(
        canonical_a.declaration().payload().entries(),
        canonical_b.declaration().payload().entries()
    );
    assert_eq!(
        canonical_a.admitted().payload().entries(),
        canonical_b.admitted().payload().entries()
    );
    assert_eq!(
        canonical_a.no_op().unwrap().payload().entries(),
        canonical_b.no_op().unwrap().payload().entries()
    );
    assert_ne!(
        canonical_a.declaration().payload().entries(),
        canonical_a.admitted().payload().entries()
    );
    assert_ne!(
        canonical_a.admitted().payload().entries(),
        canonical_a.no_op().unwrap().payload().entries()
    );
    assert!(canonical_a.skipped().is_none());
}

#[test]
fn canonical_projection_keeps_selected_node_and_selected_aspect_families_distinct() {
    let (mut runtime, feature, main, _support, primary) = build_scoped_canonical_runtime();

    let selected_node_bundle = match runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .selected_nodes([primary])
        .plan()
        .expect("selected-node merge should plan")
        .plan()
        .prepare_scoped_merge_canonical_basis_bundle(canonical_version())
    {
        TransitionOutcome::Success(ready) => ready,
        outcome => panic!("expected selected-node canonical basis bundle, got {outcome:?}"),
    };
    let selected_aspect_bundle = match runtime
        .merge_raw()
        .from(feature)
        .into(main)
        .selected_aspects([SignalSelectedAspectRequestEntry::new(primary, ASPECT_A)])
        .plan()
        .expect("selected-aspect merge should plan")
        .plan()
        .prepare_scoped_merge_canonical_basis_bundle(canonical_version())
    {
        TransitionOutcome::Success(ready) => ready,
        outcome => panic!("expected selected-aspect canonical basis bundle, got {outcome:?}"),
    };

    assert_ne!(
        selected_node_bundle.declaration().payload().entries(),
        selected_aspect_bundle.declaration().payload().entries(),
        "family-distinct scoped requests must not collapse to the same canonical declaration basis"
    );
    assert_ne!(
        selected_node_bundle.admitted().payload().entries(),
        selected_aspect_bundle.admitted().payload().entries(),
        "family-distinct scoped requests must not collapse to the same canonical admitted basis"
    );
}
