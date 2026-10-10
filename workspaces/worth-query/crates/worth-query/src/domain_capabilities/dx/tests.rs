use super::test_support::success;
use super::{
    WorthQueryDomainCapabilityMaterializationError, WorthQueryDomainCapabilityOutcomeKind,
};
use crate::domain_capabilities::certification::install_domain_capability_certification;
use crate::domain_capabilities::{
    admit_eligible_domain_capability_contribution,
    evaluate_requested_domain_capability_contribution, materialize_canonical_admission_artifact,
    materialize_intent_declaration_support_traceability_artifact,
    materialize_query_invariant_catalog_registration_artifact,
    materialize_query_preview_workflow_artifact, materialize_query_workflow_declaration,
    materialize_runtime_admission_decision, materialize_runtime_continuity_evidence,
    prepare_admitted_domain_capability_contribution_for_materialization,
    WorthQueryAdmissionContributionAuthoring, WorthQueryContinuityContributionAuthoring,
    WorthQueryInvariantCapabilityContributionAuthoring, WorthQuerySupportContributionAuthoring,
    WorthQueryWorkflowContributionAuthoring,
};
use crate::runtime::{InvariantCatalog, WorthQueryIntentDeclaration};

#[test]
fn common_intent_support_lane_matches_proof_lane_materialization() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = intent_declaration("intent-support");
    let installation = install_domain_capability_certification(request_execution);
    let domain = installation.contributions();
    let target = domain
        .intent_target(&declaration)
        .expect("installed contribution authority must remain current");

    let common = domain
        .for_intent_target(target.clone())
        .expect("certification target should belong to its installed domain")
        .supports_capability("graph.face_inner_loop_insertion")
        .because("topology substrate is available")
        .materialize()
        .expect("common support lane should materialize");

    let proof_requested = WorthQuerySupportContributionAuthoring::declaration_support(
        "worth.spatial.graph.face_inner_loop_insertion",
        "topology substrate is available",
    )
    .bind_to_installed_target(target);
    let proof_admitted = success(admit_eligible_domain_capability_contribution(success(
        evaluate_requested_domain_capability_contribution(proof_requested),
    )));
    let proof_target = proof_admitted.payload().target().clone();
    let proof = materialize_intent_declaration_support_traceability_artifact(success(
        prepare_admitted_domain_capability_contribution_for_materialization(
            proof_admitted,
            proof_target,
        ),
    ));

    assert_eq!(common, success(proof));
}

#[test]
fn common_intent_advisory_lane_matches_proof_lane_materialization() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = intent_declaration("intent-advisory");
    let installation = install_domain_capability_certification(request_execution);
    let domain = installation.contributions();
    let target = domain
        .intent_target(&declaration)
        .expect("installed contribution authority must remain current");

    let common = domain
        .for_intent_target(target.clone())
        .expect("certification target should belong to its installed domain")
        .advises("arbitration.requires_clarification")
        .because("multiple spatial candidates remain admissible")
        .materialize()
        .expect("declaration-bound advisory lane should materialize");

    let proof_requested = WorthQueryAdmissionContributionAuthoring::advisory(
        "worth.spatial.arbitration.requires_clarification",
        "multiple spatial candidates remain admissible",
    )
    .bind_to_installed_target(target);
    let proof_admitted = success(admit_eligible_domain_capability_contribution(success(
        evaluate_requested_domain_capability_contribution(proof_requested),
    )));
    let proof_target = proof_admitted.payload().target().clone();
    let proof = materialize_canonical_admission_artifact(success(
        prepare_admitted_domain_capability_contribution_for_materialization(
            proof_admitted,
            proof_target,
        ),
    ));

    assert_eq!(common.canonical_family(), proof.canonical_family());
    assert_eq!(
        common.semantic_identity_for_reporting(),
        proof.semantic_identity_for_reporting()
    );
    assert_eq!(
        common.materialization_digest(),
        proof.materialization_digest()
    );
}

