use crate::application::{
    WorthQueryDeclarationEntryOrchestrationRefusalClass,
    WorthQueryDeclarationEntryOrchestrationStage,
};
use crate::binding_pipeline::WorthQueryBindingLinkedArtifacts;
use crate::grouped_authoring::{
    worth_query_grouped_declaration_checked_on_handle,
    worth_query_grouped_orchestration_checked_on_handle, WorthQueryGroupedDeclarationChecked,
    WorthQueryGroupedDeclarationInput, WorthQueryGroupedOrchestrationChecked,
};
use crate::ordinary_outcome::{
    WorthQueryOrdinaryCheckedTopology, WorthQueryOrdinaryNextStep, WorthQueryOrdinaryOutcome,
    WorthQueryOrdinaryPosture, WorthQueryOrdinaryPostureKind,
    WorthQueryOrdinarySignalCompatibilityOrchestrationCheckedTopologyKind,
};

use super::support::{
    recovery_admitted_handle, RecoveryInput, RequiredIntentRouteFamily, SignalReceiptFamily,
};
use crate::recovery_boundary::{
    worth_query_recovery_brief_from_ordinary_outcome, WorthQueryRecoveryAction,
    WorthQueryRecoveryAspectPosture, WorthQueryRecoveryAuthoritySurface,
    WorthQueryRecoveryEvidenceStrength, WorthQueryRecoverySourceFamily,
    WorthQueryRecoveryStopFamily, WorthQueryRecoveryStopKind,
};

#[test]
fn ordinary_contribution_denial_uses_contribution_recovery_surface() {
    let posture = WorthQueryOrdinaryPosture::new(
        "contribution denied",
        WorthQueryOrdinaryPostureKind::Denied,
        WorthQueryOrdinaryNextStep::InspectCheckedLane,
        WorthQueryOrdinaryCheckedTopology::contribution_composed(
            crate::ordinary_outcome::WorthQueryOrdinaryContributionComposedCheckedTopologyKind::ContributionDenied,
            WorthQueryBindingLinkedArtifacts::new().with_envelope_digest("env-1"),
            Some("contrib-1".to_string()),
        ),
    );
    let brief = worth_query_recovery_brief_from_ordinary_outcome(
        &WorthQueryOrdinaryOutcome::<()>::Denied(posture),
    )
    .expect("denial should yield a recovery brief");

    assert_eq!(
        brief.stop_family(),
        WorthQueryRecoveryStopFamily::ContributionComposedOrchestration
    );
    assert_eq!(
        brief.stop_kind(),
        WorthQueryRecoveryStopKind::ContributionDenied
    );
    assert_eq!(
        brief.authority_surface(),
        WorthQueryRecoveryAuthoritySurface::ContributionComposition
    );
    assert_eq!(
        brief.recommended_action(),
        WorthQueryRecoveryAction::ReviewContributionIntent
    );
    assert_eq!(
        brief.route_sensitive_explanation().contribution_digest(),
        Some("contrib-1")
    );
    assert_eq!(
        brief.recovery_request().explanation().contribution_digest(),
        Some("contrib-1")
    );
}

#[test]
fn ordinary_declaration_refusal_preserves_route_sensitive_refusal_context() {
    let brief = worth_query_recovery_brief_from_ordinary_outcome(
        &WorthQueryOrdinaryOutcome::<()>::Refused(WorthQueryOrdinaryPosture::new(
            "prepared but not executed",
            WorthQueryOrdinaryPostureKind::Refused,
            WorthQueryOrdinaryNextStep::UseExplicitHandoff,
            WorthQueryOrdinaryCheckedTopology::orchestration(
                WorthQueryDeclarationEntryOrchestrationStage::RoutePlanned,
                Some("retained-1".to_string()),
                Some(
                    WorthQueryDeclarationEntryOrchestrationRefusalClass::PreparedButNotExecutedContinuation,
                ),
            ),
        )),
    )
    .expect("refusal should yield a recovery brief");

    assert_eq!(
        brief.stop_family(),
        WorthQueryRecoveryStopFamily::DeclarationEntry
    );
    assert_eq!(brief.stop_kind(), WorthQueryRecoveryStopKind::Refused);
    assert_eq!(
        brief.authority_surface(),
        WorthQueryRecoveryAuthoritySurface::AutomationBoundary
    );
    assert_eq!(
        brief.recommended_action(),
        WorthQueryRecoveryAction::UseExplicitHandoff
    );
    assert_eq!(
        brief.route_sensitive_explanation().refusal_class(),
        Some(
            crate::application::WorthQueryDeclarationEntryOrchestrationRefusalClass::PreparedButNotExecutedContinuation,
        )
    );
}

