use worth_proof::TransitionOutcome;
use worth_relational::facade::runtime::InvariantCatalog;

use super::certification_closeout_test_support::{
    admission_digest, admitted_basis_observation_plan, admitted_projection_consumption_plan,
    admitted_ready, intent_declaration, lower_runtime_envelope, projection_binding,
    projection_source, replay_gap_inputs, success,
};
use crate::domain_capabilities::authoring::{
    WorthQueryExplanationContributionAuthoring, WorthQueryInvariantCapabilityContributionAuthoring,
    WorthQuerySupportContributionAuthoring, WorthQueryWorkflowContributionAuthoring,
};
use crate::domain_capabilities::canonical_runtime::{
    materialize_intent_admission_support_traceability_report,
    materialize_intent_declaration_support_traceability_artifact,
    materialize_projection_consumption_contract, materialize_query_causal_inspection_artifact,
    materialize_query_invariant_catalog_registration_artifact,
    materialize_query_workflow_declaration, materialize_runtime_admission_decision,
    materialize_runtime_continuity_evidence,
};
use crate::domain_capabilities::certification::{
    certify_domain_capabilities_in, install_domain_capability_certification,
    worth_query_domain_capability_public_surface_inventory,
};
use crate::domain_capabilities::evaluate_requested_domain_capability_contribution;
use crate::runtime::{
    CausalEvidenceFamily, CausalInspectionMaterializationPolicy, CausalInspectionRedactionPolicy,
};

#[path = "certification_closeout_tests/certification_bundle_matches_equivalent_public_and_proof_outputs.rs"]
mod certification_bundle_matches_equivalent_public_and_proof_outputs;

#[test]
fn certification_bundle_preserves_support_scope_and_posture_distinction() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = intent_declaration();
    let admitted_plan = admitted_basis_observation_plan();
    let installation = install_domain_capability_certification(request_execution);
    let domain = installation.contributions();
    let bundle = certify_domain_capabilities_in(domain);
    let declaration_target = domain
        .intent_target(&declaration)
        .expect("installed contribution authority must remain current");
    let admitted_plan_target = domain
        .admitted_plan_target(&admitted_plan)
        .expect("installed contribution authority must remain current");

    let traceability = domain
        .for_intent_target(declaration_target.clone())
        .expect("certification target should belong to its installed domain")
        .supports_traceability("traceability.edge_split")
        .because("declaration-scoped support remains declaration scoped")
        .materialize()
        .expect("traceability support should materialize");
    let plain_support = success(
        materialize_intent_declaration_support_traceability_artifact(admitted_ready(
            WorthQuerySupportContributionAuthoring::declaration_support(
                "worth.spatial.traceability.edge_split",
                "declaration-scoped support remains declaration scoped",
            )
            .bind_to_installed_target(declaration_target),
        )),
    );
    let admitted_plan_support = success(materialize_intent_admission_support_traceability_report(
        admitted_ready(
            WorthQuerySupportContributionAuthoring::declaration_support(
                "worth.spatial.support.runtime_floor",
                "runtime floor remains explicitly supported",
            )
            .bind_to_installed_target(admitted_plan_target),
        ),
    ));

    assert_eq!(traceability.intent_name(), declaration.name());
    assert_ne!(
        traceability.materialization_digest(),
        plain_support.materialization_digest()
    );
    assert_ne!(
        traceability.materialization_digest(),
        admitted_plan_support.decision_support_traceability_digest()
    );
    assert_ne!(
        bundle
            .output_digest("support_artifact_digest")
            .expect("bundle should emit support artifact digest"),
        bundle
            .output_digest("support_traceability_digest")
            .expect("bundle should emit support traceability digest")
    );
}

