use crate::application::{
    assert_declaration_aspect_projections, WorthQueryDeclarationAspectCoverageBasis,
    WorthQueryDeclarationAspectFit, WorthQueryDeclarationEntryOrchestrationArtifactPolicy,
    WorthQueryDeclarationEntryOrchestrationExposureLevel,
    WorthQueryDeclarationEntryOrchestrationMaterializationTier,
    WorthQueryDeclarationEntryOrchestrationProduct, WorthQueryDeclarationEntryOrchestrationStage,
};

use super::super::lower::{
    worth_query_lower_declaration_entry_orchestration_on_handle,
    worth_query_lower_declaration_entry_product_orchestration_from_progressed_on_handle,
};
use super::domain::{
    admitted_handle, AspectRichFamily, AuthorityRichFamily, ConflictingAspectFamily, Input,
};

#[test]
fn orchestration_policy_exposes_semantic_publication_by_lane() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");

    let ordinary = worth_query_lower_declaration_entry_orchestration_on_handle(
        &handle,
        Input::<AspectRichFamily>::new("edge:42"),
        WorthQueryDeclarationEntryOrchestrationExposureLevel::Ordinary,
        WorthQueryDeclarationEntryOrchestrationArtifactPolicy::OrdinaryEnvelopeOnly,
    );
    let checked = worth_query_lower_declaration_entry_orchestration_on_handle(
        &handle,
        Input::<AspectRichFamily>::new("edge:42"),
        WorthQueryDeclarationEntryOrchestrationExposureLevel::Checked,
        WorthQueryDeclarationEntryOrchestrationArtifactPolicy::CheckedOutcomeOnly,
    );
    let proof = worth_query_lower_declaration_entry_orchestration_on_handle(
        &handle,
        Input::<AspectRichFamily>::new("edge:42"),
        WorthQueryDeclarationEntryOrchestrationExposureLevel::ProofVisible,
        WorthQueryDeclarationEntryOrchestrationArtifactPolicy::ProofVisibleTranscript,
    );

    assert_declaration_aspect_projections(
        ordinary.plan.foundational_aspect_publication().present(),
        &["selection.active_edge"],
    );
    assert_eq!(
        ordinary.plan.aspect_coverage_basis(),
        WorthQueryDeclarationAspectCoverageBasis::DeclaredFamilyCoverage
    );
    assert_declaration_aspect_projections(
        ordinary.plan.receipt_aspect_publication().present(),
        &["selection.active_edge", "selection.local_topology"],
    );
    assert_declaration_aspect_projections(
        ordinary.plan.envelope_aspect_publication().present(),
        &["selection.active_edge", "selection.local_topology"],
    );

    assert_declaration_aspect_projections(
        checked.plan.receipt_aspect_publication().present(),
        &["selection.active_edge"],
    );
    assert_declaration_aspect_projections(
        proof.plan.foundational_aspect_publication().present(),
        &[
            "selection.active_edge",
            "selection.local_topology",
            "selection.material_edit",
        ],
    );
    assert_declaration_aspect_projections(
        proof.plan.envelope_aspect_publication().masked(),
        &["selection.private_authority"],
    );
    assert_eq!(
        proof.plan.materialization_tier(),
        WorthQueryDeclarationEntryOrchestrationMaterializationTier::FullDescriptive
    );
}

#[test]
fn orchestration_identity_tracks_aspect_publication_inputs() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");

    let aspect_rich = worth_query_lower_declaration_entry_orchestration_on_handle(
        &handle,
        Input::<AspectRichFamily>::new("edge:42"),
        WorthQueryDeclarationEntryOrchestrationExposureLevel::Ordinary,
        WorthQueryDeclarationEntryOrchestrationArtifactPolicy::OrdinaryEnvelopeOnly,
    );
    let plain = worth_query_lower_declaration_entry_orchestration_on_handle(
        &handle,
        Input::<super::domain::AdmittedFamily>::new("edge:42"),
        WorthQueryDeclarationEntryOrchestrationExposureLevel::Ordinary,
        WorthQueryDeclarationEntryOrchestrationArtifactPolicy::OrdinaryEnvelopeOnly,
    );

    assert_ne!(
        aspect_rich.plan.orchestration_identity_digest(),
        plain.plan.orchestration_identity_digest()
    );
}

