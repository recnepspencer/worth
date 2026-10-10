use crate::grouped_authoring::{
    ordinary_outcome_from_grouped_orchestration_checked,
    worth_query_grouped_declaration_checked_on_handle,
    worth_query_grouped_orchestration_proof_on_handle, WorthQueryGroupedAtomicity,
    WorthQueryGroupedContinuityAssumption, WorthQueryGroupedDeclarationChecked,
    WorthQueryGroupedDeclarationInput, WorthQueryGroupedOrchestrationChecked,
    WorthQueryGroupedSharedPostureClaim, WorthQueryGroupedSupportFeature,
    WorthQueryGroupedSupportStatus,
};
use crate::ordinary_outcome::{
    WorthQueryOrdinaryBindingCheckedTopologyKind, WorthQueryOrdinaryNextStep,
    WorthQueryOrdinaryOutcome, WorthQueryOrdinaryPostureKind,
};

use super::support::{admitted_handle, GeometryInput, RequiredIntentGeometryInput};

#[test]
fn grouped_wrong_world_projects_to_binding_topology() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let left = admitted_handle(request_execution, "main");
    let right = admitted_handle(request_execution, "other");
    let declaration = match worth_query_grouped_declaration_checked_on_handle(
        &left,
        left.geometry_helpers()
            .local_neighborhood_for_active_face_selection(GeometryInput::new("face-a"))
            .with_member(GeometryInput::new("face-b")),
    ) {
        WorthQueryGroupedDeclarationChecked::Bound(value) => value,
        WorthQueryGroupedDeclarationChecked::MemberStopped(_) => {
            panic!("expected grouped declaration admission")
        }
    };

    let outcome = ordinary_outcome_from_grouped_orchestration_checked(
        right
            .geometry_helpers()
            .orchestrate_local_neighborhood_for_active_face_selection_checked(declaration),
    );

    match outcome {
        WorthQueryOrdinaryOutcome::WrongWorld(posture) => {
            assert_eq!(posture.kind(), WorthQueryOrdinaryPostureKind::WrongWorld);
            assert_eq!(
                posture.next_step(),
                WorthQueryOrdinaryNextStep::CorrectWorld
            );
            assert_eq!(
                posture.checked_topology().binding_kind(),
                Some(WorthQueryOrdinaryBindingCheckedTopologyKind::WrongWorld)
            );
            assert!(
                posture.reason().contains("different operating context"),
                "expected grouped wrong-world reason, got {:?}",
                posture.reason()
            );
        }
        _ => panic!("expected grouped wrong-world ordinary outcome"),
    }
}

#[test]
fn grouped_proof_into_checked_preserves_alignment_stop() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let left = admitted_handle(request_execution, "main");
    let right = admitted_handle(request_execution, "other");
    let declaration = match worth_query_grouped_declaration_checked_on_handle(
        &left,
        left.geometry_helpers()
            .local_neighborhood_for_active_face_selection(GeometryInput::new("face-a")),
    ) {
        WorthQueryGroupedDeclarationChecked::Bound(value) => value,
        WorthQueryGroupedDeclarationChecked::MemberStopped(_) => {
            panic!("expected grouped declaration admission")
        }
    };

    let proof = worth_query_grouped_orchestration_proof_on_handle(&right, declaration);
    assert!(
        proof.member_transcripts().is_empty(),
        "wrong-world proof should short-circuit before member lowering"
    );
    let checked = proof.into_checked();

    match checked {
        WorthQueryGroupedOrchestrationChecked::WrongWorld(stop) => {
            assert!(
                stop.reason().contains("different operating context"),
                "expected grouped proof alignment stop, got {:?}",
                stop.reason()
            );
        }
        _ => panic!("expected grouped wrong-world checked stop"),
    }
}

#[test]
fn grouped_proof_member_stop_short_circuits_after_first_failing_member() {
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

    let proof = worth_query_grouped_orchestration_proof_on_handle(&handle, declaration);
    assert_eq!(
        proof.member_transcripts().len(),
        1,
        "proof should stop after the first failing member instead of lowering later members"
    );

    match proof.outcome() {
        WorthQueryGroupedOrchestrationChecked::MemberStopped(stop) => {
            assert_eq!(stop.member_index(), 0);
            assert_eq!(stop.member_role().as_str(), "seed");
        }
        _ => panic!("expected grouped member-stop proof outcome"),
    }
}

#[test]
fn grouped_support_report_marks_unsupported_shared_claims() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "main");
    let declaration = match worth_query_grouped_declaration_checked_on_handle(
        &handle,
        WorthQueryGroupedDeclarationInput::local_neighborhood(RequiredIntentGeometryInput::new(
            "edge-a",
        ))
        .with_member(RequiredIntentGeometryInput::new("edge-b"))
        .with_atomicity(WorthQueryGroupedAtomicity::Atomic)
        .with_continuity_assumption(WorthQueryGroupedContinuityAssumption::PreserveNeighborhood)
        .with_shared_posture_claim(WorthQueryGroupedSharedPostureClaim::SharedMaterialPreview)
        .with_shared_posture_claim(WorthQueryGroupedSharedPostureClaim::SharedContinuity),
    ) {
        WorthQueryGroupedDeclarationChecked::Bound(value) => value,
        WorthQueryGroupedDeclarationChecked::MemberStopped(_) => {
            panic!("expected grouped declaration admission")
        }
    };

    let report = handle.grouped_support_report(&declaration);
    assert_eq!(
        report.status_for(WorthQueryGroupedSupportFeature::SharedPostureClaims),
        WorthQueryGroupedSupportStatus::Unsupported
    );
    assert!(report
        .unsupported_claims()
        .contains(&WorthQueryGroupedSharedPostureClaim::SharedMaterialPreview));
    assert!(!report
        .unsupported_claims()
        .contains(&WorthQueryGroupedSharedPostureClaim::SharedContinuity));
}
