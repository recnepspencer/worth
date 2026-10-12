use crate::domain_capabilities::{
    WorthQueryExplanationContributionAuthoring, WorthQuerySupportContributionAuthoring,
};
use crate::grouped_authoring::{
    worth_query_grouped_declaration_checked_on_handle,
    worth_query_grouped_orchestration_checked_on_handle,
    worth_query_grouped_orchestration_proof_on_handle, WorthQueryGroupedAtomicity,
    WorthQueryGroupedContinuityAssumption, WorthQueryGroupedDeclarationChecked,
    WorthQueryGroupedDeclarationInput, WorthQueryGroupedIntent, WorthQueryGroupedMemberRole,
    WorthQueryGroupedOrchestrationChecked, WorthQueryGroupedSharedPostureClaim,
};

use super::support::{admitted_handle, GeometryInput, RequiredIntentGeometryInput};

#[test]
fn helper_grouped_declaration_matches_generic_grouped_path() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "main");
    let helper_input = handle
        .geometry_helpers()
        .local_neighborhood_for_active_face_selection(GeometryInput::new("face-a"))
        .with_member(GeometryInput::new("face-b"))
        .with_shared_rationale("split the local neighborhood");
    let generic_input =
        WorthQueryGroupedDeclarationInput::local_neighborhood(GeometryInput::new("face-a"))
            .with_member(GeometryInput::new("face-b"))
            .with_shared_rationale("split the local neighborhood");

    let helper = handle
        .geometry_helpers()
        .declare_local_neighborhood_for_active_face_selection_checked(helper_input);
    let generic = worth_query_grouped_declaration_checked_on_handle(&handle, generic_input);

    match (helper, generic) {
        (
            WorthQueryGroupedDeclarationChecked::Bound(left),
            WorthQueryGroupedDeclarationChecked::Bound(right),
        ) => {
            assert_eq!(left.group_digest(), right.group_digest());
            assert_eq!(left.shared_rationale(), right.shared_rationale());
            assert_eq!(left.atomicity(), right.atomicity());
            assert_eq!(left.grouping_intent(), right.grouping_intent());
            assert_eq!(left.continuity_assumption(), right.continuity_assumption());
            assert_eq!(left.shared_posture_claims(), right.shared_posture_claims());
            assert_eq!(left.aspect_record(), right.aspect_record());
            assert_eq!(left.aspect_participation(), right.aspect_participation());
            assert_eq!(left.members().len(), right.members().len());
            assert_eq!(left.members()[0].role(), WorthQueryGroupedMemberRole::Seed);
            assert_eq!(
                left.members()[1].role(),
                WorthQueryGroupedMemberRole::Member
            );
            assert_eq!(
                left.members()[0].aspect_record(),
                right.members()[0].aspect_record()
            );
            assert_eq!(
                left.members()[1].aspect_record(),
                right.members()[1].aspect_record()
            );
        }
        _ => panic!("expected grouped declaration parity"),
    }
}

#[test]
fn grouped_declaration_digest_changes_with_semantic_posture() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "main");
    let baseline = match worth_query_grouped_declaration_checked_on_handle(
        &handle,
        WorthQueryGroupedDeclarationInput::local_neighborhood(GeometryInput::new("face-a"))
            .with_member(GeometryInput::new("face-b")),
    ) {
        WorthQueryGroupedDeclarationChecked::Bound(value) => value,
        _ => panic!("expected grouped declaration admission"),
    };
    let richer = match worth_query_grouped_declaration_checked_on_handle(
        &handle,
        WorthQueryGroupedDeclarationInput::local_neighborhood(GeometryInput::new("face-a"))
            .with_member(GeometryInput::new("face-b"))
            .with_atomicity(WorthQueryGroupedAtomicity::Atomic)
            .with_grouping_intent(WorthQueryGroupedIntent::Authoritative)
            .with_continuity_assumption(WorthQueryGroupedContinuityAssumption::PreserveNeighborhood)
            .with_shared_posture_claim(WorthQueryGroupedSharedPostureClaim::SharedSelectionFocus),
    ) {
        WorthQueryGroupedDeclarationChecked::Bound(value) => value,
        _ => panic!("expected grouped declaration admission"),
    };

    assert_ne!(baseline.group_digest(), richer.group_digest());
    assert_eq!(richer.atomicity(), WorthQueryGroupedAtomicity::Atomic);
    assert_eq!(
        richer.grouping_intent(),
        WorthQueryGroupedIntent::Authoritative
    );
    assert_eq!(
        richer.continuity_assumption(),
        WorthQueryGroupedContinuityAssumption::PreserveNeighborhood
    );
    assert_eq!(
        richer.shared_posture_claims(),
        &[WorthQueryGroupedSharedPostureClaim::SharedSelectionFocus]
    );
}