#[test]
fn common_admission_lane_matches_proof_lane_materialization() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let plan = admitted_basis_observation_plan();
    let installation = install_domain_capability_certification(request_execution);
    let domain = installation.contributions();
    let target = domain
        .admitted_plan_target(&plan)
        .expect("installed contribution authority must remain current");

    let common = domain
        .for_admitted_plan_target(target.clone())
        .expect("certification target should belong to its installed domain")
        .advises("arbitration.requires_clarification")
        .because("multiple spatial candidates remain admissible")
        .materialize()
        .expect("common advisory lane should materialize");

    let proof_requested = WorthQueryAdmissionContributionAuthoring::advisory(
        "worth.spatial.arbitration.requires_clarification",
        "multiple spatial candidates remain admissible",
    )
    .bind_to_installed_target(target);
    let proof_admitted = success(admit_eligible_domain_capability_contribution(success(
        evaluate_requested_domain_capability_contribution(proof_requested),
    )));
    let proof_target = proof_admitted.payload().target().clone();
    let proof = materialize_runtime_admission_decision(success(
        prepare_admitted_domain_capability_contribution_for_materialization(
            proof_admitted,
            proof_target,
        ),
    ));

    assert_eq!(common, success(proof));
}

#[test]
fn common_intent_preview_inspection_lane_matches_proof_lane_materialization() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = intent_declaration("intent-preview");
    let installation = install_domain_capability_certification(request_execution);
    let domain = installation.contributions();
    let target = domain
        .intent_target(&declaration)
        .expect("installed contribution authority must remain current");

    let common = domain
        .for_intent_target(target.clone())
        .expect("certification target should belong to its installed domain")
        .inspects_query_preview(
            "topology.preview_conflict",
            crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                "preview-session:42",
            ),
        )
        .because("preview should stay read-only while topology is inspected")
        .materialize()
        .expect("preview inspection lane should materialize");

    let proof_requested = WorthQueryWorkflowContributionAuthoring::preview_only_query_inspection(
        "worth.spatial.topology.preview_conflict",
        "preview should stay read-only while topology is inspected",
        crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
            "preview-session:42",
        ),
    )
    .bind_to_installed_target(target);
    let proof_admitted = success(admit_eligible_domain_capability_contribution(success(
        evaluate_requested_domain_capability_contribution(proof_requested),
    )));
    let proof_target = proof_admitted.payload().target().clone();
    let proof = materialize_query_preview_workflow_artifact(success(
        prepare_admitted_domain_capability_contribution_for_materialization(
            proof_admitted,
            proof_target,
        ),
    ));

    assert_eq!(common, success(proof));
}

#[path = "tests/common_intent_preview_mutation_lane_matches_proof_lane_materiali.rs"]
mod common_intent_preview_mutation_lane_matches_proof_lane_materiali;

#[test]
fn common_continuity_lane_matches_proof_lane_materialization() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let plan = admitted_basis_observation_plan();
    let installation = install_domain_capability_certification(request_execution);
    let domain = installation.contributions();
    let target = domain
        .admitted_plan_target(&plan)
        .expect("installed contribution authority must remain current");

    let common = domain
        .for_admitted_plan_target(target.clone())
        .expect("certification target should belong to its installed domain")
        .preserves_continuity("identity.edge_split", "edge:before", "edge:after")
        .because("edge split replaces one edge with one canonical successor")
        .materialize()
        .expect("common continuity lane should materialize");

    let proof_requested = WorthQueryContinuityContributionAuthoring::preserved_rebind(
        "edge:before",
        "edge:after",
        "worth.spatial.identity.edge_split",
        "edge split replaces one edge with one canonical successor",
    )
    .bind_to_installed_target(target);
    let proof_admitted = success(admit_eligible_domain_capability_contribution(success(
        evaluate_requested_domain_capability_contribution(proof_requested),
    )));
    let proof_target = proof_admitted.payload().target().clone();
    let proof = materialize_runtime_continuity_evidence(success(
        prepare_admitted_domain_capability_contribution_for_materialization(
            proof_admitted,
            proof_target,
        ),
    ));

    assert_eq!(common, success(proof));
}

