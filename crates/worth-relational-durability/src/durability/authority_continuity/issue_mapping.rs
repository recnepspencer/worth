use crate::durability::data::RecoveryAuthorityContinuityCheck;
use crate::durability::data::RecoveryAuthorityParity;
use crate::durability::data::RecoveryVerificationOutcome;
use crate::replay::data::ReplayVerificationLayer;
use branch::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use contracts::durability::data::recovery_errors::RecoveryAuthorityContinuityMismatch;
use contracts::performance::data::runtime_complexity_counters::{
    RuntimeComplexityCounters, RuntimeComplexityCountersOwnerPort as _,
};
use contracts::schema::continuity::errors::SchemaContinuityBundleIssue;
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;

pub(super) fn apply_continuity_issue(
    instrumentation: &RuntimeInstrumentation,
    authority_continuity: &mut RecoveryAuthorityContinuityCheck,
    envelope: &branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope,
    issue: SchemaContinuityBundleIssue,
) {
    let detail = issue.detail();
    instrumentation.count(RuntimeComplexityCounters::count_replay_digest_parity);
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
            apply_descriptor_semantics_version_drift(
                instrumentation,
                authority_continuity,
                expected,
                found,
            )
        }
        SchemaContinuityBundleIssue::DescriptorCanonicalBasisVersionMismatch {
            expected,
            found,
        } => apply_descriptor_canonical_basis_version_drift(
            instrumentation,
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
    envelope: &branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope,
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
    envelope: &branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope,
    boundary_fingerprint: Option<
        contracts::schema::data::continuity::boundary_fingerprint::SchemaBoundaryFingerprint,
    >,
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
    envelope: &branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope,
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
    instrumentation: &RuntimeInstrumentation,
    authority_continuity: &mut RecoveryAuthorityContinuityCheck,
    expected: contracts::schema::data::continuity::descriptor_versions::DescriptorSemanticsVersion,
    found: contracts::schema::data::continuity::descriptor_versions::DescriptorSemanticsVersion,
) {
    instrumentation.count_descriptor_version_mismatch();
    authority_continuity.descriptor_version_parity = RecoveryAuthorityParity::drift();
    authority_continuity.first_mismatch.get_or_insert(
        RecoveryAuthorityContinuityMismatch::DescriptorSemanticsVersion { expected, found },
    );
}

fn apply_descriptor_canonical_basis_version_drift(
    instrumentation: &RuntimeInstrumentation,
    authority_continuity: &mut RecoveryAuthorityContinuityCheck,
    expected: contracts::schema::data::continuity::descriptor_versions::DescriptorCanonicalBasisVersion,
    found: contracts::schema::data::continuity::descriptor_versions::DescriptorCanonicalBasisVersion,
) {
    instrumentation.count_descriptor_version_mismatch();
    authority_continuity.descriptor_version_parity = RecoveryAuthorityParity::drift();
    authority_continuity.first_mismatch.get_or_insert(
        RecoveryAuthorityContinuityMismatch::DescriptorCanonicalBasisVersion { expected, found },
    );
}

fn apply_schema_lineage_drift(
    authority_continuity: &mut RecoveryAuthorityContinuityCheck,
    envelope: &branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope,
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
