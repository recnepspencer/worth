use super::*;

#[test]
fn shared_bridge_rows_still_preserve_query_meaning() {
    let matrix = MilestoneNineOneCertificationAdapter::
        query_subscription_declaration_and_lowering_parity_test();

    let grouped = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "grouped-query-meaning-shares-collection-bridge-family")
        .expect("grouped query meaning row should exist");
    assert_eq!(
        grouped.control_lane.bridge_family,
        grouped.hostile_lane.bridge_family
    );
    assert_ne!(
        grouped.control_lane.query_family,
        grouped.hostile_lane.query_family
    );
    assert_ne!(
        grouped.control_lane.declaration_digest,
        grouped.hostile_lane.declaration_digest
    );

    let inspector = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "inspector-query-meaning-shares-detail-bridge-family")
        .expect("inspector query meaning row should exist");
    assert_eq!(
        inspector.control_lane.bridge_family,
        inspector.hostile_lane.bridge_family
    );
    assert_ne!(
        inspector.control_lane.query_family,
        inspector.hostile_lane.query_family
    );
    assert_ne!(
        inspector.control_lane.declaration_digest,
        inspector.hostile_lane.declaration_digest
    );
}

#[test]
fn activation_certification_binds_scale_source_identity() {
    let matrix = MilestoneNineOneCertificationAdapter::
        query_subscription_declaration_and_lowering_parity_test();

    for row in &matrix.rows {
        assert_eq!(
            row.control_lane.scale_activation_digest, row.control_lane.activation_digest,
            "row '{}' control lane scale proof should bind activation",
            row.row_name
        );
        assert_eq!(
            row.control_lane.scale_admission_digest, row.control_lane.admission_digest,
            "row '{}' control lane scale proof should bind admission",
            row.row_name
        );
    }
}

#[test]
fn basis_request_binds_policy_and_tenant_meaning() {
    let matrix = MilestoneNineOneCertificationAdapter::
        query_subscription_declaration_and_lowering_parity_test();
    let row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "basis-request-binds-policy-tenant-meaning")
        .expect("policy/tenant basis binding row should exist");

    assert_ne!(
        row.control_lane.declaration_digest,
        row.hostile_lane.declaration_digest
    );
    assert_ne!(
        row.control_lane.basis_request_digest,
        row.hostile_lane.basis_request_digest
    );
    assert_eq!(
        row.hostile_lane.basis_request_digest,
        row.parity_lane.basis_request_digest
    );
}

#[test]
fn relationship_proof_binds_subscription_meaning() {
    let matrix = MilestoneNineOneCertificationAdapter::
        query_subscription_declaration_and_lowering_parity_test();
    let row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "relationship-proof-binds-subscription-meaning")
        .expect("relationship proof binding row should exist");

    assert_ne!(
        row.control_lane.declaration_digest,
        row.hostile_lane.declaration_digest
    );
    assert_ne!(
        row.control_lane.basis_request_digest,
        row.hostile_lane.basis_request_digest
    );
    assert_eq!(
        row.hostile_lane.basis_request_digest,
        row.parity_lane.basis_request_digest
    );
}

