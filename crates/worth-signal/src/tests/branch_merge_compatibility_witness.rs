use worth_proof::TransitionOutcome;

use crate::diagnostics::replay::ReplayEventKind;
use crate::facade::*;
use crate::logic::transaction::{
    bridge_signal_merge_compatibility_trust_boundary,
    BoundaryBridgedSignalMergeCompatibilityArtifact, SignalMergeCompatibilityArtifact,
    SignalMergeCompatibilityDenial, SignalMergeCompatibilityDenialKind,
};
use crate::tests::support::{version_ab, ASPECT_A};

fn build_phase10_runtime() -> (
    SignalRuntime<(), (), (), (), ()>,
    SignalBranchHandle,
    SignalBranchHandle,
    NodeId,
) {
    let graph = SignalGraph::new();
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let node = runtime
        .graph_mut()
        .node()
        .reads_aspects([ASPECT_A])
        .produces_aspects([ASPECT_A])
        .build();

    runtime
        .transaction(&mut (), |tx| {
            tx.read(node, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.current_branch();
    let feature = runtime
        .create_branch("feature-phase10-compatibility")
        .unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(&mut (), |tx| {
            tx.mark_dirty(node, ASPECT_A)?;
            tx.read(node, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(10, 0))))
            })?;
            Ok(())
        })
        .unwrap();
    runtime.switch_branch(main.clone()).unwrap();

    (runtime, feature, main, node)
}

fn expect_branch_basis(
    runtime: &mut SignalRuntime<(), (), (), (), ()>,
    branch: SignalBranchHandle,
) -> SignalBranchBasisArtifact {
    match runtime.branch_basis_artifact(branch) {
        TransitionOutcome::Success(artifact) => artifact,
        outcome => panic!("expected branch basis artifact, got {outcome:?}"),
    }
}

fn expect_compatibility(
    outcome: TransitionOutcome<SignalMergeCompatibilityArtifact, SignalMergeCompatibilityDenial>,
) -> SignalMergeCompatibilityArtifact {
    match outcome {
        TransitionOutcome::Success(artifact) => artifact,
        other => panic!("expected compatibility artifact, got {other:?}"),
    }
}

fn latest_branch_merge_event(
    runtime: &SignalRuntime<(), (), (), (), ()>,
    branch_id: SignalBranchId,
) -> crate::diagnostics::replay::ReplayEvent {
    runtime
        .replay_for_branch(branch_id)
        .frames
        .iter()
        .rev()
        .find(|event| event.kind == ReplayEventKind::BranchMerged)
        .cloned()
        .expect("branch merge replay event should exist")
}