#[test]
fn grouped_orchestration_matches_generic_checked_lowering() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "main");
    let declaration = handle
        .geometry_helpers()
        .declare_local_neighborhood_for_active_face_selection(
            handle
                .geometry_helpers()
                .local_neighborhood_for_active_face_selection(GeometryInput::new("face-a"))
                .with_member(GeometryInput::new("face-b")),
        )
        .unwrap();

    let helper = handle
        .geometry_helpers()
        .orchestrate_local_neighborhood_for_active_face_selection_checked(declaration.clone());
    let generic = worth_query_grouped_orchestration_checked_on_handle(&handle, declaration);

    match (helper, generic) {
        (
            WorthQueryGroupedOrchestrationChecked::Bound(left),
            WorthQueryGroupedOrchestrationChecked::Bound(right),
        ) => {
            assert_eq!(left.orchestration_digest(), right.orchestration_digest());
            assert_eq!(
                left.declaration().aspect_record(),
                right.declaration().aspect_record()
            );
            assert_eq!(
                left.declaration().aspect_participation(),
                right.declaration().aspect_participation()
            );
            assert_eq!(
                left.member_envelopes().len(),
                right.member_envelopes().len()
            );
            assert_eq!(
                left.member_envelopes()[0].aspect_record(),
                right.member_envelopes()[0].aspect_record()
            );
            assert_eq!(
                left.member_envelopes()[0].role(),
                right.member_envelopes()[0].role()
            );
            assert_eq!(
                left.member_envelopes()[0].envelope().declaration_digest(),
                right.member_envelopes()[0].envelope().declaration_digest()
            );
        }
        _ => panic!("expected grouped orchestration parity"),
    }
}

#[test]
fn grouped_route_receipt_and_envelope_preserve_member_aspect_witness() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "main");
    let declaration = handle
        .geometry_helpers()
        .declare_local_neighborhood_for_active_face_selection(
            handle
                .geometry_helpers()
                .local_neighborhood_for_active_face_selection(GeometryInput::new("face-a"))
                .with_member(GeometryInput::new("face-b")),
        )
        .unwrap();

    let route_checked = handle.grouped_route_checked(declaration.clone());
    let route_proof = handle.grouped_route_proof(declaration.clone());
    let receipt_checked = handle.grouped_receipt_checked(declaration.clone());
    let receipt_proof = handle.grouped_receipt_proof(declaration.clone());
    let envelope_checked = handle.grouped_envelope_checked(declaration.clone());
    let envelope_proof = handle.grouped_envelope_proof(declaration);

    assert_eq!(
        route_checked.declaration().group_digest(),
        route_proof.declaration().group_digest()
    );
    assert_eq!(
        route_checked.members()[0].aspect_record(),
        route_proof.members()[0].aspect_record()
    );
    assert_eq!(
        receipt_checked.members()[1].aspect_record(),
        receipt_proof.members()[1].aspect_record()
    );
    assert_eq!(
        envelope_checked.members()[0].aspect_record(),
        envelope_proof.members()[0].aspect_record()
    );
}