#[test]
fn rejection_rows_are_typed_and_non_cosmetic() {
    let matrix = MilestoneNineOneCertificationAdapter::
        query_subscription_declaration_and_lowering_parity_test();

    let view = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "view-family-mismatch-denies-before-declaration")
        .expect("view mismatch row should exist");
    assert_eq!(
        view.hostile_lane.failure_class,
        MilestoneNineOneFailureClass::FamilySelectionDenied
    );
    assert_eq!(
        view.hostile_lane.failure_kind,
        "view_family_live_family_mismatch"
    );
    assert_eq!(view.hostile_lane.diagnostic_stage, "view_mismatch");
    assert_ne!(
        view.hostile_lane.diagnostic_digest, view.control_lane.diagnostics_digest,
        "view mismatch row must carry hostile diagnostic evidence"
    );

    let bridge = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "bridge-family-unsupported-denies-before-admission")
        .expect("bridge family row should exist");
    assert_eq!(
        bridge.hostile_lane.failure_class,
        MilestoneNineOneFailureClass::BridgeLoweringDenied
    );
    assert_eq!(
        bridge.hostile_lane.failure_kind,
        "bridge_family_unsupported"
    );
    assert_eq!(
        bridge.hostile_lane.diagnostic_stage,
        "bridge_family_lowering"
    );
    assert_ne!(
        bridge.hostile_lane.diagnostic_digest, bridge.control_lane.diagnostics_digest,
        "bridge family rejection must carry hostile diagnostic evidence"
    );

    for row_name in [
        "masked-detail-slice-denies-before-bridge-lowering",
        "masked-table-ordering-denies-before-bridge-lowering",
        "masked-grouped-membership-denies-before-bridge-lowering",
    ] {
        let masked = matrix
            .rejection_rows
            .iter()
            .find(|row| row.row_name == row_name)
            .expect("masked-slice row should exist");
        assert_eq!(
            masked.hostile_lane.failure_class,
            MilestoneNineOneFailureClass::DeclarationDenied
        );
        assert_eq!(masked.hostile_lane.failure_kind, "unsupported_masked_slice");
        assert_eq!(masked.hostile_lane.diagnostic_stage, "declaration");
        assert_ne!(
            masked.hostile_lane.diagnostic_digest, masked.control_lane.diagnostics_digest,
            "masked slice row '{}' must carry hostile diagnostic evidence",
            row_name
        );
    }

    let broken_proof = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "broken-relationship-proof-denies-before-bridge-lowering")
        .expect("broken relationship proof row should exist");
    assert_eq!(
        broken_proof.hostile_lane.failure_class,
        MilestoneNineOneFailureClass::FamilySelectionDenied
    );
    assert_eq!(
        broken_proof.hostile_lane.failure_kind,
        "relationship_proof_admission_drift"
    );
    assert_eq!(
        broken_proof.hostile_lane.diagnostic_stage,
        "relationship_proof_drift"
    );
    assert_ne!(
        broken_proof.hostile_lane.diagnostic_digest, broken_proof.control_lane.diagnostics_digest,
        "relationship proof drift must carry hostile diagnostic evidence"
    );
    assert_ne!(
        broken_proof.hostile_lane.counter_snapshot_digest,
        bridge.hostile_lane.counter_snapshot_digest,
        "relationship proof drift must not reuse bridge-family counter evidence"
    );

    let durable = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "durable-reload-overclaim-denies-before-activation")
        .expect("durable reload row should exist");
    assert_eq!(
        durable.hostile_lane.failure_class,
        MilestoneNineOneFailureClass::AdmissionDenied
    );
    assert_eq!(
        durable.hostile_lane.failure_kind,
        "durable_reload_overclaim"
    );
    assert_eq!(
        durable.hostile_lane.diagnostic_stage,
        "durable_reload_overclaim"
    );
    assert!(!durable.hostile_lane.support_profile_digest.is_empty());
    assert_ne!(
        durable.hostile_lane.diagnostic_digest, durable.control_lane.diagnostics_digest,
        "durable reload row must carry hostile diagnostic evidence"
    );

    let scale_source = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "scale-report-source-mismatch-denies-certification")
        .expect("scale source mismatch row should exist");
    assert_eq!(
        scale_source.hostile_lane.failure_class,
        MilestoneNineOneFailureClass::CertificationDenied
    );
    assert_eq!(
        scale_source.hostile_lane.failure_kind,
        "scale_slope_source_mismatch"
    );
    assert_eq!(scale_source.hostile_lane.diagnostic_stage, "certification");

    let scale_zero_row = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "scale-zero-row-baseline-denied")
        .expect("scale zero-row rejection should exist");
    assert_eq!(
        scale_zero_row.hostile_lane.failure_class,
        MilestoneNineOneFailureClass::CertificationDenied
    );
    assert_eq!(
        scale_zero_row.hostile_lane.failure_kind,
        "scale_slope_drift"
    );
    assert_eq!(
        scale_zero_row.hostile_lane.diagnostic_stage,
        "certification"
    );

    for row in &matrix.rejection_rows {
        assert!(!row.hostile_lane.diagnostic_digest.is_empty());
        assert_ne!(
            row.hostile_lane.failure_digest, row.hostile_lane.diagnostic_digest,
            "rejection row '{}' failure digest must bind more than the diagnostic alone",
            row.row_name
        );
        assert_ne!(
            row.hostile_lane.counter_snapshot_digest, row.control_lane.counter_snapshot_digest,
            "rejection row '{}' must not reuse control counter evidence",
            row.row_name
        );
    }
}
