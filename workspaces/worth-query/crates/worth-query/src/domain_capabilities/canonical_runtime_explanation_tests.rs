use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::TransitionOutcome;

use super::targets::WorthQueryLowerRuntimeBoundaryBoundContributionTarget;
use super::test_support::{lower_runtime_target, ready, success};
use super::{
    materialize_query_causal_inspection_artifact, materialize_query_causal_inspection_review,
    WorthQueryExplanationContributionAuthoring,
};
use crate::runtime::{
    anchor_causal_observation, causal_inspection_target, resolve_causal_evidence_references,
    CausalEvidenceFamily, CausalEvidenceReferenceResolution, CausalInspectionReason,
    CausalInspectionSupportPosture, WorthQueryReadExecutionEngine, WorthQueryReadReceipt,
};

#[test]
fn explanation_runtime_materializer_builds_denied_causal_artifact(
) -> Result<(), IdentityDerivationDenial> {
    let read_receipt = WorthQueryReadReceipt::test_only(
        "read-graph:domain-capability",
        "query:domain-capability",
        "basis:domain-capability",
        "result:domain-capability",
        WorthQueryReadExecutionEngine::QueryRuntimeHistorical,
    )?;
    let observation = crate::runtime::QueryObservationReceipt::from_read_receipt(
        &read_receipt,
        crate::basis_lifecycle::basis_lifecycle()
            .historical_snapshot("domain-capability", true)
            .inspect()
            .unwrap(),
    )?;
    let anchor = anchor_causal_observation(
        observation.clone(),
        CausalInspectionReason::HistoricalReplayResult,
    )
    .expect("historical replay observation should anchor");
    let CausalEvidenceReferenceResolution::Resolved { reference_set, .. } =
        resolve_causal_evidence_references(anchor, &[CausalEvidenceFamily::QueryInspection])
    else {
        panic!("query-inspection-only replay evidence should resolve");
    };
    let target = causal_inspection_target(
        observation.observation_target().clone(),
        observation.result_shape_context().clone(),
    )
    .expect("observation-derived target should be valid");

    let artifact = success(materialize_query_causal_inspection_artifact(
        ready_explanation(
            WorthQueryExplanationContributionAuthoring::store_backed_replay_gap_explanation(
                "explanation.store_backed_replay",
                "store-backed replay should deny without the required lower-runtime evidence",
                reference_set,
                target,
                vec![CausalEvidenceFamily::QueryInspection],
                crate::runtime::CausalInspectionRedactionPolicy::PreserveDetail,
                crate::runtime::CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
            )?,
        )?,
    )?);

    assert_eq!(
        artifact.kind(),
        crate::runtime::CausalInspectionArtifactKind::Denied
    );
    assert!(artifact.is_denied());
    assert!(artifact.bridge_envelope_for_reporting().is_none());
    assert!(!artifact.artifact_for_reporting().is_empty());

    Ok(())
}

#[test]
fn explanation_runtime_materializer_denies_missing_semantics(
) -> Result<(), IdentityDerivationDenial> {
    let outcome = materialize_query_causal_inspection_artifact(ready_explanation(
        WorthQueryExplanationContributionAuthoring::requires_context(
            "explanation.support.only",
            "support-only explanation cannot mint a causal inspection artifact",
        )?,
    )?)?;

    assert!(matches!(
        outcome,
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == super::WorthQueryDomainCapabilityProgressionDenialKind::MissingCanonicalMaterializationSemantics
    ));

    Ok(())
}

#[test]
fn explanation_runtime_review_preserves_denied_plan_identity(
) -> Result<(), IdentityDerivationDenial> {
    let read_receipt = WorthQueryReadReceipt::test_only(
        "read-graph:domain-capability",
        "query:domain-capability",
        "basis:domain-capability",
        "result:domain-capability",
        WorthQueryReadExecutionEngine::QueryRuntimeHistorical,
    )?;
    let observation = crate::runtime::QueryObservationReceipt::from_read_receipt(
        &read_receipt,
        crate::basis_lifecycle::basis_lifecycle()
            .historical_snapshot("domain-capability", true)
            .inspect()
            .unwrap(),
    )?;
    let anchor = anchor_causal_observation(
        observation.clone(),
        CausalInspectionReason::HistoricalReplayResult,
    )
    .expect("historical replay observation should anchor");
    let CausalEvidenceReferenceResolution::Resolved { reference_set, .. } =
        resolve_causal_evidence_references(anchor, &[CausalEvidenceFamily::QueryInspection])
    else {
        panic!("query-inspection-only replay evidence should resolve");
    };
    let target = causal_inspection_target(
        observation.observation_target().clone(),
        observation.result_shape_context().clone(),
    )
    .expect("observation-derived target should be valid");
    let review_contribution = ready_explanation(
        WorthQueryExplanationContributionAuthoring::store_backed_replay_gap_explanation(
            "explanation.store_backed_replay",
            "store-backed replay should deny without the required lower-runtime evidence",
            reference_set.clone(),
            target.clone(),
            vec![CausalEvidenceFamily::QueryInspection],
            crate::runtime::CausalInspectionRedactionPolicy::PreserveDetail,
            crate::runtime::CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
        )?,
    )?;
    let artifact_contribution = ready_explanation(
        WorthQueryExplanationContributionAuthoring::store_backed_replay_gap_explanation(
            "explanation.store_backed_replay",
            "store-backed replay should deny without the required lower-runtime evidence",
            reference_set,
            target,
            vec![CausalEvidenceFamily::QueryInspection],
            crate::runtime::CausalInspectionRedactionPolicy::PreserveDetail,
            crate::runtime::CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
        )?,
    )?;

    let review = success(materialize_query_causal_inspection_review(
        review_contribution,
    )?);
    let artifact = success(materialize_query_causal_inspection_artifact(
        artifact_contribution,
    )?);

    assert_eq!(
        review.plan().support_posture(),
        CausalInspectionSupportPosture::Denied
    );
    assert_eq!(review.semantic_code(), "explanation.store_backed_replay");
    match artifact {
        crate::runtime::QueryCausalInspectionArtifact::Denied(denied) => {
            assert_eq!(
                denied.query_denial_for_reporting(),
                review.plan().admission_digest()
            );
        }
        other => panic!("expected denied causal inspection artifact, got {other:?}"),
    }

    Ok(())
}

fn ready_explanation(
    authoring: WorthQueryExplanationContributionAuthoring,
) -> Result<
    super::WorthQueryMaterializationReadyExplanationContribution<
        WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
    >,
    IdentityDerivationDenial,
> {
    Ok(ready(
        authoring
            .bind_to_lower_runtime_boundary_target(lower_runtime_target("boundary-explanation")?)
            .expect("fixture digest must fit"),
    )?)
}
