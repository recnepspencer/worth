use worth_proof::TransitionOutcome;

use crate::diagnostics::replay::{ReplayEvent, ReplayEventDetail, ReplayEventKind};
use crate::facade::*;
use crate::logic::transaction::{
    bridge_signal_merge_compatibility_trust_boundary,
    BoundaryBridgedSignalMergeCompatibilityArtifact, SignalMergeCompatibilityArtifact,
    SignalMergeCompatibilityDenial, SignalMergeCompatibilityDenialKind,
    SignalMergeCompatibilityPostureKind, SignalMergeSupportInspectionAbsence,
    SignalMergeSupportInspectionAbsenceKind, SignalMergeSupportInspectionWitness,
    SignalMergeSupportReadinessPosture,
};
use crate::tests::support::{version_ab, ASPECT_A};

fn build_phase11_runtime() -> (
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
    let feature = runtime.create_branch("feature-phase11-inspection").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.mark_dirty(node, ASPECT_A)?;
            tx.read(node, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(11, 0))))
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
        other => panic!("expected branch basis artifact, got {other:?}"),
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

fn expect_support(
    outcome: TransitionOutcome<
        SignalMergeSupportInspectionWitness,
        SignalMergeSupportInspectionAbsence,
    >,
) -> SignalMergeSupportInspectionWitness {
    match outcome {
        TransitionOutcome::Success(witness) => witness,
        other => panic!("expected merge support inspection witness, got {other:?}"),
    }
}

fn latest_branch_merge_event(
    runtime: &SignalRuntime<(), (), (), (), ()>,
    branch_id: SignalBranchId,
) -> ReplayEvent {
    runtime
        .replay_for_branch(branch_id)
        .frames
        .iter()
        .rev()
        .find(|event| event.kind == ReplayEventKind::BranchMerged)
        .cloned()
        .expect("branch merge replay event should exist")
}

#[test]
fn support_inspection_is_equivalent_across_result_replay_and_compatibility_lanes() {
    let (mut summary_runtime, summary_feature, summary_main, _node) = build_phase11_runtime();
    let planned = summary_runtime
        .merge_raw()
        .from(summary_feature)
        .into(summary_main.clone())
        .plan()
        .expect("summary merge planning should succeed");
    let lowered_request = planned.lowered_request().clone();
    let plan = planned.plan().clone();
    drop(planned);
    let summary = summary_runtime
        .execute_branch_merge_request_plan_summary_for_test(&lowered_request, &plan)
        .expect("ordinary merge execution summary should succeed");
    let summary_basis = expect_branch_basis(&mut summary_runtime, summary_main.clone());
    let summary_support =
        expect_support(summary_runtime.merge_execution_summary_support_inspection(
            summary_basis,
            summary_main,
            &summary,
        ));

    let (mut ordinary_runtime, feature, main, _node) = build_phase11_runtime();
    let result = ordinary_runtime
        .merge_raw()
        .from(feature)
        .into(main.clone())
        .run()
        .expect("ordinary merge result should succeed");
    let basis = expect_branch_basis(&mut ordinary_runtime, main.clone());
    let result_support = expect_support(ordinary_runtime.merge_result_support_inspection(
        basis.clone(),
        main.clone(),
        &result,
    ));
    let replay_support = expect_support(ordinary_runtime.replay_merge_support_inspection(
        basis.clone(),
        main.clone(),
        &latest_branch_merge_event(&ordinary_runtime, main.id),
    ));
    let compatibility = expect_compatibility(ordinary_runtime.merge_result_compatibility_artifact(
        basis.clone(),
        main.clone(),
        &result,
    ));
    let compatibility_support =
        expect_support(ordinary_runtime.merge_compatibility_support_inspection(
            basis.clone(),
            main.clone(),
            &result.scoped_merge_proof,
            &result.strategy_witness,
            &compatibility,
        ));

    let (mut compatibility_runtime, compatibility_feature, compatibility_main, _node) =
        build_phase11_runtime();
    let compatibility_result = compatibility_runtime
        .merge_branch_raw(compatibility_feature, compatibility_main.clone())
        .expect("compatibility merge lane should succeed");
    let compatibility_basis =
        expect_branch_basis(&mut compatibility_runtime, compatibility_main.clone());
    let compatibility_lane_support =
        expect_support(compatibility_runtime.merge_result_support_inspection(
            compatibility_basis,
            compatibility_main,
            &compatibility_result,
        ));

    assert_eq!(result_support, summary_support);
    assert_eq!(result_support, replay_support);
    assert_eq!(result_support, compatibility_support);
    assert_eq!(result_support, compatibility_lane_support);
    assert_eq!(
        result_support.readiness_posture(),
        SignalMergeSupportReadinessPosture::CurrentBasis
    );

    let bridged: BoundaryBridgedSignalMergeCompatibilityArtifact =
        bridge_signal_merge_compatibility_trust_boundary(compatibility);
    let bridged_support = expect_support(
        ordinary_runtime.bridged_merge_compatibility_support_inspection(
            basis,
            main,
            &result.scoped_merge_proof,
            &result.strategy_witness,
            &bridged,
        ),
    );

    assert_eq!(
        bridged_support.branch_basis_row(),
        result_support.branch_basis_row()
    );
    assert_eq!(bridged_support.scope_row(), result_support.scope_row());
    assert_eq!(
        bridged_support.strategy_row(),
        result_support.strategy_row()
    );
    assert_eq!(
        bridged_support.readiness_posture(),
        SignalMergeSupportReadinessPosture::BoundaryBridgedAuthorityRevalidationRequired
    );
    assert_ne!(
        bridged_support.inspection_digest(),
        result_support.inspection_digest()
    );
}