#[test]
fn certification_bundle_matches_admission_continuity_and_aftermath_outputs() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let admitted_plan = admitted_basis_observation_plan();
    let projection_plan = admitted_projection_consumption_plan();
    let installation = install_domain_capability_certification(request_execution);
    let domain = installation.contributions();
    let bundle = certify_domain_capabilities_in(domain);
    let admitted_plan_target = domain
        .admitted_plan_target(&admitted_plan)
        .expect("installed contribution authority must remain current");
    let projection_plan_target = domain
        .admitted_plan_target(&projection_plan)
        .expect("installed contribution authority must remain current");

    let admission = domain
        .for_admitted_plan_target(admitted_plan_target.clone())
        .expect("certification target should belong to its installed domain")
        .advises("admission.routing_gap")
        .because("runtime routing still needs clarification")
        .materialize()
        .expect("admission common lane should materialize");
    let admission_proof = success(materialize_runtime_admission_decision(admitted_ready(
        crate::domain_capabilities::WorthQueryAdmissionContributionAuthoring::advisory(
            "worth.spatial.admission.routing_gap",
            "runtime routing still needs clarification",
        )
        .bind_to_installed_target(admitted_plan_target.clone()),
    )));
    assert_eq!(admission, admission_proof);
    assert_eq!(
        bundle.output_digest("admission_artifact_digest"),
        Some(admission_digest(&admission))
    );

    let continuity = domain
        .for_admitted_plan_target(admitted_plan_target.clone())
        .expect("certification target should belong to its installed domain")
        .preserves_continuity("continuity.edge_split", "edge:before", "edge:after")
        .because("edge split preserves one authoritative successor")
        .materialize()
        .expect("continuity common lane should materialize");
    let continuity_proof = success(materialize_runtime_continuity_evidence(admitted_ready(
        crate::domain_capabilities::WorthQueryContinuityContributionAuthoring::preserved_rebind(
            "edge:before",
            "edge:after",
            "worth.spatial.continuity.edge_split",
            "edge split preserves one authoritative successor",
        )
        .bind_to_installed_target(admitted_plan_target),
    )));
    assert_eq!(continuity, continuity_proof);
    assert_eq!(
        bundle.output_digest("continuity_artifact_digest"),
        Some(continuity.continuity_resolution_digest().as_str())
    );

    let aftermath = domain
        .for_admitted_plan_target(projection_plan_target.clone())
        .expect("certification target should belong to its installed domain")
        .consumes_projection_contract(
            "aftermath.projection_contract",
            crate::domain_capabilities::WorthQueryProjectionContractRequest::new(
                projection_source(),
                projection_binding(),
                crate::projection_consumption::ProjectMaterializedFacts::declare()
                    .display_field_path(
                        crate::projection_consumption::projection_fact_field_path_from_segments([
                            worth_foundational::facade::FieldKey::new("field")
                                .expect("projection fact field segment should admit"),
                            worth_foundational::facade::FieldKey::new("visible")
                                .expect("projection fact field segment should admit"),
                        ]),
                    ),
            ),
        )
        .because("projection aftermath should bind a stable contract")
        .materialize()
        .expect("aftermath common lane should materialize");
    let aftermath_proof = success(materialize_projection_consumption_contract(admitted_ready(
        crate::domain_capabilities::WorthQueryAftermathContributionAuthoring::consumes_projection_contract(
            "worth.spatial.aftermath.projection_contract",
            "projection aftermath should bind a stable contract",
            projection_source(),
            projection_binding(),
            crate::projection_consumption::ProjectMaterializedFacts::declare()
                .display_field_path(crate::projection_consumption::projection_fact_field_path_from_segments([worth_foundational::facade::FieldKey::new("field").expect("projection fact field segment should admit"), worth_foundational::facade::FieldKey::new("visible").expect("projection fact field segment should admit")])),
        )
        .bind_to_installed_target(projection_plan_target),
    )));
    assert_eq!(aftermath, aftermath_proof);
    assert_eq!(
        bundle.output_digest("aftermath_artifact_digest"),
        Some(aftermath.contract_digest())
    );
}

#[test]
fn certification_bundle_boundary_and_failure_outputs_track_live_surfaces() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = intent_declaration();
    let installation = install_domain_capability_certification(request_execution);
    let domain = installation.contributions();
    let bundle = certify_domain_capabilities_in(domain);
    let target = domain
        .intent_target(&declaration)
        .expect("installed contribution authority must remain current");
    let denial = match evaluate_requested_domain_capability_contribution(
        WorthQuerySupportContributionAuthoring::declaration_support(
            "worth.spatial.support.failure",
            "",
        )
        .bind_to_installed_target(target),
    ) {
        TransitionOutcome::Denied(denial) => denial,
        _ => panic!("expected typed denial"),
    };

    assert_eq!(
        bundle.output_digest("public_boundary_digest"),
        Some(
            worth_query_domain_capability_public_surface_inventory()
                .public_surface_digest()
                .as_str()
        )
    );
    assert_eq!(
        bundle.output_digest("failure_digest"),
        Some(denial.failure_for_reporting())
    );
}

#[test]
fn certification_bundle_populates_stable_width_and_slope_outputs() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let installation = install_domain_capability_certification(request_execution);
    let bundle = certify_domain_capabilities_in(installation.contributions());
    let slopes = bundle.slope_report();
    let contribution_width = slopes.counter_snapshot().contribution_width().to_string();
    let trace_width = slopes.counter_snapshot().trace_width().to_string();
    let category_width = slopes.counter_snapshot().category_width().to_string();
    let support_width = slopes.counter_snapshot().support_width().to_string();

    assert_eq!(
        bundle.output_digest("counter_snapshot"),
        Some(slopes.counter_snapshot().digest())
    );
    assert_eq!(
        bundle.output_digest("contribution_width"),
        Some(contribution_width.as_str())
    );
    assert_eq!(
        bundle.output_digest("trace_width"),
        Some(trace_width.as_str())
    );
    assert_eq!(
        bundle.output_digest("category_width"),
        Some(category_width.as_str())
    );
    assert_eq!(
        bundle.output_digest("support_width"),
        Some(support_width.as_str())
    );
    assert_eq!(
        bundle.output_digest("contribution_materialization_slope_digest"),
        Some(slopes.contribution_materialization_slope_digest())
    );
    assert_eq!(
        bundle.output_digest("trace_materialization_slope_digest"),
        Some(slopes.trace_materialization_slope_digest())
    );
    assert_eq!(
        bundle.output_digest("category_materialization_slope_digest"),
        Some(slopes.category_materialization_slope_digest())
    );
    assert_eq!(
        bundle.output_digest("support_materialization_slope_digest"),
        Some(slopes.support_materialization_slope_digest())
    );
    assert!(slopes.counter_snapshot().contribution_width() > 0);
    assert!(slopes.counter_snapshot().trace_width() > 0);
    assert!(slopes.counter_snapshot().category_width() >= 7);
    assert!(slopes.counter_snapshot().support_width() >= 2);
}
