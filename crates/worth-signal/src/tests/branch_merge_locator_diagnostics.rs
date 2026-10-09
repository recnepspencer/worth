use worth_foundational::facade::{
    plan_diagnostic_explanation_bundle, AdmissionReadinessProfile, CanonicalizationRuleVersion,
    CertificationPostureProfile, CompatibilityPostureProfile, DiagnosticRichnessProfile,
    FoundationalDiagnosticDeliveryClass, FoundationalProfileSet, FoundationalProfileSetInput,
    FoundationalTransitionLocator, RetentionDeliveryProfile, SupportPostureProfile,
};
use worth_proof::TransitionOutcome;

use crate::facade::*;
use crate::tests::branch_merge_scoped_denial_support::{
    build_scoped_denial_runtime, selected_aspect_scope_digest,
};
use crate::tests::support::{version_ab, ASPECT_A, ASPECT_B};

fn retained_standard_profile() -> FoundationalProfileSet {
    FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: DiagnosticRichnessProfile::Standard,
        support_posture: SupportPostureProfile::SupportReady,
        compatibility_posture: CompatibilityPostureProfile::CompatibilityLowered,
        admission_readiness: AdmissionReadinessProfile::Admitted,
        retention_delivery: RetentionDeliveryProfile::Retained,
        certification_posture: CertificationPostureProfile::EvidenceBacked,
        execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
        observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
    })
    .expect("retained standard profile should be coherent")
}

