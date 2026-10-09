use worth_proof::TransitionOutcome;

use crate::diagnostics::replay::ReplayEventDetail;
use crate::facade::*;
use crate::tests::support::{version_ab, ASPECT_A};

fn build_strategy_identity_runtime() -> (
    SignalRuntime<(), (), (), (), ()>,
    SignalBranchHandle,
    SignalBranchHandle,
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
    let node = runtime
        .graph_mut()
        .node()
        .reads_aspects([ASPECT_A])
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
    let feature = runtime.create_branch("feature-phase9-strategy").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.mark_dirty(node, ASPECT_A)?;
            tx.read(node, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(9, 0))))
            })?;
            Ok(())
        })
        .unwrap();
    runtime.switch_branch(main.clone()).unwrap();

    (runtime, feature, main, node)
}

#[test]
fn merge_strategy_witness_is_equivalent_across_ordinary_and_compatibility_lanes() {
    let (mut ordinary_runtime, feature, main, _node) = build_strategy_identity_runtime();
    let ordinary = ordinary_runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .run()
        .expect("ordinary merge lane should succeed");

    let (mut compatibility_runtime, compatibility_feature, compatibility_main, _node) =
        build_strategy_identity_runtime();
    let compatibility = compatibility_runtime
        .merge_branch_raw(compatibility_feature.clone(), compatibility_main.clone())
        .expect("compatibility merge lane should succeed");

    assert_eq!(ordinary.strategy_witness, compatibility.strategy_witness);
    assert_eq!(
        ordinary.strategy_witness.witness_digest(),
        compatibility.strategy_witness.witness_digest()
    );
    assert_eq!(
        ordinary.strategy_witness.merge_strategy_digest(),
        compatibility.strategy_witness.merge_strategy_digest()
    );
    assert_eq!(
        ordinary.strategy_witness.invalidation_strategy_digest(),
        compatibility
            .strategy_witness
            .invalidation_strategy_digest()
    );
    assert_eq!(
        ordinary.strategy_witness.delivery_strategy_digest(),
        compatibility.strategy_witness.delivery_strategy_digest()
    );
}

#[test]
fn strategy_witness_is_retained_across_plan_result_and_replay_surfaces() {
    let (mut runtime, feature, main, _node) = build_strategy_identity_runtime();
    let planned = runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .plan()
        .expect("strategy witness planning should succeed");
    let planned_witness = planned.plan().strategy_witness().clone();
    let plan_report =
        merge_plan_proof_report(planned.plan(), planned.plan().registry_bundle_digest());
    drop(planned);

    let result = runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .run()
        .expect("strategy witness execution should succeed");
    let result_report = merge_result_proof_report(&result);
    let retained_replay_witness = runtime
        .replay_for_branch(main.id)
        .frames
        .iter()
        .rev()
        .find_map(|frame| {
            frame
                .detail
                .as_ref()
                .and_then(ReplayEventDetail::as_strategy_witness)
        })
        .cloned()
        .expect("retained replay history should carry the strategy witness");

    assert_eq!(&planned_witness, &plan_report.strategy_witness);
    assert_eq!(result.strategy_witness, result_report.strategy_witness);
    assert_eq!(&planned_witness, &result.strategy_witness);
    assert_eq!(retained_replay_witness, result.strategy_witness);
}

#[test]
fn synthetic_or_incomplete_strategy_witness_is_denied() {
    let denied = SignalMergeStrategyWitness::try_from_identities(None, None, None);
    match denied {
        TransitionOutcome::Denied(denial) => {
            assert_eq!(
                denial.kind(),
                &SignalMergeStrategyWitnessDenialKind::MissingMergeStrategyIdentity
            );
        }
        outcome => panic!("expected typed strategy witness denial, got {outcome:?}"),
    }

    let merge_identity = SignalMergeStrategyIdentity::new(
        BranchMergeStrategy::AdoptSourceHead,
        MergeStrategyName::new("signal.merge.adopt-source-head"),
        "merge-digest".to_owned(),
        MergeStrategySelectionBasis::DivergenceDefault,
        MergeBaseStrategyName::new("signal.merge-base.fork-point"),
        "merge-base-digest".to_owned(),
        MergeBaseSelectionBasis::BuiltInDefault,
        "bundle-digest".to_owned(),
    )
    .expect("merge identity should validate");
    let invalidation_identity = SignalInvalidationStrategyIdentity::new(
        MergeBoundaryWitnessKind::MutationJournalBoundary,
        ConflictIsolationPolicyName::new("signal.conflict-isolation.per-node"),
        "conflict-isolation-digest".to_owned(),
        ConflictIsolationSelectionBasis::BuiltInDefault,
        IdentityMatcherName::new("signal.identity.exact-node-id"),
        "identity-digest".to_owned(),
        IdentityMatcherSelectionBasis::BuiltInDefault,
    )
    .expect("invalidation identity should validate");

    let denied = SignalMergeStrategyWitness::try_from_identities(
        Some(merge_identity),
        Some(invalidation_identity),
        None,
    );
    match denied {
        TransitionOutcome::Denied(denial) => {
            assert_eq!(
                denial.kind(),
                &SignalMergeStrategyWitnessDenialKind::MissingDeliveryStrategyIdentity
            );
        }
        outcome => panic!("expected missing delivery strategy denial, got {outcome:?}"),
    }
}

#[test]
fn admitted_policy_changes_mutate_only_the_relevant_strategy_identity_surface() {
    let (mut default_runtime, feature, main, _node) = build_strategy_identity_runtime();
    let default_result = default_runtime
        .merge_raw()
        .from(feature)
        .into(main)
        .run()
        .expect("default merge should succeed");

    let (mut customized_runtime, feature, main, _node) = build_strategy_identity_runtime();
    let customized_result = customized_runtime
        .merge_raw()
        .from(feature)
        .into(main)
        .conflict_isolation_policy_named("signal.conflict-isolation.per-aspect")
        .run()
        .expect("explicit conflict-isolation merge should succeed");

    assert_ne!(
        default_result.strategy_witness,
        customized_result.strategy_witness,
        "a real admitted policy change must alter the retained strategy witness instead of being flattened away"
    );
    assert_ne!(
        default_result
            .strategy_witness
            .invalidation_strategy_digest(),
        customized_result
            .strategy_witness
            .invalidation_strategy_digest(),
        "changing conflict-isolation should surface as an invalidation-strategy identity change"
    );
    assert_eq!(
        customized_result.selected_conflict_isolation_name.as_str(),
        "signal.conflict-isolation.per-aspect",
        "the retained merge result should expose the explicitly requested admitted conflict-isolation policy"
    );
    assert_eq!(
        default_result.strategy_witness.delivery_strategy_digest(),
        customized_result
            .strategy_witness
            .delivery_strategy_digest(),
        "changing conflict-isolation alone should not invent a delivery-strategy change"
    );
}