#[test]
fn ordinary_signal_basis_mismatch_maps_to_signal_repair() {
    let brief = worth_query_recovery_brief_from_ordinary_outcome(
        &WorthQueryOrdinaryOutcome::<()>::BasisMismatch(WorthQueryOrdinaryPosture::new(
            "basis mismatch",
            WorthQueryOrdinaryPostureKind::BasisMismatch,
            WorthQueryOrdinaryNextStep::RefreshBasis,
            WorthQueryOrdinaryCheckedTopology::signal_compatibility_orchestration(
                WorthQueryOrdinarySignalCompatibilityOrchestrationCheckedTopologyKind::BasisMismatch,
                WorthQueryBindingLinkedArtifacts::new().with_envelope_digest("env-1"),
            ),
        )),
    )
    .expect("basis mismatch should yield a recovery brief");

    assert_eq!(
        brief.stop_family(),
        WorthQueryRecoveryStopFamily::SignalCompatibilityOrchestration
    );
    assert_eq!(brief.stop_kind(), WorthQueryRecoveryStopKind::BasisMismatch);
    assert_eq!(
        brief.authority_surface(),
        WorthQueryRecoveryAuthoritySurface::SignalCompatibility
    );
    assert_eq!(
        brief.recommended_action(),
        WorthQueryRecoveryAction::RefreshBasis
    );
}

#[test]
fn grouped_wrong_world_checked_uses_grouped_recovery_family() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let left = recovery_admitted_handle(request_execution, "left");
    let right = recovery_admitted_handle(request_execution, "right");
    let declaration = match worth_query_grouped_declaration_checked_on_handle(
        &left,
        WorthQueryGroupedDeclarationInput::local_neighborhood(
            RecoveryInput::<SignalReceiptFamily>::new("edge-a"),
        ),
    ) {
        WorthQueryGroupedDeclarationChecked::Bound(value) => value,
        WorthQueryGroupedDeclarationChecked::MemberStopped(_) => {
            panic!("grouped declaration should admit")
        }
    };

    let checked = worth_query_grouped_orchestration_checked_on_handle(&right, declaration);
    assert!(matches!(
        checked,
        WorthQueryGroupedOrchestrationChecked::WrongWorld(_)
    ));

    let brief = right
        .recover_from_grouped_orchestration_checked(checked)
        .expect("grouped wrong-world stop should yield a recovery brief");
    assert_eq!(
        brief.stop_family(),
        WorthQueryRecoveryStopFamily::GroupedNeighborhoodOrchestration
    );
    assert_eq!(brief.stop_kind(), WorthQueryRecoveryStopKind::WrongWorld);
    assert_eq!(
        brief.authority_surface(),
        WorthQueryRecoveryAuthoritySurface::AdmittedOperatingWorld
    );
    assert_eq!(
        brief.recommended_action(),
        WorthQueryRecoveryAction::CorrectWorld
    );
}

#[test]
fn grouped_wrong_world_recovery_brief_matches_between_checked_and_proof_lanes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let left = recovery_admitted_handle(request_execution, "left");
    let right = recovery_admitted_handle(request_execution, "right");
    let declaration = |edge_ref| match worth_query_grouped_declaration_checked_on_handle(
        &left,
        WorthQueryGroupedDeclarationInput::local_neighborhood(
            RecoveryInput::<SignalReceiptFamily>::new(edge_ref),
        ),
    ) {
        WorthQueryGroupedDeclarationChecked::Bound(value) => value,
        WorthQueryGroupedDeclarationChecked::MemberStopped(_) => {
            panic!("grouped declaration should admit")
        }
    };

    let checked_brief = right
        .recover_from_grouped_orchestration_checked(
            worth_query_grouped_orchestration_checked_on_handle(&right, declaration("edge-b")),
        )
        .expect("checked grouped stop should yield a recovery brief");
    let proof_brief = right
        .recover_from_grouped_orchestration_proof(
            crate::grouped_authoring::worth_query_grouped_orchestration_proof_on_handle(
                &right,
                declaration("edge-b"),
            ),
        )
        .expect("proof grouped stop should yield a recovery brief");

    assert_eq!(checked_brief.stop_family(), proof_brief.stop_family());
    assert_eq!(checked_brief.stop_kind(), proof_brief.stop_kind());
    assert_eq!(
        checked_brief.authority_surface(),
        proof_brief.authority_surface()
    );
    assert_eq!(
        checked_brief.recommended_action(),
        proof_brief.recommended_action()
    );
}

