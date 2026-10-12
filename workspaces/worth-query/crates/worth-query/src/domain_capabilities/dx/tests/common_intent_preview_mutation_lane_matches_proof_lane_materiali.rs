//! common intent preview mutation lane matches proof lane materialization.
use super::*;

#[test]
fn common_intent_preview_mutation_lane_matches_proof_lane_materialization() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = intent_declaration("intent-preview-mutation");
    let installation = install_domain_capability_certification(request_execution);
    let domain = installation.contributions();
    let target = domain
        .intent_target(&declaration)
        .expect("installed contribution authority must remain current");

    let common = domain
        .for_intent_target(target.clone())
        .expect("certification target should belong to its installed domain")
        .plans_preview_mutation(
            "topology.preview_mutation",
            crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                "preview-session:77",
            ),
        )
        .because("promotion-eligible preview can plan a bounded mutation workflow")
        .materialize()
        .expect("preview mutation lane should materialize");

    let proof_requested =
        WorthQueryWorkflowContributionAuthoring::promotion_eligible_mutation_lowering(
            "worth.spatial.topology.preview_mutation",
            "promotion-eligible preview can plan a bounded mutation workflow",
            crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                "preview-session:77",
            ),
        )
        .bind_to_installed_target(target);
    let proof_admitted = success(admit_eligible_domain_capability_contribution(success(
        evaluate_requested_domain_capability_contribution(proof_requested),
    )));
    let proof_target = proof_admitted.payload().target().clone();
    let proof = materialize_query_workflow_declaration(success(
        prepare_admitted_domain_capability_contribution_for_materialization(
            proof_admitted,
            proof_target,
        ),
    ));

    assert_eq!(common.workflow_declaration(), &success(proof));
}
