use super::certification::install_domain_capability_certification;
use super::certification_closeout_test_support::{
    admitted_basis_observation_plan, admitted_ready, intent_declaration, lower_runtime_envelope,
};
use super::{
    materialize_canonical_admission_artifact, materialize_canonical_aftermath_artifact,
    materialize_canonical_continuity_artifact, materialize_canonical_explanation_artifact,
    materialize_canonical_invariant_capability_artifact,
    materialize_canonical_support_traceability_artifact, materialize_canonical_workflow_artifact,
    WorthQueryAdmissionContributionAuthoring, WorthQueryAftermathContributionAuthoring,
    WorthQueryContinuityContributionAuthoring, WorthQueryExplanationContributionAuthoring,
    WorthQueryInvariantCapabilityContributionAuthoring, WorthQuerySupportContributionAuthoring,
    WorthQueryWorkflowContributionAuthoring,
};

#[test]
fn every_contribution_family_materializes_under_one_installed_authority() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = intent_declaration();
    let admitted_plan = admitted_basis_observation_plan();
    let lower_runtime = lower_runtime_envelope("installed-authority-family-matrix");
    let installation = install_domain_capability_certification(request_execution);
    let domain = installation.contributions();
    let declaration_target = domain
        .intent_target(&declaration)
        .expect("installed contribution authority must remain current");
    let admitted_target = domain
        .admitted_plan_target(&admitted_plan)
        .expect("installed contribution authority must remain current");
    let lower_runtime_target = domain
        .lower_runtime_target(&lower_runtime)
        .expect("installed contribution authority must remain current");
    let expected_authority = declaration_target.authority().authority_identity().clone();
    let expected_package = declaration_target.authority().package_identity().clone();
    let expected_world = declaration_target.world_identity().clone();

    macro_rules! assert_installed_authority {
        ($artifact:expr) => {{
            let artifact = $artifact;
            let authority = artifact
                .installed_authority()
                .expect("installed contribution materialization must retain authority");
            assert_eq!(authority.authority_identity(), &expected_authority);
            assert_eq!(authority.package_identity(), &expected_package);
            assert_eq!(artifact.installed_world_identity(), Some(&expected_world));
        }};
    }

    assert_installed_authority!(materialize_canonical_admission_artifact(admitted_ready(
        WorthQueryAdmissionContributionAuthoring::advisory(
            "worth.spatial.admission.matrix",
            "admission remains installed",
        )
        .bind_to_installed_target(admitted_target.clone()),
    )));
    assert_installed_authority!(materialize_canonical_support_traceability_artifact(
        admitted_ready(
            WorthQuerySupportContributionAuthoring::declaration_traceability(
                "worth.spatial.support.matrix",
                "support remains installed",
            )
            .bind_to_installed_target(declaration_target.clone()),
        ),
    ));
    assert_installed_authority!(materialize_canonical_invariant_capability_artifact(
        admitted_ready(
            WorthQueryInvariantCapabilityContributionAuthoring::capability_gap(
                "worth.spatial.invariant.matrix",
                "invariant posture remains installed",
            )
            .bind_to_installed_target(declaration_target.clone()),
        ),
    ));
    assert_installed_authority!(materialize_canonical_workflow_artifact(admitted_ready(
        WorthQueryWorkflowContributionAuthoring::preview_only(
            "worth.spatial.workflow.matrix",
            "workflow remains installed",
        )
        .bind_to_installed_target(declaration_target.clone()),
    )));
    assert_installed_authority!(materialize_canonical_continuity_artifact(admitted_ready(
        WorthQueryContinuityContributionAuthoring::preserved(
            "worth.spatial.continuity.matrix",
            "continuity remains installed",
        )
        .bind_to_installed_target(declaration_target),
    )));
    assert_installed_authority!(materialize_canonical_aftermath_artifact(admitted_ready(
        WorthQueryAftermathContributionAuthoring::establishes_fact(
            "worth.spatial.aftermath.matrix",
            "aftermath remains installed",
        )
        .bind_to_installed_target(admitted_target),
    )));
    assert_installed_authority!(materialize_canonical_explanation_artifact(admitted_ready(
        WorthQueryExplanationContributionAuthoring::requires_context(
            "worth.spatial.explanation.matrix",
            "explanation remains installed",
        )
        .bind_to_installed_target(lower_runtime_target),
    )));
}