#[test]
fn orchestration_identity_changes_when_lane_publication_changes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");

    let ordinary = worth_query_lower_declaration_entry_orchestration_on_handle(
        &handle,
        Input::<AspectRichFamily>::new("edge:42"),
        WorthQueryDeclarationEntryOrchestrationExposureLevel::Ordinary,
        WorthQueryDeclarationEntryOrchestrationArtifactPolicy::OrdinaryEnvelopeOnly,
    );
    let checked = worth_query_lower_declaration_entry_orchestration_on_handle(
        &handle,
        Input::<AspectRichFamily>::new("edge:42"),
        WorthQueryDeclarationEntryOrchestrationExposureLevel::Checked,
        WorthQueryDeclarationEntryOrchestrationArtifactPolicy::CheckedOutcomeOnly,
    );

    assert_ne!(
        ordinary.plan.orchestration_identity_digest(),
        checked.plan.orchestration_identity_digest()
    );
}

#[test]
fn progressed_product_plans_carry_reviewed_aspect_coverage_basis() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let progressed = handle
        .declare_review_and_progress(Input::<AspectRichFamily>::new("edge:42"))
        .unwrap_or_else(|_| panic!("progression should admit"));

    let lowered =
        worth_query_lower_declaration_entry_product_orchestration_from_progressed_on_handle(
            &handle,
            progressed,
            WorthQueryDeclarationEntryOrchestrationExposureLevel::Checked,
            WorthQueryDeclarationEntryOrchestrationArtifactPolicy::CheckedOutcomeOnly,
            WorthQueryDeclarationEntryOrchestrationProduct::RoutePlan,
            None,
        );

    assert_eq!(
        lowered.plan.aspect_coverage_basis(),
        WorthQueryDeclarationAspectCoverageBasis::ReviewedRetainedCoverage
    );
}

#[test]
fn declared_family_coverage_keeps_conflicting_publication_out_of_visible_surfaces() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");

    let proof = worth_query_lower_declaration_entry_orchestration_on_handle(
        &handle,
        Input::<ConflictingAspectFamily>::new("edge:42"),
        WorthQueryDeclarationEntryOrchestrationExposureLevel::ProofVisible,
        WorthQueryDeclarationEntryOrchestrationArtifactPolicy::ProofVisibleTranscript,
    );

    assert_eq!(
        proof.plan.aspect_coverage_basis(),
        WorthQueryDeclarationAspectCoverageBasis::DeclaredFamilyCoverage
    );
    assert_declaration_aspect_projections(
        proof.plan.foundational_aspect_publication().present(),
        &["selection.active_edge", "selection.local_topology"],
    );
    assert_declaration_aspect_projections(
        proof.plan.foundational_aspect_publication().masked(),
        &["selection.material_edit", "selection.private_authority"],
    );
}

