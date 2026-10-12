use crate::application::{
    assert_declaration_aspect_projections, WorthQueryAsyncLegalityDenialKind,
    WorthQueryDeclarationAdmissionOrLegalityError, WorthQueryDeclarationCapabilityStatus,
    WorthQueryDeclarationLegalityChecked, WorthQueryDeclarationLegalityContract,
    WorthQueryDeclarationLegalityDenial, WorthQueryTemporalLegalityDenialKind,
};

mod async_fixtures;
mod fixtures;

use async_fixtures::{
    AsyncCurrentFamily, AsyncDeclaration, AsyncHistoricalFamily, AsyncPreviewFamily,
};
use fixtures::{
    admitted_handle, Declaration, DeferredLegalityFamily, DurableAdmissionFamily,
    IllegalDispositionFamily, IllegalRoleFamily, LegalFamily, MaskedCoverageFamily,
    TemporalCurrentFamily, TemporalDeclaration, TemporalHistoricalFamily, TemporalPreviewFamily,
};

#[test]
fn legal_declaration_review_yields_legality_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let world_basis = handle.retained_world_basis();
    let declaration = handle
        .declare(Declaration::<LegalFamily>::new("edge:42"))
        .expect("declaration should admit");

    let legal = handle
        .review_legality(declaration)
        .expect("legality review should pass");

    assert_eq!(legal.declaration_family_key(), "split-edge");
    assert!(legal.is_structurally_legal());
    assert_eq!(
        legal.legality_contract(),
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    );
    assert_eq!(
        legal.support_report().declare_status(),
        WorthQueryDeclarationCapabilityStatus::Admitted
    );
    assert_eq!(
        legal.operating_context_identity_digest(),
        world_basis.operating_context_identity_digest()
    );
    assert_eq!(
        legal.canonical_declaration().handle_identity_digest(),
        world_basis.handle_identity_for_reporting()
    );
    assert_declaration_aspect_projections(
        legal.aspect_contract().required(),
        &["selection.active_edge"],
    );
    assert_declaration_aspect_projections(
        legal.reviewed_aspect_coverage().present(),
        &["selection.active_edge"],
    );
}

#[test]
fn legality_review_rejects_declarations_from_a_different_admitted_world() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let left = admitted_handle(request_execution, "collaborative");
    let right = admitted_handle(request_execution, "restricted");
    let declaration = left
        .declare(Declaration::<LegalFamily>::new("edge:42"))
        .expect("declaration should admit");

    match right.review_legality_checked(declaration) {
        WorthQueryDeclarationLegalityChecked::Illegal(
            crate::application::WorthQueryDeclarationLegalityDenial::WrongAdmittedWorld { .. },
        ) => {}
        other => panic!(
            "expected wrong-world denial, got {:?}",
            std::mem::discriminant(&other)
        ),
    }
}

#[test]
fn legality_review_distinguishes_role_and_surface_failures() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");

    let bad_role = handle
        .declare(Declaration::<IllegalRoleFamily>::new("edge:42"))
        .expect("declaration should admit");
    match handle.review_legality_checked(bad_role) {
        WorthQueryDeclarationLegalityChecked::Illegal(
            crate::application::WorthQueryDeclarationLegalityDenial::IllegalRoleClaim { .. },
        ) => {}
        other => panic!(
            "expected role denial, got {:?}",
            std::mem::discriminant(&other)
        ),
    }

    let bad_surface = handle
        .declare(Declaration::<IllegalDispositionFamily>::new("edge:42"))
        .expect("declaration should admit");
    match handle.review_legality_checked(bad_surface) {
        WorthQueryDeclarationLegalityChecked::Illegal(
            crate::application::WorthQueryDeclarationLegalityDenial::IllegalSurfaceDisposition {
                ..
            },
        ) => {}
        other => panic!(
            "expected disposition denial, got {:?}",
            std::mem::discriminant(&other)
        ),
    }
}

#[test]
fn legality_boundary_can_defer_even_after_family_admission() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let declaration = handle
        .declare(Declaration::<DeferredLegalityFamily>::new("edge:42"))
        .expect("declaration should admit");

    match handle.review_legality_checked(declaration) {
        WorthQueryDeclarationLegalityChecked::Illegal(
            crate::application::WorthQueryDeclarationLegalityDenial::DeferredByLegalityBoundary {
                ..
            },
        ) => {}
        other => panic!(
            "expected deferred legality denial, got {:?}",
            std::mem::discriminant(&other)
        ),
    }
}

#[test]
fn declare_and_review_preserves_admission_vs_legality_split() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");

    match handle.declare_and_review(Declaration::<DurableAdmissionFamily>::new("edge:42")) {
        Err(WorthQueryDeclarationAdmissionOrLegalityError::Admission(admission)) => {
            assert!(matches!(
                admission,
                crate::application::WorthQueryDeclarationAdmissionError::Unsupported(_)
            ));
        }
        _ => panic!("expected admission denial"),
    }

    match handle.declare_and_review(Declaration::<IllegalRoleFamily>::new("edge:42")) {
        Err(WorthQueryDeclarationAdmissionOrLegalityError::Legality(
            crate::application::WorthQueryDeclarationLegalityDenial::IllegalRoleClaim { .. },
        )) => {}
        _ => panic!("expected legality denial"),
    }
}

#[test]
fn legality_evidence_preserves_masked_aspect_coverage_without_promoting_it_to_reviewed_presence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let declaration = handle
        .declare(Declaration::<MaskedCoverageFamily>::new("edge:42"))
        .expect("declaration should admit");

    let legal = handle
        .review_legality(declaration)
        .expect("legality review should pass");

    assert_declaration_aspect_projections(
        legal.reviewed_aspect_coverage().present(),
        &["selection.active_edge"],
    );
    assert_declaration_aspect_projections(
        legal.reviewed_aspect_coverage().masked(),
        &["selection.active_edge"],
    );
}