#[test]
fn common_invariant_registration_lane_matches_proof_lane_materialization() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = intent_declaration("intent-invariant");
    let invariant_catalog = InvariantCatalog::default();
    let installation = install_domain_capability_certification(request_execution);
    let domain = installation.contributions();
    let target = domain
        .intent_target(&declaration)
        .expect("installed contribution authority must remain current");

    let common = domain
        .for_intent_target(target.clone())
        .expect("certification target should belong to its installed domain")
        .register_invariant_catalog("spatial.non_manifold_edge_split", invariant_catalog.clone())
        .because("geometry kernel must reject non-manifold edge splits")
        .materialize()
        .expect("common invariant lane should materialize");

    let proof_requested =
        WorthQueryInvariantCapabilityContributionAuthoring::invariant_registration(
            invariant_catalog,
            "worth.spatial.spatial.non_manifold_edge_split",
            "geometry kernel must reject non-manifold edge splits",
        )
        .bind_to_installed_target(target);
    let proof_admitted = success(admit_eligible_domain_capability_contribution(success(
        evaluate_requested_domain_capability_contribution(proof_requested),
    )));
    let proof_target = proof_admitted.payload().target().clone();
    let proof = materialize_query_invariant_catalog_registration_artifact(success(
        prepare_admitted_domain_capability_contribution_for_materialization(
            proof_admitted,
            proof_target,
        ),
    ));

    assert_eq!(common, success(proof));
}

#[test]
fn checked_lane_preserves_denied_outcome_metadata_and_typed_error() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = intent_declaration("intent-denial");

    let checked = install_domain_capability_certification(request_execution)
        .contributions()
        .for_intent(&declaration)
        .expect("installed contribution authority must remain current")
        .supports_capability("graph.face_inner_loop_insertion")
        .because("")
        .try_materialize();

    assert_eq!(
        checked.kind(),
        WorthQueryDomainCapabilityOutcomeKind::Denied
    );
    assert_eq!(checked.category(), "support-traceability");
    assert_eq!(checked.semantic_posture(), "declaration-support");
    assert_eq!(
        checked.target_kind(),
        crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind::IntentDeclaration
    );
    assert!(checked.denial().is_some());

    let error = install_domain_capability_certification(request_execution)
        .contributions()
        .for_intent(&declaration)
        .expect("installed contribution authority must remain current")
        .supports_capability("graph.face_inner_loop_insertion")
        .because("")
        .materialize()
        .expect_err("empty detail should deny");
    assert!(matches!(
        error,
        WorthQueryDomainCapabilityMaterializationError::Denied(_)
    ));
}

#[test]
fn checked_intent_admission_lane_preserves_declaration_bound_metadata() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = intent_declaration("intent-admission-denial");

    let checked = install_domain_capability_certification(request_execution)
        .contributions()
        .for_intent(&declaration)
        .expect("installed contribution authority must remain current")
        .advises("arbitration.requires_clarification")
        .because("")
        .try_materialize();

    assert_eq!(
        checked.kind(),
        WorthQueryDomainCapabilityOutcomeKind::Denied
    );
    assert_eq!(checked.category(), "admission");
    assert_eq!(checked.semantic_posture(), "advisory");
    assert_eq!(
        checked.target_kind(),
        crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind::IntentDeclaration
    );
    assert!(checked.denial().is_some());
    assert!(matches!(
        checked.transition_outcome(),
        worth_proof::TransitionOutcome::Denied(_)
    ));
}

fn intent_declaration(name: &str) -> WorthQueryIntentDeclaration {
    WorthQueryIntentDeclaration::strategy_commit(
        name,
        "spatial.commit",
        "1",
        "geometry.patch",
        crate::runtime::WorthQueryIntentInput::object([(
            "edge",
            crate::runtime::WorthQueryIntentInput::string("e-1"),
        )]),
    )
}

fn admitted_basis_observation_plan() -> crate::runtime::WorthQueryAdmittedIntentPlan {
    let request =
        crate::intent_admission::WorthQueryRawIntentAdmissionRequest::basis_observation_lane(
            crate::basis_lifecycle::RawBasisIntent::CurrentHead,
        )
        .expect("basis-observation request should build");

    match crate::intent_admission::admit_runtime_intent_request(request) {
        crate::intent_admission::WorthQueryIntentAdmissionDecision::Admitted(plan) => plan,
        other => panic!("expected admitted basis-observation plan, got {other:?}"),
    }
}