#[test]
fn grouped_contribution_helper_matches_generic_grouped_lowering() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "main");
    let helper_input = handle
        .geometry_helpers()
        .local_neighborhood_for_active_face_selection(GeometryInput::new("face-a"))
        .with_member(GeometryInput::new("face-b"))
        .with_shared_support_contribution(
            WorthQuerySupportContributionAuthoring::declaration_support(
                "support.shared",
                "shared grouped support",
            ),
        )
        .with_member_contribution(
            1,
            crate::contribution_composed_orchestration::WorthQueryContributionIntent::explanation(
                WorthQueryExplanationContributionAuthoring::explains_fallback(
                    "explain.member",
                    "member-local fallback explanation",
                ),
            ),
        );
    let generic_input = handle
        .geometry_helpers()
        .local_neighborhood_for_active_face_selection(GeometryInput::new("face-a"))
        .with_member(GeometryInput::new("face-b"))
        .with_shared_support_contribution(
            WorthQuerySupportContributionAuthoring::declaration_support(
                "support.shared",
                "shared grouped support",
            ),
        )
        .with_member_contribution(
            1,
            crate::contribution_composed_orchestration::WorthQueryContributionIntent::explanation(
                WorthQueryExplanationContributionAuthoring::explains_fallback(
                    "explain.member",
                    "member-local fallback explanation",
                ),
            ),
        );

    let helper = match handle
        .geometry_helpers()
        .grouped_contributions_for_active_face_selection_checked(helper_input)
    {
        Ok(value) => value,
        Err(_) => panic!("grouped helper contributions should admit"),
    };
    let generic = match handle.grouped_contributions_checked(generic_input) {
        Ok(value) => value,
        Err(_) => panic!("generic grouped contributions should admit"),
    };

    assert_eq!(
        helper.declaration().group_digest(),
        generic.declaration().group_digest()
    );
    assert_eq!(helper.members().len(), generic.members().len());
    assert_eq!(
        helper.members()[0].0.aspect_record(),
        generic.members()[0].0.aspect_record()
    );
    assert_eq!(
        helper.members()[0].1.composition_for_reporting(),
        generic.members()[0].1.composition_for_reporting()
    );
    assert_eq!(
        helper.members()[1].1.composition_for_reporting(),
        generic.members()[1].1.composition_for_reporting()
    );
}

#[test]
fn grouped_checked_and_proof_preserve_same_member_aspect_witness() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "main");
    let declaration = handle
        .geometry_helpers()
        .declare_local_neighborhood_for_active_face_selection(
            handle
                .geometry_helpers()
                .local_neighborhood_for_active_face_selection(GeometryInput::new("face-a"))
                .with_member(GeometryInput::new("face-b")),
        )
        .unwrap();

    let checked = worth_query_grouped_orchestration_checked_on_handle(&handle, declaration.clone());
    let proof = worth_query_grouped_orchestration_proof_on_handle(&handle, declaration);

    match (checked, proof.into_checked()) {
        (
            WorthQueryGroupedOrchestrationChecked::Bound(left),
            WorthQueryGroupedOrchestrationChecked::Bound(right),
        ) => {
            assert_eq!(left.orchestration_digest(), right.orchestration_digest());
            assert_eq!(
                left.declaration().aspect_record(),
                right.declaration().aspect_record()
            );
            assert_eq!(
                left.declaration().aspect_participation(),
                right.declaration().aspect_participation()
            );
            assert_eq!(
                left.member_envelopes()[0].aspect_record(),
                right.member_envelopes()[0].aspect_record()
            );
            assert_eq!(
                left.member_envelopes()[1].aspect_record(),
                right.member_envelopes()[1].aspect_record()
            );
        }
        _ => panic!("expected grouped checked/proof parity"),
    }
}

#[test]
fn grouped_member_stop_checked_and_proof_preserve_same_member_witness() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "main");
    let declaration = match worth_query_grouped_declaration_checked_on_handle(
        &handle,
        WorthQueryGroupedDeclarationInput::local_neighborhood(RequiredIntentGeometryInput::new(
            "edge-a",
        ))
        .with_member(RequiredIntentGeometryInput::new("edge-b")),
    ) {
        WorthQueryGroupedDeclarationChecked::Bound(value) => value,
        WorthQueryGroupedDeclarationChecked::MemberStopped(_) => {
            panic!("expected grouped declaration admission")
        }
    };

    let checked = worth_query_grouped_orchestration_checked_on_handle(&handle, declaration.clone());
    let proof = worth_query_grouped_orchestration_proof_on_handle(&handle, declaration);

    match (checked, proof.into_checked()) {
        (
            WorthQueryGroupedOrchestrationChecked::MemberStopped(left),
            WorthQueryGroupedOrchestrationChecked::MemberStopped(right),
        ) => {
            assert_eq!(left.member_index(), right.member_index());
            assert_eq!(left.member_role(), right.member_role());
            assert_eq!(left.member_aspect_record(), right.member_aspect_record());
            assert_eq!(
                left.declaration().aspect_record(),
                right.declaration().aspect_record()
            );
            assert_eq!(
                left.declaration().aspect_participation(),
                right.declaration().aspect_participation()
            );
        }
        _ => panic!("expected grouped member-stop checked/proof parity"),
    }
}
