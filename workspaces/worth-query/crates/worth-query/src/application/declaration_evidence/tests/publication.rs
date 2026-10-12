use worth_foundational::facade::FoundationalBoundaryEvidenceMaterializationProfile;

use crate::application::{
    assert_declaration_aspect_projections, WorthQueryDeclarationAspectCoverageBasis,
    WorthQueryDeclarationFoundationalEvidenceInput,
};

use super::domain::{admitted_handle, AspectRichFamily, ConflictingAspectFamily, Declaration};

#[test]
fn foundational_profiles_publish_semantic_slices_honestly() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let progression = handle
        .declare_review_and_progress(Declaration::<AspectRichFamily>::new("edge:42"))
        .unwrap_or_else(|_| panic!("progression should admit"));

    let lean = handle
        .describe_foundational_with_profile(
            WorthQueryDeclarationFoundationalEvidenceInput::admitted_progression(
                progression.clone(),
            ),
            FoundationalBoundaryEvidenceMaterializationProfile::ElideSupportAndDiagnostics,
        )
        .expect("lean foundational evidence should describe");
    let support_ready = handle
        .describe_foundational_with_profile(
            WorthQueryDeclarationFoundationalEvidenceInput::admitted_progression(
                progression.clone(),
            ),
            FoundationalBoundaryEvidenceMaterializationProfile::ElideDiagnostics,
        )
        .expect("support-ready foundational evidence should describe");
    let full = handle
        .describe_foundational_with_profile(
            WorthQueryDeclarationFoundationalEvidenceInput::admitted_progression(progression),
            FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness,
        )
        .expect("full foundational evidence should describe");

    assert_declaration_aspect_projections(
        lean.aspect_publication().present(),
        &["selection.active_edge"],
    );
    assert!(lean.aspect_publication().widened().is_empty());
    assert_declaration_aspect_projections(
        lean.aspect_publication().elided(),
        &["selection.local_topology", "selection.material_edit"],
    );

    assert_declaration_aspect_projections(
        support_ready.aspect_publication().present(),
        &["selection.active_edge", "selection.local_topology"],
    );
    assert_declaration_aspect_projections(
        support_ready.aspect_publication().widened(),
        &["selection.local_topology"],
    );

    assert_declaration_aspect_projections(
        full.aspect_publication().present(),
        &[
            "selection.active_edge",
            "selection.local_topology",
            "selection.material_edit",
        ],
    );
    assert_declaration_aspect_projections(
        full.aspect_publication().masked(),
        &["selection.private_authority"],
    );
    assert_eq!(lean.aspect_contract(), support_ready.aspect_contract());
    assert_eq!(support_ready.aspect_coverage(), full.aspect_coverage());
    assert_eq!(
        full.aspect_coverage_basis(),
        WorthQueryDeclarationAspectCoverageBasis::ReviewedRetainedCoverage
    );
}

#[test]
fn foundational_publication_changes_without_changing_retained_truth_identity() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let progression = handle
        .declare_review_and_progress(Declaration::<AspectRichFamily>::new("edge:42"))
        .unwrap_or_else(|_| panic!("progression should admit"));

    let lean = handle
        .describe_foundational_with_profile(
            WorthQueryDeclarationFoundationalEvidenceInput::admitted_progression(
                progression.clone(),
            ),
            FoundationalBoundaryEvidenceMaterializationProfile::ElideSupportAndDiagnostics,
        )
        .expect("lean foundational evidence should describe");
    let full = handle
        .describe_foundational_with_profile(
            WorthQueryDeclarationFoundationalEvidenceInput::admitted_progression(progression),
            FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness,
        )
        .expect("full foundational evidence should describe");

    assert_eq!(lean.declaration_digest(), full.declaration_digest());
    assert_eq!(lean.progression_digest(), full.progression_digest());
    assert_ne!(lean.aspect_publication(), full.aspect_publication());
}

#[test]
fn legality_denial_foundational_evidence_marks_support_reported_coverage_basis() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let declaration = handle
        .declare(Declaration::<super::domain::IllegalRoleFamily>::new(
            "edge:42",
        ))
        .unwrap_or_else(|_| panic!("declaration should admit"));
    let denial = handle
        .review_legality(declaration)
        .err()
        .unwrap_or_else(|| panic!("legality should deny"));

    let evidence = handle
        .describe_foundational(
            WorthQueryDeclarationFoundationalEvidenceInput::legality_denial(denial),
        )
        .expect("denied foundational evidence should still describe");

    assert_eq!(
        evidence.aspect_coverage_basis(),
        WorthQueryDeclarationAspectCoverageBasis::SupportReportedCoverage
    );
}

#[test]
fn conflicting_aspects_stay_masked_even_on_full_foundational_publication() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let progression = handle
        .declare_review_and_progress(Declaration::<ConflictingAspectFamily>::new("edge:42"))
        .unwrap_or_else(|_| panic!("progression should admit"));

    let evidence = handle
        .describe_foundational_with_profile(
            WorthQueryDeclarationFoundationalEvidenceInput::admitted_progression(progression),
            FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness,
        )
        .expect("full foundational evidence should describe");

    assert_declaration_aspect_projections(
        evidence.aspect_publication().present(),
        &["selection.active_edge", "selection.local_topology"],
    );
    assert_declaration_aspect_projections(
        evidence.aspect_publication().masked(),
        &["selection.material_edit", "selection.private_authority"],
    );
}