#[test]
fn grouped_member_stop_recovery_preserves_member_aspect_context() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = recovery_admitted_handle(request_execution, "main");
    let declaration = match worth_query_grouped_declaration_checked_on_handle(
        &handle,
        WorthQueryGroupedDeclarationInput::local_neighborhood(RecoveryInput::<
            RequiredIntentRouteFamily,
        >::new("edge-c"))
        .with_member(RecoveryInput::<RequiredIntentRouteFamily>::new("edge-d")),
    ) {
        WorthQueryGroupedDeclarationChecked::Bound(value) => value,
        WorthQueryGroupedDeclarationChecked::MemberStopped(_) => {
            panic!("grouped declaration should admit")
        }
    };
    let expected_member_aspect = declaration.members()[0].aspect_record().clone();

    let checked = worth_query_grouped_orchestration_checked_on_handle(&handle, declaration);
    let brief = handle
        .recover_from_grouped_orchestration_checked(checked)
        .expect("grouped member stop should yield a recovery brief");

    assert_eq!(
        brief.stop_family(),
        WorthQueryRecoveryStopFamily::GroupedNeighborhoodOrchestration
    );
    assert_eq!(
        brief.source_family(),
        WorthQueryRecoverySourceFamily::GroupedNeighborhood
    );
    assert_eq!(
        brief.evidence_strength(),
        WorthQueryRecoveryEvidenceStrength::CheckedRetained
    );
    assert_eq!(
        brief.aspect_posture(),
        WorthQueryRecoveryAspectPosture::RetainedContractAndCoverage
    );
    let context = brief
        .explanation()
        .grouped_member_context()
        .expect("grouped member stop should carry member-local aspect context");
    assert_eq!(context.member_index(), 0);
    assert_eq!(context.member_role().as_str(), "seed");
    assert_eq!(context.aspect_record(), &expected_member_aspect);
    assert!(brief
        .explanation()
        .has_retained_grouped_member_aspect_context());
}

#[test]
fn grouped_member_stop_recovery_brief_matches_between_checked_and_proof_lanes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = recovery_admitted_handle(request_execution, "main");
    let declaration = || match worth_query_grouped_declaration_checked_on_handle(
        &handle,
        WorthQueryGroupedDeclarationInput::local_neighborhood(RecoveryInput::<
            RequiredIntentRouteFamily,
        >::new("edge-c"))
        .with_member(RecoveryInput::<RequiredIntentRouteFamily>::new("edge-d")),
    ) {
        WorthQueryGroupedDeclarationChecked::Bound(value) => value,
        WorthQueryGroupedDeclarationChecked::MemberStopped(_) => {
            panic!("grouped declaration should admit")
        }
    };

    let checked_brief = handle
        .recover_from_grouped_orchestration_checked(
            worth_query_grouped_orchestration_checked_on_handle(&handle, declaration()),
        )
        .expect("checked grouped member stop should yield a recovery brief");
    let proof_brief = handle
        .recover_from_grouped_orchestration_proof(
            crate::grouped_authoring::worth_query_grouped_orchestration_proof_on_handle(
                &handle,
                declaration(),
            ),
        )
        .expect("proof grouped member stop should yield a recovery brief");

    assert_eq!(checked_brief.stop_family(), proof_brief.stop_family());
    assert_eq!(checked_brief.stop_kind(), proof_brief.stop_kind());
    assert_eq!(checked_brief.source_family(), proof_brief.source_family());
    assert_eq!(checked_brief.aspect_posture(), proof_brief.aspect_posture());
    assert_eq!(
        checked_brief.authority_surface(),
        proof_brief.authority_surface()
    );
    assert_eq!(
        checked_brief.recommended_action(),
        proof_brief.recommended_action()
    );
    assert_eq!(
        checked_brief
            .explanation()
            .grouped_member_context()
            .expect("checked grouped member stop should carry member context"),
        proof_brief
            .explanation()
            .grouped_member_context()
            .expect("proof grouped member stop should carry member context")
    );
    assert_eq!(
        checked_brief.evidence_strength(),
        WorthQueryRecoveryEvidenceStrength::CheckedRetained
    );
    assert_eq!(
        proof_brief.evidence_strength(),
        WorthQueryRecoveryEvidenceStrength::ProofRetained
    );
}