fn mismatched_strategy_witness() -> SignalMergeStrategyWitness {
    let merge_identity = SignalMergeStrategyIdentity::new(
        BranchMergeStrategy::AdoptSourceHead,
        MergeStrategyName::new("signal.merge.adopt-source-head"),
        "merge-digest-mismatch".to_owned(),
        MergeStrategySelectionBasis::DivergenceDefault,
        MergeBaseStrategyName::new("signal.merge-base.fork-point"),
        "merge-base-digest-mismatch".to_owned(),
        MergeBaseSelectionBasis::BuiltInDefault,
        "bundle-digest-mismatch".to_owned(),
    )
    .expect("merge identity should validate");
    let invalidation_identity = SignalInvalidationStrategyIdentity::new(
        MergeBoundaryWitnessKind::MutationJournalBoundary,
        ConflictIsolationPolicyName::new("signal.conflict-isolation.per-node"),
        "conflict-isolation-digest-mismatch".to_owned(),
        ConflictIsolationSelectionBasis::BuiltInDefault,
        IdentityMatcherName::new("signal.identity.exact-node-id"),
        "identity-digest-mismatch".to_owned(),
        IdentityMatcherSelectionBasis::BuiltInDefault,
    )
    .expect("invalidation identity should validate");
    let delivery_identity = SignalDeliveryStrategyIdentity::new(
        ConflictPolicyName::new("signal.conflict-policy.fail-fast"),
        "conflict-policy-digest-mismatch".to_owned(),
        ConflictPolicySelectionBasis::BuiltInDefault,
        SourceOnlyPolicyName::new("signal.source-only.adopt"),
        "source-only-digest-mismatch".to_owned(),
        SourceOnlyPolicySelectionBasis::BuiltInDefault,
        DeletionPolicyName::new("signal.deletion-policy.preserve-target"),
        "deletion-policy-digest-mismatch".to_owned(),
        DeletionPolicySelectionBasis::BuiltInDefault,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .expect("delivery identity should validate");

    match SignalMergeStrategyWitness::try_from_identities(
        Some(merge_identity),
        Some(invalidation_identity),
        Some(delivery_identity),
    ) {
        TransitionOutcome::Success(witness) => witness,
        other => panic!("expected mismatched strategy witness, got {other:?}"),
    }
}

#[test]
fn compatibility_witness_is_equivalent_across_result_replay_and_compatibility_lanes() {
    let (mut ordinary_runtime, feature, main, _node) = build_phase10_runtime();
    let ordinary_result = ordinary_runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .run()
        .expect("ordinary merge should succeed");
    let ordinary_basis = expect_branch_basis(&mut ordinary_runtime, main.clone());
    let ordinary_result_compat =
        expect_compatibility(ordinary_runtime.merge_result_compatibility_artifact(
            ordinary_basis.clone(),
            main.clone(),
            &ordinary_result,
        ));
    let ordinary_replay_compat =
        expect_compatibility(ordinary_runtime.replay_merge_compatibility_artifact(
            ordinary_basis.clone(),
            main.clone(),
            &latest_branch_merge_event(&ordinary_runtime, main.id),
        ));
    let ordinary_replay_event = latest_branch_merge_event(&ordinary_runtime, main.id);
    let retained_replay_witness = ordinary_replay_event
        .detail
        .as_ref()
        .and_then(|detail| detail.as_compatibility_witness())
        .expect("branch merge replay detail should retain compatibility witness");

    let (mut compatibility_runtime, compatibility_feature, compatibility_main, _node) =
        build_phase10_runtime();
    let compatibility_result = compatibility_runtime
        .merge_branch_raw(compatibility_feature, compatibility_main.clone())
        .expect("compatibility merge lane should succeed");
    let compatibility_basis =
        expect_branch_basis(&mut compatibility_runtime, compatibility_main.clone());
    let compatibility_artifact =
        expect_compatibility(compatibility_runtime.merge_result_compatibility_artifact(
            compatibility_basis,
            compatibility_main,
            &compatibility_result,
        ));

    assert_eq!(
        ordinary_result.compatibility_witness,
        *retained_replay_witness
    );
    assert_eq!(
        ordinary_result_compat.payload().compatibility_digest(),
        ordinary_replay_compat.payload().compatibility_digest()
    );
    assert_eq!(
        ordinary_result_compat.payload().fact_inventory(),
        ordinary_replay_compat.payload().fact_inventory()
    );
    assert_eq!(
        ordinary_result.compatibility_witness.compatibility_digest(),
        retained_replay_witness.compatibility_digest()
    );
    assert_eq!(
        ordinary_result_compat.payload().compatibility_digest(),
        compatibility_artifact.payload().compatibility_digest()
    );
    assert_eq!(
        ordinary_result_compat.payload().fact_inventory(),
        compatibility_artifact.payload().fact_inventory()
    );
}

#[test]
fn stale_missing_and_mismatched_inputs_are_denied_before_compatibility_publication() {
    let (mut stale_runtime, stale_feature, stale_main, _node) = build_phase10_runtime();
    let stale_basis = expect_branch_basis(&mut stale_runtime, stale_main.clone());
    let stale_result = stale_runtime
        .merge_raw()
        .from(stale_feature)
        .into(stale_main.clone())
        .run()
        .expect("merge execution should succeed");

    match stale_runtime.merge_result_compatibility_artifact(stale_basis, stale_main, &stale_result)
    {
        TransitionOutcome::Denied(denial) => {
            assert_eq!(
                denial.kind(),
                SignalMergeCompatibilityDenialKind::StaleBranchBasis
            );
        }
        other => panic!("expected stale basis denial, got {other:?}"),
    }

    let (mut runtime, feature, main, _node) = build_phase10_runtime();
    let planned = runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .plan()
        .expect("planning should succeed");
    let strategy_witness = planned.plan().strategy_witness().clone();
    drop(planned);
    let fresh_basis = expect_branch_basis(&mut runtime, main.clone());
    match runtime.merge_compatibility_artifact_from_parts(
        fresh_basis,
        main.clone(),
        None,
        Some(strategy_witness),
    ) {
        TransitionOutcome::Denied(denial) => {
            assert_eq!(
                denial.kind(),
                SignalMergeCompatibilityDenialKind::MissingScopedMergeProof
            );
        }
        other => panic!("expected missing scoped proof denial, got {other:?}"),
    }

    let result = runtime
        .merge_raw()
        .from(feature)
        .into(main.clone())
        .run()
        .expect("merge execution should succeed");
    let post_merge_basis = expect_branch_basis(&mut runtime, main.clone());
    let compatibility = expect_compatibility(runtime.merge_result_compatibility_artifact(
        post_merge_basis.clone(),
        main.clone(),
        &result,
    ));
    let bridged: BoundaryBridgedSignalMergeCompatibilityArtifact =
        bridge_signal_merge_compatibility_trust_boundary(compatibility);

    match runtime.readmit_merge_compatibility_artifact(
        bridged,
        post_merge_basis,
        main,
        Some(result.scoped_merge_proof.clone()),
        Some(mismatched_strategy_witness()),
    ) {
        TransitionOutcome::Denied(denial) => {
            assert_eq!(
                denial.kind(),
                SignalMergeCompatibilityDenialKind::StrategyWitnessMismatch
            );
        }
        other => panic!("expected strategy witness mismatch denial, got {other:?}"),
    }
}

#[test]
fn trust_boundary_bridge_and_readmission_preserve_compatibility_truth_without_digest_drift() {
    let (mut runtime, feature, main, _node) = build_phase10_runtime();
    let result = runtime
        .merge_raw()
        .from(feature)
        .into(main.clone())
        .run()
        .expect("merge execution should succeed");
    let post_merge_basis = expect_branch_basis(&mut runtime, main.clone());
    let current = expect_compatibility(runtime.merge_result_compatibility_artifact(
        post_merge_basis.clone(),
        main.clone(),
        &result,
    ));
    let bridged: BoundaryBridgedSignalMergeCompatibilityArtifact =
        bridge_signal_merge_compatibility_trust_boundary(current.clone());

    let readmitted = expect_compatibility(runtime.readmit_merge_compatibility_artifact(
        bridged,
        post_merge_basis,
        main,
        Some(result.scoped_merge_proof.clone()),
        Some(result.strategy_witness.clone()),
    ));

    assert_eq!(
        current.payload().compatibility_digest(),
        readmitted.payload().compatibility_digest()
    );
    assert_eq!(
        current.payload().fact_inventory(),
        readmitted.payload().fact_inventory()
    );
}

/// Native merge fixture shared by retained-replay storage proofs.
pub(crate) fn emitted_merge_replay_event() -> crate::diagnostics::replay::ReplayEvent {
    let (mut runtime, feature, main, _) = build_phase10_runtime();
    runtime
        .merge_raw()
        .from(feature)
        .into(main.clone())
        .run()
        .unwrap();
    latest_branch_merge_event(&runtime, main.id)
}
