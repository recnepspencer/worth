//! common lower runtime explanation review matches proof lane review.
use super::*;

#[test]
fn common_lower_runtime_explanation_review_matches_proof_lane_review() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let envelope = lower_runtime_envelope("boundary-explanation-review");
    let (reference_set, inspection_target) = replay_gap_inputs();
    let installation = install_domain_capability_certification(request_execution);
    let domain = installation.contributions();
    let target = domain
        .lower_runtime_target(&envelope)
        .expect("installed contribution authority must remain current");

    let common = domain
        .for_lower_runtime_target(target.clone())
        .expect("certification target should belong to its installed domain")
        .explains_store_backed_replay_gap(
            "replay.store_gap",
            WorthQueryLowerRuntimeExplanationRequest::explains_store_backed_replay_gap(
                reference_set.clone(),
                inspection_target.clone(),
                vec![CausalEvidenceFamily::QueryInspection],
                CausalInspectionRedactionPolicy::PreserveDetail,
                CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
            ),
        )
        .because("store-backed replay still lacks the retained lower-runtime evidence lane")
        .review()
        .expect("lower-runtime explanation review should materialize");

    let proof_requested =
        WorthQueryExplanationContributionAuthoring::store_backed_replay_gap_explanation(
            "worth.spatial.replay.store_gap",
            "store-backed replay still lacks the retained lower-runtime evidence lane",
            reference_set,
            inspection_target,
            vec![CausalEvidenceFamily::QueryInspection],
            CausalInspectionRedactionPolicy::PreserveDetail,
            CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
        )
        .bind_to_installed_target(target);
    let proof_admitted = success(admit_eligible_domain_capability_contribution(success(
        evaluate_requested_domain_capability_contribution(proof_requested),
    )));
    let proof_target = proof_admitted.payload().target().clone();
    let proof = materialize_query_causal_inspection_review(success(
        prepare_admitted_domain_capability_contribution_for_materialization(
            proof_admitted,
            proof_target,
        ),
    ));

    assert_eq!(common, success(proof));
}