fn build_locator_runtime() -> (
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
    let feature = runtime.create_branch("feature-phase8-locator").unwrap();
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

#[test]
fn scoped_merge_locators_and_compact_diagnostics_round_trip_from_retained_proof() {
    let (mut runtime, feature, main, node) = build_locator_runtime();
    let result = runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .selected_aspects([
            SignalSelectedAspectRequestEntry::new(node, ASPECT_A),
            SignalSelectedAspectRequestEntry::new(node, ASPECT_B),
        ])
        .run()
        .expect("selected-aspect merge should execute");

    let locator_bundle = result.scoped_merge_locator_bundle();
    assert!(matches!(
        locator_bundle.scope(),
        FoundationalTransitionLocator::MergeScope(_)
    ));
    assert_eq!(locator_bundle.requested().len(), 2);
    assert_eq!(locator_bundle.admitted().len(), 2);
    assert_eq!(locator_bundle.no_op().len(), 1);
    assert!(locator_bundle.skipped().is_empty());
    let locator_basis = match result
        .scoped_merge_proof
        .prepare_locator_canonical_basis_bundle(
            CanonicalizationRuleVersion::new("worth.signal.phase8.locator-basis")
                .expect("valid locator basis version"),
            feature.id,
            main.id,
        ) {
        TransitionOutcome::Success(ready) => ready,
        outcome => panic!("expected locator canonical basis bundle, got {outcome:?}"),
    };
    assert_ne!(
        locator_basis.scope().payload(),
        locator_basis.requested()[0].payload()
    );
    assert_ne!(
        locator_basis.admitted()[0].payload(),
        locator_basis.no_op()[0].payload()
    );

    let result_rows = result.scoped_merge_compact_diagnostic_rows();
    let retained_proof = runtime
        .replay_for_branch(main.id)
        .frames
        .iter()
        .rev()
        .find_map(|frame| {
            frame
                .detail
                .as_ref()
                .and_then(|detail| detail.as_scoped_merge_proof())
        })
        .cloned()
        .expect("retained replay proof should exist for scoped merge");
    let retained_rows = retained_proof.compact_diagnostic_rows(feature.id, main.id);
    let result_explanation = plan_diagnostic_explanation_bundle(
        result
            .scoped_merge_proof
            .prepare_request_diagnostic_explanation(feature.id, main.id),
        retained_standard_profile(),
        FoundationalDiagnosticDeliveryClass::MustBeHot,
    )
    .expect("result request explanation should materialize")
    .selected_rows();
    let retained_explanation = plan_diagnostic_explanation_bundle(
        retained_proof.prepare_request_diagnostic_explanation(feature.id, main.id),
        retained_standard_profile(),
        FoundationalDiagnosticDeliveryClass::MustBeHot,
    )
    .expect("retained request explanation should materialize")
    .selected_rows();

    assert_eq!(result_rows, retained_rows);
    assert_eq!(result_explanation, retained_explanation);
    assert!(result_rows
        .iter()
        .any(|row| row.code() == "merge-scope.requested"));
    assert!(result_rows
        .iter()
        .any(|row| row.code() == "merge-scope.admitted"));
    assert!(result_rows
        .iter()
        .any(|row| row.code() == "merge-scope.no-op"));
    assert_eq!(
        result_rows
            .iter()
            .find(|row| row.code() == "merge-scope.requested")
            .expect("requested row")
            .digest(),
        result.scoped_merge_proof.declaration_digest()
    );
}

#[test]
fn scoped_denial_and_unavailable_locators_stay_family_distinct_and_canonicalizable() {
    let version = CanonicalizationRuleVersion::new("worth.signal.phase8.locator-tests")
        .expect("valid canonicalization version");
    let (mut runtime, feature, main, _primary) = build_scoped_denial_runtime();
    let denied_entry = SignalSelectedAspectRequestEntry::new(NodeId::new(777, 2), ASPECT_A);
    let denied = match runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .selected_aspects([denied_entry.clone()])
        .plan()
    {
        Err(SignalError::BranchMergeFailed {
            evidence: Some(evidence),
            ..
        }) => match *evidence {
            BranchMergeFailureEvidence::ScopedDenial(evidence) => evidence,
            other => panic!("expected scoped denial evidence, got {other:?}"),
        },
        _ => panic!("expected scoped denial evidence"),
    };
    assert_eq!(
        denied.scope_digest,
        selected_aspect_scope_digest(&feature, &main, denied_entry)
    );

    let unavailable = match runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .selected_nodes([NodeId::new(1, 0)])
        .strategy_hint(BranchMergeStrategy::RebaseSourceOntoTarget)
        .plan()
    {
        Err(SignalError::BranchMergeFailed {
            evidence: Some(evidence),
            ..
        }) => match *evidence {
            BranchMergeFailureEvidence::ScopedUnavailable(evidence) => evidence,
            other => panic!("expected scoped unavailable evidence, got {other:?}"),
        },
        _ => panic!("expected scoped unavailable evidence"),
    };

    let denied_locator = denied.denied_locator(feature.id, main.id);
    let unavailable_locator = unavailable.unavailable_locator(feature.id, main.id);
    assert!(matches!(
        denied_locator,
        FoundationalTransitionLocator::SelectedAspectScope(_)
    ));
    assert!(matches!(
        unavailable_locator,
        FoundationalTransitionLocator::MergeScope(_)
    ));

    let denied_basis = match denied.prepare_canonical_basis(version.clone(), feature.id, main.id) {
        TransitionOutcome::Success(ready) => ready,
        outcome => panic!("expected scoped denial canonical basis, got {outcome:?}"),
    };
    let unavailable_basis = match unavailable.prepare_canonical_basis(version, feature.id, main.id)
    {
        TransitionOutcome::Success(ready) => ready,
        outcome => panic!("expected scoped unavailable canonical basis, got {outcome:?}"),
    };
    assert_ne!(
        denied_basis.payload().entries(),
        unavailable_basis.payload().entries()
    );
    let denied_explanation = plan_diagnostic_explanation_bundle(
        denied.prepare_diagnostic_explanation(feature.id, main.id),
        retained_standard_profile(),
        FoundationalDiagnosticDeliveryClass::MustBeHot,
    )
    .expect("denied explanation should materialize")
    .selected_rows();
    let unavailable_explanation = plan_diagnostic_explanation_bundle(
        unavailable.prepare_diagnostic_explanation(feature.id, main.id),
        retained_standard_profile(),
        FoundationalDiagnosticDeliveryClass::MustBeHot,
    )
    .expect("unavailable explanation should materialize")
    .selected_rows();
    assert_ne!(denied_explanation, unavailable_explanation);
    assert_eq!(
        denied.compact_diagnostic_row(feature.id, main.id).code(),
        "merge-scope.denied"
    );
    assert_eq!(
        unavailable
            .compact_diagnostic_row(feature.id, main.id)
            .code(),
        "merge-scope.unavailable"
    );
}
