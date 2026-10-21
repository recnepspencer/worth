use crate::runtime::{
    WorthQueryGraphReadAccessAdmission, WorthQueryGraphReadAccessAdmissionPosture,
    WorthQueryGraphReadAccessDenialKind, WorthQueryGraphReadAccessRequirementKind,
    WorthQueryGraphReadRequiredCapabilityOwner,
};

use crate::runtime::tests::graph_read_access::support::graph_index_inventory::runtime_profiles::{
    default_graph_support_workspace, profile_requiring_graph_access_capability_registration,
    profile_requiring_store_backed_graph_index, profile_with_ephemeral_graph_support,
    profile_with_graph_support_temporarily_unavailable, profile_without_graph_support,
    workspace_with_graph_support,
};
use crate::runtime::tests::graph_read_access::support::graph_read_access::read_surface_declarations::graph_access_family;
use crate::runtime::tests::graph_read_access::support::graph_read_access_cost_model::{
    dense_traversal_family, projection_only_family, simple_traversal_family, workspace,
};

#[test]
fn closeout_matrix_certifies_representative_access_postures() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_posture(
        request_execution,
        "closeout.matrix.inline",
        |workspace| projection_only_family(workspace, "closeout-inline"),
        WorthQueryGraphReadAccessAdmissionPosture::InlineIndexed,
        None,
        None,
    );

    let mut traversal_workspace = default_graph_support_workspace(
        request_execution,
        "graph-read-access.closeout.matrix.traversal",
    );
    let traversal = graph_access_family(&mut traversal_workspace, "closeout-traversal");
    assert_admission_posture(
        &admission_for_family(&mut traversal_workspace, &traversal),
        WorthQueryGraphReadAccessAdmissionPosture::InlineIndexed,
        None,
        None,
    );

    let mut ephemeral_workspace = workspace_with_graph_support(
        request_execution,
        "graph-read-access.closeout.matrix.ephemeral",
        profile_with_ephemeral_graph_support(
            WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
        ),
    );
    let ephemeral = simple_traversal_family(&mut ephemeral_workspace, "closeout-ephemeral");
    assert_admission_posture(
        &admission_for_family(&mut ephemeral_workspace, &ephemeral),
        WorthQueryGraphReadAccessAdmissionPosture::BoundedEphemeralIndex,
        None,
        None,
    );

    assert_posture(
        request_execution,
        "closeout.matrix.dense-supported",
        |workspace| dense_traversal_family(workspace, "closeout-dense-supported"),
        WorthQueryGraphReadAccessAdmissionPosture::InlineIndexed,
        None,
        None,
    );

    assert_support_profile_posture(
        request_execution,
        profile_without_graph_support(
            WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
        ),
        WorthQueryGraphReadAccessAdmissionPosture::Denied,
        Some(WorthQueryGraphReadAccessDenialKind::UnsupportedGraphIndexSupport),
        Some(WorthQueryGraphReadAccessAdmissionPosture::Denied),
    );

    assert_support_profile_posture(
        request_execution,
        profile_with_graph_support_temporarily_unavailable(
            WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
        ),
        WorthQueryGraphReadAccessAdmissionPosture::Denied,
        Some(WorthQueryGraphReadAccessDenialKind::RequiredAsyncMaterialization),
        Some(WorthQueryGraphReadAccessAdmissionPosture::AsyncMaterializationRequired),
    );

    assert_support_profile_posture(
        request_execution,
        profile_requiring_store_backed_graph_index(
            WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
        ),
        WorthQueryGraphReadAccessAdmissionPosture::Denied,
        Some(WorthQueryGraphReadAccessDenialKind::RequiredPersistentIndex),
        Some(WorthQueryGraphReadAccessAdmissionPosture::PersistentIndexRequired),
    );

    assert_support_profile_posture(
        request_execution,
        profile_requiring_graph_access_capability_registration(
            WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
        ),
        WorthQueryGraphReadAccessAdmissionPosture::Denied,
        Some(WorthQueryGraphReadAccessDenialKind::RequiredAccessCapabilityRegistration),
        Some(WorthQueryGraphReadAccessAdmissionPosture::AccessCapabilityRegistrationRequired),
    );
}

#[test]
fn closeout_matrix_every_requirement_has_case_and_support_owner() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "graph-read-access.closeout.matrix.requirements",
    );
    let family = simple_traversal_family(&mut workspace, "closeout-requirements");
    let admission = admission_for_family(&mut workspace, &family);

    for requirement_row in admission.requirement_set().rows() {
        let case = admission
            .case_registry()
            .case_for_requirement_kind(requirement_row.kind())
            .expect("every requirement row must map to an access case");
        assert_eq!(case.requirement_kind(), requirement_row.kind());
    }

    assert!(admission
        .graph_index_inventory_match_report()
        .matches()
        .iter()
        .all(
            |inventory_match| inventory_match.required_capability_owner()
                == &WorthQueryGraphReadRequiredCapabilityOwner::QueryRuntime
        ));
}

fn assert_posture(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace_name: &str,
    family: impl FnOnce(
        &mut crate::runtime::WorthQueryWorkspace,
    ) -> crate::runtime::WorthQueryReadFamily,
    expected_posture: WorthQueryGraphReadAccessAdmissionPosture,
    expected_denial: Option<WorthQueryGraphReadAccessDenialKind>,
    expected_suggested_posture: Option<WorthQueryGraphReadAccessAdmissionPosture>,
) {
    let mut workspace = workspace(request_execution, workspace_name);
    let family = family(&mut workspace);
    assert_admission_posture(
        &admission_for_family(&mut workspace, &family),
        expected_posture,
        expected_denial,
        expected_suggested_posture,
    );
}

fn assert_support_profile_posture(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    support_profile: crate::runtime::WorthQueryRuntimeSupportProfile,
    expected_posture: WorthQueryGraphReadAccessAdmissionPosture,
    expected_denial: Option<WorthQueryGraphReadAccessDenialKind>,
    expected_suggested_posture: Option<WorthQueryGraphReadAccessAdmissionPosture>,
) {
    let mut workspace = workspace_with_graph_support(
        request_execution,
        "graph-read-access.closeout.matrix.support",
        support_profile,
    );
    let family = simple_traversal_family(&mut workspace, "closeout-support-posture");
    assert_admission_posture(
        &admission_for_family(&mut workspace, &family),
        expected_posture,
        expected_denial,
        expected_suggested_posture,
    );
}

fn admission_for_family(
    workspace: &mut crate::runtime::WorthQueryWorkspace,
    family: &crate::runtime::WorthQueryReadFamily,
) -> WorthQueryGraphReadAccessAdmission {
    workspace
        .read_family_intent(family)
        .review()
        .expect("closeout read should review")
        .graph_read_access_admission()
        .expect("closeout read should produce access admission")
}

fn assert_admission_posture(
    admission: &WorthQueryGraphReadAccessAdmission,
    expected_posture: WorthQueryGraphReadAccessAdmissionPosture,
    expected_denial: Option<WorthQueryGraphReadAccessDenialKind>,
    expected_suggested_posture: Option<WorthQueryGraphReadAccessAdmissionPosture>,
) {
    assert_eq!(admission.posture(), &expected_posture);
    assert_eq!(
        admission.denial().map(|denial| denial.kind().clone()),
        expected_denial
    );
    assert_eq!(
        admission
            .denial()
            .map(|denial| denial.suggested_posture().clone()),
        expected_suggested_posture
    );
}