#[test]
fn support_inspection_refuses_to_synthesize_missing_retained_posture() {
    let (mut runtime, feature, main, _node) = build_phase11_runtime();
    let planned = runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .plan()
        .expect("planning should succeed");
    let scoped_merge_proof = planned.plan().scoped_merge_proof().clone();
    let strategy_witness = planned.plan().strategy_witness().clone();
    drop(planned);
    let basis = expect_branch_basis(&mut runtime, main.clone());

    match runtime.merge_support_inspection_from_retained_parts(
        basis.clone(),
        main.clone(),
        Some(&scoped_merge_proof),
        Some(&strategy_witness),
        None,
        SignalMergeCompatibilityPostureKind::CurrentBasis,
    ) {
        TransitionOutcome::Denied(absence) => {
            assert_eq!(
                absence.kind(),
                SignalMergeSupportInspectionAbsenceKind::MissingCompatibilityWitness
            );
        }
        other => panic!("expected missing compatibility witness absence, got {other:?}"),
    }

    let result = runtime
        .merge_raw()
        .from(feature)
        .into(main.clone())
        .run()
        .expect("merge execution should succeed");
    let post_merge_basis = expect_branch_basis(&mut runtime, main.clone());

    match runtime.merge_support_inspection_from_retained_parts(
        post_merge_basis.clone(),
        main.clone(),
        None,
        Some(&result.strategy_witness),
        Some(&result.compatibility_witness),
        SignalMergeCompatibilityPostureKind::CurrentBasis,
    ) {
        TransitionOutcome::Denied(SignalMergeSupportInspectionAbsence::CompatibilityDenied(
            denial,
        )) => {
            assert_eq!(
                denial.kind(),
                SignalMergeCompatibilityDenialKind::MissingScopedMergeProof
            );
        }
        other => panic!("expected missing scoped merge proof denial, got {other:?}"),
    }

    let replay_event = ReplayEvent {
        detail: Some(ReplayEventDetail::Message(
            "not a branch merge summary".to_owned(),
        )),
        ..latest_branch_merge_event(&runtime, main.id)
    };
    match runtime.replay_merge_support_inspection(post_merge_basis, main, &replay_event) {
        TransitionOutcome::Denied(absence) => {
            assert_eq!(
                absence.kind(),
                SignalMergeSupportInspectionAbsenceKind::ReplayDetailUnavailable
            );
        }
        other => panic!("expected replay detail unavailable absence, got {other:?}"),
    }
}
