use crate::application::{
    assert_declaration_aspect_projections, WorthQueryDeclarationAspectCoverageBasis,
    WorthQueryDeclarationAspectFit, WorthQueryDeclarationBridgeRoutingInput,
    WorthQueryDeclarationEntryInspectionInput, WorthQueryDeclarationEntryReadinessStatus,
    WorthQueryDeclarationRelationalRoutingInput, WorthQueryDeclarationSignalCompatibilityInput,
};

use super::support::{
    authority_rich_envelope, bridge_signal_envelope, handle, Input, RelationalFamily,
};

#[test]
fn wrong_handle_retained_artifacts_are_denied_before_inspection() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let source = handle(request_execution, "source");
    let target = handle(request_execution, "target");
    let envelope = bridge_signal_envelope(&source, "edge:42");

    let error = match target.inspect_declaration_entry(
        WorthQueryDeclarationEntryInspectionInput::envelope_checked(
            crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(envelope),
        ),
    ) {
        Ok(_) => panic!("wrong-handle inspection must deny"),
        Err(error) => error,
    };
    assert!(error.reason().contains("same admitted handle"));
}

#[test]
fn envelope_only_inspection_does_not_fake_lower_authority_posture() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "preview");
    let envelope = bridge_signal_envelope(&handle, "edge:42");

    let inspection = match handle.inspect_declaration_entry(
        WorthQueryDeclarationEntryInspectionInput::envelope_checked(
            crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(envelope),
        ),
    ) {
        Ok(inspection) => inspection,
        Err(_) => panic!("inspection should succeed"),
    };

    assert!(inspection.relational_posture().is_none());
    assert!(inspection.bridge_posture().is_none());
    assert!(inspection.signal_posture().is_none());
    assert_eq!(inspection.matching_row_digests().len(), 5);
}

#[test]
fn signal_checked_inspection_exposes_signal_posture() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "preview");
    let envelope = bridge_signal_envelope(&handle, "edge:42");
    let checked = handle.signal_compatibility_checked(
        WorthQueryDeclarationSignalCompatibilityInput::enveloped(envelope),
    );
    let inspection = match handle.inspect_declaration_entry(
        WorthQueryDeclarationEntryInspectionInput::signal_compatibility_checked(checked),
    ) {
        Ok(inspection) => inspection,
        Err(_) => panic!("inspection should succeed"),
    };

    assert!(inspection.signal_posture().is_some());
    assert!(inspection.matching_row_digests().len() >= 2);
}

#[test]
fn authority_rich_envelope_inspection_surfaces_authority_summaries_in_readiness() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "preview");
    let envelope = authority_rich_envelope(&handle, "edge:42");

    let inspection = match handle.inspect_declaration_entry(
        WorthQueryDeclarationEntryInspectionInput::envelope_checked(
            crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(envelope),
        ),
    ) {
        Ok(inspection) => inspection,
        Err(_) => panic!("inspection should succeed"),
    };

    assert_declaration_aspect_projections(
        inspection.envelope_aspect_publication().masked(),
        &["selection.private_authority"],
    );

    let relational_row = inspection
        .readiness()
        .rows()
        .iter()
        .find(|row| row.crossing_row().relational_truth_claim().is_some())
        .expect("relational row should exist");
    assert_eq!(
        relational_row
            .relational_authority_summary()
            .expect("relational summary should exist")
            .aspect_coverage_basis(),
        WorthQueryDeclarationAspectCoverageBasis::EnvelopePublishedCoverage
    );

    let bridge_row = inspection
        .readiness()
        .rows()
        .iter()
        .find(|row| row.crossing_row().bridge_continuation_family().is_some())
        .expect("bridge row should exist");
    assert_eq!(
        bridge_row.status(),
        WorthQueryDeclarationEntryReadinessStatus::Unsupported
    );
    assert!(bridge_row
        .bridge_authority_summary()
        .expect("bridge summary should exist")
        .mapped_aspects()
        .present()
        .is_empty());
    assert!(bridge_row
        .bridge_authority_summary()
        .expect("bridge summary should exist")
        .aspect_mismatch()
        .is_some());

    let signal_row = inspection
        .readiness()
        .rows()
        .iter()
        .find(|row| row.crossing_row().signal_execution_family().is_some())
        .expect("signal row should exist");
    let signal_summary = signal_row
        .signal_authority_summary()
        .expect("signal summary should exist");
    assert_declaration_aspect_projections(
        signal_summary.dependency_aspects().required(),
        &["signal.material_edit"],
    );
    assert_declaration_aspect_projections(
        signal_summary.produced_aspects().required(),
        &["signal.preview_patch"],
    );
}

