use worth_proof::TransitionOutcome;

use crate::domain_capabilities::certification::install_domain_capability_certification;
use crate::domain_capabilities::certification_closeout_test_support::{
    admitted_basis_observation_plan, admitted_projection_consumption_plan, intent_declaration,
    lower_runtime_envelope, projection_binding, projection_source, replay_gap_inputs,
};
use worth_relational::facade::runtime::InvariantCatalog;

fn success<T>(
    outcome: crate::domain_capabilities::WorthQueryDomainCapabilityTransitionOutcome<T>,
) -> T {
    match outcome {
        TransitionOutcome::Success(value) => value,
        _ => panic!("expected success"),
    }
}

fn projection_contract_request() -> crate::domain_capabilities::WorthQueryProjectionContractRequest
{
    crate::domain_capabilities::WorthQueryProjectionContractRequest::new(
        projection_source(),
        projection_binding(),
        crate::projection_consumption::ProjectMaterializedFacts::declare().display_field_path(
            crate::projection_consumption::projection_fact_field_path_from_segments([
                worth_foundational::facade::FieldKey::new("field")
                    .expect("projection fact field segment should admit"),
                worth_foundational::facade::FieldKey::new("visible")
                    .expect("projection fact field segment should admit"),
            ]),
        ),
    )
}

fn store_backed_replay_gap_request(
) -> crate::domain_capabilities::WorthQueryLowerRuntimeExplanationRequest {
    let (reference_set, target) = replay_gap_inputs();
    crate::domain_capabilities::WorthQueryLowerRuntimeExplanationRequest::explains_store_backed_replay_gap(
        reference_set,
        target,
        vec![crate::runtime::CausalEvidenceFamily::QueryInspection],
        crate::runtime::CausalInspectionRedactionPolicy::PreserveDetail,
        crate::runtime::CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
    )
}

#[test]
fn public_common_and_checked_lanes_converge_for_named_categories() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = intent_declaration();
    let admitted_plan = admitted_basis_observation_plan();
    let projection_plan = admitted_projection_consumption_plan();
    let lower_runtime = lower_runtime_envelope("domain-capability-public-lane");
    let installation = install_domain_capability_certification(request_execution);
    let domain = installation.contributions();

    let support_common = domain
        .for_intent(&declaration)
        .expect("installed contribution authority must remain current")
        .supports_traceability("traceability.edge_split")
        .because("public common and checked support lanes must agree")
        .materialize()
        .expect("common support lane should materialize");
    let support_checked = success(
        domain
            .for_intent(&declaration)
            .expect("installed contribution authority must remain current")
            .supports_traceability("traceability.edge_split")
            .because("public common and checked support lanes must agree")
            .try_materialize()
            .into_transition_outcome(),
    );
    assert_eq!(support_common, support_checked);

    let admission_common = domain
        .for_admitted_intent_plan(&admitted_plan)
        .expect("installed contribution authority must remain current")
        .advises("admission.routing_gap")
        .because("public common and checked admission lanes must agree")
        .materialize()
        .expect("common admission lane should materialize");
    let admission_checked = success(
        domain
            .for_admitted_intent_plan(&admitted_plan)
            .expect("installed contribution authority must remain current")
            .advises("admission.routing_gap")
            .because("public common and checked admission lanes must agree")
            .try_materialize()
            .into_transition_outcome(),
    );
    assert_eq!(admission_common, admission_checked);

    let workflow_common = domain
        .for_intent(&declaration)
        .expect("installed contribution authority must remain current")
        .plans_preview_mutation(
            "workflow.preview_mutation",
            crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                "preview-session:public-lane",
            ),
        )
        .because("public common and checked workflow lanes must agree")
        .materialize()
        .expect("common workflow lane should materialize");
    let workflow_checked = success(
        domain
            .for_intent(&declaration)
            .expect("installed contribution authority must remain current")
            .plans_preview_mutation(
                "workflow.preview_mutation",
                crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                    "preview-session:public-lane",
                ),
            )
            .because("public common and checked workflow lanes must agree")
            .try_materialize()
            .into_transition_outcome(),
    );
    assert_eq!(workflow_common, workflow_checked);

    let continuity_common = domain
        .for_admitted_intent_plan(&admitted_plan)
        .expect("installed contribution authority must remain current")
        .preserves_continuity("continuity.edge_split", "edge:before", "edge:after")
        .because("public common and checked continuity lanes must agree")
        .materialize()
        .expect("common continuity lane should materialize");
    let continuity_checked = success(
        domain
            .for_admitted_intent_plan(&admitted_plan)
            .expect("installed contribution authority must remain current")
            .preserves_continuity("continuity.edge_split", "edge:before", "edge:after")
            .because("public common and checked continuity lanes must agree")
            .try_materialize()
            .into_transition_outcome(),
    );
    assert_eq!(continuity_common, continuity_checked);

    let aftermath_common = domain
        .for_admitted_intent_plan(&projection_plan)
        .expect("installed contribution authority must remain current")
        .consumes_projection_contract(
            "aftermath.projection_contract",
            projection_contract_request(),
        )
        .because("public common and checked aftermath lanes must agree")
        .materialize()
        .expect("common aftermath lane should materialize");
    let aftermath_checked = success(
        domain
            .for_admitted_intent_plan(&projection_plan)
            .expect("installed contribution authority must remain current")
            .consumes_projection_contract(
                "aftermath.projection_contract",
                projection_contract_request(),
            )
            .because("public common and checked aftermath lanes must agree")
            .try_materialize()
            .into_transition_outcome(),
    );
    assert_eq!(aftermath_common, aftermath_checked);

    let explanation_common = domain
        .for_lower_runtime_boundary_envelope(&lower_runtime)
        .expect("installed contribution authority must remain current")
        .explains_store_backed_replay_gap(
            "explanation.store_backed_replay",
            store_backed_replay_gap_request(),
        )
        .because("public common and checked explanation lanes must agree")
        .materialize_artifact()
        .expect("common explanation lane should materialize");
    let explanation_checked = success(
        domain
            .for_lower_runtime_boundary_envelope(&lower_runtime)
            .expect("installed contribution authority must remain current")
            .explains_store_backed_replay_gap(
                "explanation.store_backed_replay",
                store_backed_replay_gap_request(),
            )
            .because("public common and checked explanation lanes must agree")
            .try_materialize_artifact()
            .into_transition_outcome(),
    );
    assert_eq!(explanation_common, explanation_checked);

    let invariant_common = domain
        .for_intent(&declaration)
        .expect("installed contribution authority must remain current")
        .register_invariant_catalog("invariant.edge_split", InvariantCatalog::default())
        .because("public common and checked invariant lanes must agree")
        .materialize()
        .expect("common invariant lane should materialize");
    let invariant_checked = success(
        domain
            .for_intent(&declaration)
            .expect("installed contribution authority must remain current")
            .register_invariant_catalog("invariant.edge_split", InvariantCatalog::default())
            .because("public common and checked invariant lanes must agree")
            .try_materialize()
            .into_transition_outcome(),
    );
    assert_eq!(invariant_common, invariant_checked);
}
