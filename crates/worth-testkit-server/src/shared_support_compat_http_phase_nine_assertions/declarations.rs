pub use {worth_foundational::facade::DiagnosticRichnessProfile};
pub use {worth_server::WorthServerCompatibilityFileEnvelope, worth_server::WorthServerFileTransferDisposition, worth_server::WorthServerQueryHandoffDenial, worth_server::WorthServerQueryHandoffDenialCode};

pub fn assert_same_metadata_identity(
    left: &WorthServerCompatibilityFileEnvelope,
    right: &WorthServerCompatibilityFileEnvelope,
) {
    assert_eq!(
        left.metadata_receipt().metadata_identity(),
        right.metadata_receipt().metadata_identity()
    );
    assert_eq!(
        left.metadata_receipt().tenant_id(),
        right.metadata_receipt().tenant_id()
    );
    assert_eq!(
        left.metadata_receipt().workspace_digest(),
        right.metadata_receipt().workspace_digest()
    );
    assert_eq!(
        left.metadata_receipt().branch_digest(),
        right.metadata_receipt().branch_digest()
    );
}

pub fn assert_policy_alignment(envelope: &WorthServerCompatibilityFileEnvelope) {
    assert_eq!(
        envelope.metadata_receipt().metadata_identity(),
        envelope.policy_decision().metadata_identity()
    );
    assert_eq!(
        envelope.metadata_receipt().workspace_digest(),
        envelope.policy_decision().workspace_digest()
    );
    assert_eq!(
        envelope.metadata_receipt().branch_digest(),
        envelope.policy_decision().branch_digest()
    );
    assert_eq!(
        envelope.metadata_receipt().metadata_identity(),
        envelope.transfer_provenance().metadata_identity()
    );
}

pub fn assert_transfer_disposition(
    envelope: &WorthServerCompatibilityFileEnvelope,
    expected: WorthServerFileTransferDisposition,
) {
    assert_eq!(envelope.transfer_provenance().disposition(), expected);
    assert_eq!(
        envelope.transfer_provenance().byte_motion_observed(),
        !matches!(
            expected,
            worth_server::WorthServerFileTransferDisposition::MetadataOnlyObservation
                | worth_server::WorthServerFileTransferDisposition::HeadOnlyEgress
        )
    );
}

pub fn assert_diagnostics_profile(
    envelope: &WorthServerCompatibilityFileEnvelope,
    expected: DiagnosticRichnessProfile,
) {
    assert_eq!(envelope.policy_decision().diagnostics_profile(), expected);
    assert_eq!(
        envelope.transfer_provenance().diagnostics_profile(),
        expected
    );
}

pub fn assert_denial(
    denial: &WorthServerQueryHandoffDenial,
    expected_code: WorthServerQueryHandoffDenialCode,
    detail_fragment: &str,
) {
    assert_eq!(denial.code(), expected_code);
    assert!(
        denial.detail().contains(detail_fragment),
        "expected denial detail `{}` to contain `{detail_fragment}`",
        denial.detail()
    );
}