#[test]
fn temporal_declarations_pass_legality_when_runtime_temporal_support_is_admitted() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let declaration = handle
        .declare(TemporalDeclaration::<TemporalCurrentFamily>::new("edge:42"))
        .expect("temporal declaration should admit canonically");

    let legal = handle
        .review_legality(declaration)
        .expect("runtime-backed temporal legality should now admit");
    assert!(legal.is_structurally_legal());
}

#[test]
fn temporal_preview_and_historical_truth_basis_remain_typed_legality_denials() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let preview = handle
        .declare(TemporalDeclaration::<TemporalPreviewFamily>::new("edge:42"))
        .expect("temporal preview declaration should admit canonically");
    let historical = handle
        .declare(TemporalDeclaration::<TemporalHistoricalFamily>::new(
            "edge:42",
        ))
        .expect("temporal historical declaration should admit canonically");

    match handle.review_legality_checked(preview) {
        WorthQueryDeclarationLegalityChecked::Illegal(
            crate::application::WorthQueryDeclarationLegalityDenial::TemporalProjectionUnsupported {
                kind,
                ..
            },
        ) => {
            assert_eq!(kind, WorthQueryTemporalLegalityDenialKind::PreviewTruthBasisUnsupported);
        }
        other => panic!(
            "expected preview temporal legality denial, got {:?}",
            std::mem::discriminant(&other)
        ),
    }

    match handle.review_legality_checked(historical) {
        WorthQueryDeclarationLegalityChecked::Illegal(
            crate::application::WorthQueryDeclarationLegalityDenial::TemporalProjectionUnsupported {
                kind,
                ..
            },
        ) => {
            assert_eq!(
                kind,
                WorthQueryTemporalLegalityDenialKind::HistoricalTruthBasisUnsupported
            );
        }
        other => panic!(
            "expected historical temporal legality denial, got {:?}",
            std::mem::discriminant(&other)
        ),
    }
}

#[test]
fn async_declarations_pass_legality_when_runtime_async_support_is_admitted() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let declaration = handle
        .declare(AsyncDeclaration::<AsyncCurrentFamily>::bridge_blocking(
            "edge:42",
        ))
        .expect("async declaration should admit canonically");

    let legal = handle
        .review_legality(declaration)
        .expect("runtime-backed async legality should now admit");
    assert!(legal.is_structurally_legal());
}

#[test]
fn async_preview_and_historical_truth_basis_remain_typed_legality_denials() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let preview = handle
        .declare(AsyncDeclaration::<AsyncPreviewFamily>::bridge_blocking(
            "edge:42",
        ))
        .expect("async preview declaration should admit canonically");
    let historical = handle
        .declare(AsyncDeclaration::<AsyncHistoricalFamily>::bridge_blocking(
            "edge:42",
        ))
        .expect("async historical declaration should admit canonically");

    match handle.review_legality_checked(preview) {
        WorthQueryDeclarationLegalityChecked::Illegal(
            WorthQueryDeclarationLegalityDenial::AsyncProjectionUnsupported { kind, .. },
        ) => {
            assert_eq!(
                kind,
                WorthQueryAsyncLegalityDenialKind::PreviewTruthBasisUnsupported
            );
        }
        other => panic!(
            "expected preview async legality denial, got {:?}",
            std::mem::discriminant(&other)
        ),
    }

    match handle.review_legality_checked(historical) {
        WorthQueryDeclarationLegalityChecked::Illegal(
            WorthQueryDeclarationLegalityDenial::AsyncProjectionUnsupported { kind, .. },
        ) => {
            assert_eq!(
                kind,
                WorthQueryAsyncLegalityDenialKind::HistoricalTruthBasisUnsupported
            );
        }
        other => panic!(
            "expected historical async legality denial, got {:?}",
            std::mem::discriminant(&other)
        ),
    }
}

#[test]
fn async_clause_mismatches_produce_typed_legality_denials_before_progression() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let cases = [
        (
            AsyncDeclaration::<AsyncCurrentFamily>::external_blocking("edge:42"),
            WorthQueryAsyncLegalityDenialKind::UnsupportedSourceFamily(
                crate::application::WorthQueryAsyncSourceFamily::ExternalResource,
            ),
        ),
        (
            AsyncDeclaration::<AsyncCurrentFamily>::bridge_refresh("edge:42"),
            WorthQueryAsyncLegalityDenialKind::UnsupportedLoadingPosture(
                crate::application::WorthQueryAsyncLoadingPosture::BackgroundRefresh,
            ),
        ),
        (
            AsyncDeclaration::<AsyncCurrentFamily>::bridge_retain_stale("edge:42"),
            WorthQueryAsyncLegalityDenialKind::UnsupportedFailurePosture(
                crate::application::WorthQueryAsyncFailurePosture::RetainStaleValue,
            ),
        ),
        (
            AsyncDeclaration::<AsyncCurrentFamily>::bridge_completion("edge:42"),
            WorthQueryAsyncLegalityDenialKind::CompletionLifecycleUnsupported,
        ),
    ];

    for (input, expected_kind) in cases {
        let declaration = handle
            .declare(input)
            .expect("async mismatch declaration should admit canonically");
        match handle.review_legality_checked(declaration) {
            WorthQueryDeclarationLegalityChecked::Illegal(
                WorthQueryDeclarationLegalityDenial::AsyncProjectionUnsupported { kind, .. },
            ) => {
                assert_eq!(kind, expected_kind);
            }
            other => panic!(
                "expected async mismatch legality denial, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }
}