#[test]
fn relational_and_bridge_checked_inspection_expose_authority_specific_posture() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "preview");
    let relational = handle.route_relational_truth_checked(
        WorthQueryDeclarationRelationalRoutingInput::envelope_checked(
            crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(
                authority_rich_envelope(&handle, "edge:42"),
            ),
        ),
    );
    let bridge = handle.route_bridge_continuation_checked(
        WorthQueryDeclarationBridgeRoutingInput::envelope_checked(
            crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(
                authority_rich_envelope(&handle, "edge:42"),
            ),
        ),
    );

    let relational_inspection = match handle.inspect_declaration_entry(
        WorthQueryDeclarationEntryInspectionInput::relational_routing_checked(relational),
    ) {
        Ok(inspection) => inspection,
        Err(_) => panic!("relational inspection should succeed"),
    };
    let bridge_inspection = match handle.inspect_declaration_entry(
        WorthQueryDeclarationEntryInspectionInput::bridge_routing_checked(bridge),
    ) {
        Ok(inspection) => inspection,
        Err(_) => panic!("bridge inspection should succeed"),
    };

    assert_eq!(
        relational_inspection
            .relational_posture()
            .expect("relational posture should exist")
            .aspect_summary()
            .aspect_fit(),
        WorthQueryDeclarationAspectFit::CompatibleSuperset
    );
    let bridge_posture = bridge_inspection
        .bridge_posture()
        .expect("bridge posture should exist");
    assert!(bridge_posture.denial_cause().is_some());
    assert_eq!(
        bridge_posture.continuation_mode(),
        Some(crate::application::WorthQueryDeclarationBridgeContinuationMode::PreviewSession)
    );
    assert_eq!(
        bridge_posture.truth_context(),
        Some(crate::application::WorthQueryDeclarationBridgeTruthContext::Preview)
    );
    assert_eq!(
        bridge_posture.continuation_family(),
        Some(crate::application::WorthQueryDeclarationBridgeContinuationFamily::PreviewSession)
    );
    assert_ne!(
        bridge_posture.aspect_summary().mapping_fit(),
        WorthQueryDeclarationAspectFit::Exact
    );
    assert!(bridge_posture
        .aspect_summary()
        .mapped_aspects()
        .present()
        .is_empty());
}

#[test]
fn denied_relational_inspection_keeps_unproven_authority_metadata_absent() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "preview");
    let envelope = bridge_signal_envelope(&handle, "edge:42");
    let checked = handle.route_relational_truth_checked(
        WorthQueryDeclarationRelationalRoutingInput::enveloped(envelope),
    );

    let inspection = match handle.inspect_declaration_entry(
        WorthQueryDeclarationEntryInspectionInput::relational_routing_checked(checked),
    ) {
        Ok(inspection) => inspection,
        Err(_) => panic!("relational inspection should succeed"),
    };

    let posture = inspection
        .relational_posture()
        .expect("relational posture should exist");
    assert!(posture.denial_cause().is_some());
    assert_eq!(posture.truth_claim(), None);
    assert_eq!(posture.authority_family(), None);
}

#[test]
fn denied_signal_inspection_keeps_unproven_execution_metadata_absent() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "preview");
    let progressed = handle
        .declare_review_and_progress(Input::<RelationalFamily>::new("edge:42"))
        .unwrap_or_else(|_| panic!("progression should admit"));
    let envelope = handle
        .envelope_routes_from_progressed(progressed)
        .unwrap_or_else(|_| panic!("envelope should succeed"));
    let checked = handle.signal_compatibility_checked(
        WorthQueryDeclarationSignalCompatibilityInput::enveloped(envelope),
    );

    let inspection = match handle.inspect_declaration_entry(
        WorthQueryDeclarationEntryInspectionInput::signal_compatibility_checked(checked),
    ) {
        Ok(inspection) => inspection,
        Err(_) => panic!("signal inspection should succeed"),
    };

    let posture = inspection
        .signal_posture()
        .expect("signal posture should exist");
    assert!(posture.denial_cause().is_some());
    assert_eq!(posture.execution_family(), None);
    assert!(posture.basis_families().is_empty());
}
