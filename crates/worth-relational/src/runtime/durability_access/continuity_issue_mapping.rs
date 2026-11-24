use crate::durability::data::{
    RecoveryAuthorityContinuityCheck, RecoveryAuthorityContinuityMismatch, RecoveryAuthorityParity,
    RecoveryVerificationOutcome,
};
use crate::replay::data::ReplayVerificationLayer;
use crate::runtime::RelationalRuntime;
use crate::schema::SchemaContinuityBundleIssue;

pub(super) fn apply_continuity_issue(
    runtime: &RelationalRuntime,
    authority_continuity: &mut RecoveryAuthorityContinuityCheck,
    envelope: &crate::history::data::CanonicalCommitEnvelope,
    issue: SchemaContinuityBundleIssue,
) {
    let detail = issue.detail();
    runtime
        .performance_access()
        .count_replay_verification_layer(ReplayVerificationLayer::DigestParity);
    authority_continuity.verification_outcome = RecoveryVerificationOutcome::Rejected {
        layer: ReplayVerificationLayer::DigestParity,
        detail: detail.clone(),
    };
    match issue {
        SchemaContinuityBundleIssue::IncompleteBundle => {
            apply_schema_transition_artifact_drift(authority_continuity, envelope, detail)
        }
        SchemaContinuityBundleIssue::ContinuationDescriptorDrift {
            boundary_fingerprint,
        } => apply_continuation_descriptor_drift(
            authority_continuity,
            envelope,
            boundary_fingerprint,
            detail,
        ),
        SchemaContinuityBundleIssue::ReconciliationDescriptorDrift => {
            apply_reconciliation_descriptor_drift(authority_continuity, envelope, detail)
        }
        SchemaContinuityBundleIssue::ContinuationBoundaryFingerprintMismatch {
            boundary_fingerprint,
        } => apply_continuation_descriptor_drift(
            authority_continuity,
            envelope,
            Some(boundary_fingerprint),
            detail,
        ),
        SchemaContinuityBundleIssue::DescriptorSemanticsVersionMismatch { expected, found } => {
            apply_descriptor_semantics_version_drift(runtime, authority_continuity, expected, found)
        }
        SchemaContinuityBundleIssue::DescriptorCanonicalBasisVersionMismatch {
            expected,
            found,
        } => apply_descriptor_canonical_basis_version_drift(
            runtime,
            authority_continuity,
            expected,
            found,
        ),
        SchemaContinuityBundleIssue::VisibleBridgeProofMismatch => {
            apply_continuation_descriptor_drift(
                authority_continuity,
                envelope,
                envelope
                    .schema_continuation_descriptor
                    .as_ref()
                    .map(|descriptor| descriptor.boundary_fingerprint),
                detail,
            )
        }
        SchemaContinuityBundleIssue::TargetSchemaVersionMismatch => {
            apply_schema_transition_artifact_drift(authority_continuity, envelope, detail)
        }
        SchemaContinuityBundleIssue::LineageSchemaVersionMismatch => {
            apply_schema_lineage_drift(authority_continuity, envelope, detail)
        }
        SchemaContinuityBundleIssue::HistoricalReinterpretationViolation => {
            apply_continuation_descriptor_drift(
                authority_continuity,
                envelope,
                envelope
                    .schema_continuation_descriptor
                    .as_ref()
                    .map(|descriptor| descriptor.boundary_fingerprint),
                detail,
            )
        }
    }
}

fn apply_schema_transition_artifact_drift(
    authority_continuity: &mut RecoveryAuthorityContinuityCheck,
    envelope: &crate::history::data::CanonicalCommitEnvelope,
    detail: String,
) {
    authority_continuity.schema_transition_parity = RecoveryAuthorityParity::drift();
    authority_continuity.first_mismatch.get_or_insert(
        RecoveryAuthorityContinuityMismatch::SchemaTransitionArtifact {
            commit_id: envelope.commit.commit_id.0,
            detail,
        },
    );
}

fn apply_continuation_descriptor_drift(
    authority_continuity: &mut RecoveryAuthorityContinuityCheck,
    envelope: &crate::history::data::CanonicalCommitEnvelope,
    boundary_fingerprint: Option<crate::schema::data::SchemaBoundaryFingerprint>,
    detail: String,
) {
    authority_continuity.continuation_descriptor_parity = RecoveryAuthorityParity::drift();
    authority_continuity.first_mismatch.get_or_insert(
        RecoveryAuthorityContinuityMismatch::ContinuationDescriptor {
            commit_id: envelope.commit.commit_id.0,
            boundary_fingerprint,
            detail,
        },
    );
}

fn apply_reconciliation_descriptor_drift(
    authority_continuity: &mut RecoveryAuthorityContinuityCheck,
    envelope: &crate::history::data::CanonicalCommitEnvelope,
    detail: String,
) {
    authority_continuity.reconciliation_descriptor_parity = RecoveryAuthorityParity::drift();
    authority_continuity.first_mismatch.get_or_insert(
        RecoveryAuthorityContinuityMismatch::ReconciliationDescriptor {
            commit_id: envelope.commit.commit_id.0,
            detail,
        },
    );
}

fn apply_descriptor_semantics_version_drift(
    runtime: &RelationalRuntime,
    authority_continuity: &mut RecoveryAuthorityContinuityCheck,
    expected: crate::schema::data::DescriptorSemanticsVersion,
    found: crate::schema::data::DescriptorSemanticsVersion,
) {
    runtime
        .performance_access()
        .count_descriptor_version_mismatch();
    authority_continuity.descriptor_version_parity = RecoveryAuthorityParity::drift();
    authority_continuity.first_mismatch.get_or_insert(
        RecoveryAuthorityContinuityMismatch::DescriptorSemanticsVersion { expected, found },
    );
}

fn apply_descriptor_canonical_basis_version_drift(
    runtime: &RelationalRuntime,
    authority_continuity: &mut RecoveryAuthorityContinuityCheck,
    expected: crate::schema::data::DescriptorCanonicalBasisVersion,
    found: crate::schema::data::DescriptorCanonicalBasisVersion,
) {
    runtime
        .performance_access()
        .count_descriptor_version_mismatch();
    authority_continuity.descriptor_version_parity = RecoveryAuthorityParity::drift();
    authority_continuity.first_mismatch.get_or_insert(
        RecoveryAuthorityContinuityMismatch::DescriptorCanonicalBasisVersion { expected, found },
    );
}

fn apply_schema_lineage_drift(
    authority_continuity: &mut RecoveryAuthorityContinuityCheck,
    envelope: &crate::history::data::CanonicalCommitEnvelope,
    detail: String,
) {
    authority_continuity.schema_lineage_parity = RecoveryAuthorityParity::drift();
    authority_continuity.first_mismatch.get_or_insert(
        RecoveryAuthorityContinuityMismatch::SchemaLineage {
            commit_id: envelope.commit.commit_id.0,
            detail,
        },
    );
}