#[test]
fn product_orchestration_receipts_and_envelopes_match_planned_crossing_publication() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let progressed = handle
        .declare_review_and_progress(Input::<AspectRichFamily>::new("edge:42"))
        .unwrap_or_else(|_| panic!("progression should admit"));

    let receipt_lowered =
        worth_query_lower_declaration_entry_product_orchestration_from_progressed_on_handle(
            &handle,
            progressed.clone(),
            WorthQueryDeclarationEntryOrchestrationExposureLevel::Checked,
            WorthQueryDeclarationEntryOrchestrationArtifactPolicy::CheckedOutcomeOnly,
            WorthQueryDeclarationEntryOrchestrationProduct::Receipt,
            None,
        );
    let envelope_lowered =
        worth_query_lower_declaration_entry_product_orchestration_from_progressed_on_handle(
            &handle,
            progressed,
            WorthQueryDeclarationEntryOrchestrationExposureLevel::Checked,
            WorthQueryDeclarationEntryOrchestrationArtifactPolicy::CheckedOutcomeOnly,
            WorthQueryDeclarationEntryOrchestrationProduct::Envelope,
            None,
        );

    let receipt = match receipt_lowered.checked {
        super::super::lower::WorthQueryDeclarationEntryProductChecked::Receipt(
            crate::application::WorthQueryDeclarationReceiptChecked::Issued(receipt),
        ) => receipt,
        _ => panic!("receipt product should issue"),
    };
    let envelope = match envelope_lowered.checked {
        super::super::lower::WorthQueryDeclarationEntryProductChecked::Envelope(
            crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(envelope),
        ) => envelope,
        _ => panic!("envelope product should issue"),
    };

    assert_eq!(
        receipt.aspect_publication(),
        receipt_lowered.plan.receipt_aspect_publication()
    );
    assert_eq!(
        envelope.aspect_publication(),
        envelope_lowered.plan.envelope_aspect_publication()
    );
    assert_eq!(envelope.aspect_publication(), receipt.aspect_publication());
}

#[test]
fn envelope_proof_keeps_envelope_ceiling_while_exposing_authority_summaries() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let progressed = handle
        .declare_review_and_progress(Input::<AuthorityRichFamily>::new("edge:42"))
        .unwrap_or_else(|_| panic!("progression should admit"));

    let proof = handle.orchestrate_envelope_from_progressed_proof(progressed);

    assert_eq!(
        proof.plan().product(),
        WorthQueryDeclarationEntryOrchestrationProduct::Envelope
    );
    assert_eq!(
        proof
            .step_records()
            .last()
            .expect("proof should end at envelope")
            .stage(),
        WorthQueryDeclarationEntryOrchestrationStage::EnvelopeConstructed
    );
    assert_eq!(
        proof.relational_authority_summary(),
        proof.plan().relational_authority_summary()
    );
    assert_eq!(
        proof.bridge_authority_summary(),
        proof.plan().bridge_authority_summary()
    );
    assert_eq!(
        proof.signal_authority_summary(),
        proof.plan().signal_authority_summary()
    );
    assert_eq!(
        proof.relational_authority_summary().aspect_coverage_basis(),
        WorthQueryDeclarationAspectCoverageBasis::EnvelopePublishedCoverage
    );
    assert_eq!(
        proof.bridge_authority_summary().mapping_fit(),
        WorthQueryDeclarationAspectFit::MissingRequired
    );
    assert_declaration_aspect_projections(
        proof
            .signal_authority_summary()
            .produced_aspects()
            .required(),
        &["signal.preview_patch"],
    );
}

#[test]
fn authority_summaries_change_plan_identity_when_retained_meaning_changes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let authority_rich = worth_query_lower_declaration_entry_orchestration_on_handle(
        &handle,
        Input::<AuthorityRichFamily>::new("edge:42"),
        WorthQueryDeclarationEntryOrchestrationExposureLevel::Ordinary,
        WorthQueryDeclarationEntryOrchestrationArtifactPolicy::OrdinaryEnvelopeOnly,
    );
    let plain = worth_query_lower_declaration_entry_orchestration_on_handle(
        &handle,
        Input::<AspectRichFamily>::new("edge:42"),
        WorthQueryDeclarationEntryOrchestrationExposureLevel::Ordinary,
        WorthQueryDeclarationEntryOrchestrationArtifactPolicy::OrdinaryEnvelopeOnly,
    );

    assert_ne!(
        authority_rich.plan.orchestration_identity_digest(),
        plain.plan.orchestration_identity_digest()
    );
    assert_ne!(
        authority_rich.plan.bridge_authority_summary(),
        plain.plan.bridge_authority_summary()
    );
    assert_ne!(
        authority_rich.plan.signal_authority_summary(),
        plain.plan.signal_authority_summary()
    );
}
