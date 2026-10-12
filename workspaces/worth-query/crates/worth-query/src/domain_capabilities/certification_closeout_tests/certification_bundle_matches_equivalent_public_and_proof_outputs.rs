//! certification bundle matches equivalent public and proof outputs.
use super::*;

#[test]
fn certification_bundle_matches_equivalent_public_and_proof_outputs() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = intent_declaration();
    let lower_runtime = lower_runtime_envelope("domain-capability-closeout");
    let (reference_set, target) = replay_gap_inputs();
    let installation = install_domain_capability_certification(request_execution);
    let domain = installation.contributions();
    let bundle = certify_domain_capabilities_in(domain);
    let declaration_target = domain
        .intent_target(&declaration)
        .expect("installed contribution authority must remain current");
    let lower_runtime_target = domain
        .lower_runtime_target(&lower_runtime)
        .expect("installed contribution authority must remain current");

    let support = domain
        .for_intent_target(declaration_target.clone())
        .expect("certification target should belong to its installed domain")
        .supports_traceability("traceability.edge_split")
        .because("declaration-scoped support remains declaration scoped")
        .materialize()
        .expect("support common lane should materialize");
    let support_proof = success(
        materialize_intent_declaration_support_traceability_artifact(admitted_ready(
            WorthQuerySupportContributionAuthoring::declaration_traceability(
                "worth.spatial.traceability.edge_split",
                "declaration-scoped support remains declaration scoped",
            )
            .bind_to_installed_target(declaration_target.clone()),
        )),
    );
    assert_eq!(support, support_proof);
    assert_eq!(
        bundle.output_digest("support_artifact_digest"),
        Some(support.materialization_digest())
    );

    let workflow = domain
        .for_intent_target(declaration_target.clone())
        .expect("certification target should belong to its installed domain")
        .plans_preview_mutation(
            "workflow.preview_mutation",
            crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                "preview-session:certification",
            ),
        )
        .because("preview mutation planning should preserve canonical workflow semantics")
        .materialize()
        .expect("workflow common lane should materialize");
    let workflow_proof = success(materialize_query_workflow_declaration(admitted_ready(
        WorthQueryWorkflowContributionAuthoring::promotion_eligible_mutation_lowering(
            "worth.spatial.workflow.preview_mutation",
            "preview mutation planning should preserve canonical workflow semantics",
            crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                "preview-session:certification",
            ),
        )
        .bind_to_installed_target(declaration_target.clone()),
    )));
    assert_eq!(workflow.workflow_declaration(), &workflow_proof);
    assert_eq!(
        bundle.output_digest("workflow_artifact_digest"),
        Some(
            workflow
                .workflow_declaration()
                .report()
                .declaration_digest()
        )
    );

    let explanation = domain
        .for_lower_runtime_target(lower_runtime_target.clone())
        .expect("certification target should belong to its installed domain")
        .explains_store_backed_replay_gap(
            "explanation.store_backed_replay",
            crate::domain_capabilities::WorthQueryLowerRuntimeExplanationRequest::explains_store_backed_replay_gap(
                reference_set.clone(),
                target.clone(),
                vec![CausalEvidenceFamily::QueryInspection],
                CausalInspectionRedactionPolicy::PreserveDetail,
                CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
            ),
        )
        .because("store-backed replay should preserve denied explanation identity")
        .materialize_artifact()
        .expect("explanation common lane should materialize");
    let explanation_proof = success(materialize_query_causal_inspection_artifact(
        admitted_ready(
            WorthQueryExplanationContributionAuthoring::store_backed_replay_gap_explanation(
                "worth.spatial.explanation.store_backed_replay",
                "store-backed replay should preserve denied explanation identity",
                reference_set,
                target,
                vec![CausalEvidenceFamily::QueryInspection],
                CausalInspectionRedactionPolicy::PreserveDetail,
                CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
            )
            .bind_to_installed_target(lower_runtime_target),
        ),
    ));
    assert_eq!(explanation, explanation_proof);
    assert_eq!(
        bundle.output_digest("explanation_artifact_digest"),
        Some(explanation.artifact_for_reporting())
    );

    let invariant = domain
        .for_intent_target(declaration_target.clone())
        .expect("certification target should belong to its installed domain")
        .register_invariant_catalog("invariant.edge_split", InvariantCatalog::default())
        .because("geometry kernel must reject invalid edge splits")
        .materialize()
        .expect("invariant common lane should materialize");
    let invariant_proof = success(materialize_query_invariant_catalog_registration_artifact(
        admitted_ready(
            WorthQueryInvariantCapabilityContributionAuthoring::invariant_registration(
                InvariantCatalog::default(),
                "worth.spatial.invariant.edge_split",
                "geometry kernel must reject invalid edge splits",
            )
            .bind_to_installed_target(declaration_target),
        ),
    ));
    assert_eq!(invariant, invariant_proof);
}
