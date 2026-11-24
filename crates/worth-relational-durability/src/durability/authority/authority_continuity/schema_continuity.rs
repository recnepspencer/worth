use crate::durability::data::RecoveryPlan;
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use branch::schema::continuity::bundle_admission::validate_schema_continuity_bundle;
use contracts::config::data::runtime_config::RelationalRuntimeConfig;
use contracts::durability::data::recovery_errors::{
    DurabilityError, RecoveryAuthorityContinuityMismatch, RecoveryFailureClass,
};
use contracts::schema::continuity::errors::SchemaContinuityBundleIssue;
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;

pub(super) fn validate_schema_recovery_authority_continuity(
    config: &RelationalRuntimeConfig,
    plan: &RecoveryPlan,
) -> Result<(), DurabilityError> {
    let descriptor_policy = config.schema.descriptor_semantics_policy.clone();
    let canonical_basis_policy = config.schema.descriptor_canonical_basis_policy.clone();
    let runtime_descriptor_version = descriptor_policy.current_write_version();
    let runtime_canonical_basis_version = canonical_basis_policy.current_write_version();
    if !descriptor_policy.supports(plan.descriptor_semantics_version) {
        return Err(DurabilityError::new(
            RecoveryFailureClass::SchemaMismatch,
            "recovery descriptor semantics version mismatch",
        )
        .with_authority_continuity_mismatch(
            RecoveryAuthorityContinuityMismatch::DescriptorSemanticsVersion {
                expected: plan.descriptor_semantics_version,
                found: runtime_descriptor_version,
            },
        ));
    }

    let checkpoint_envelopes = plan
        .checkpoint
        .as_ref()
        .map(|checkpoint| checkpoint.envelopes.as_slice())
        .unwrap_or(&[]);
    let tail_envelopes = plan.tail_envelopes_in_stream_order();
    for envelope in checkpoint_envelopes
        .iter()
        .map(
            branch::history::data::positioned_canonical_commit::PositionedCanonicalCommit::envelope,
        )
        .chain(tail_envelopes)
    {
        if envelope.descriptor_semantics_version != plan.descriptor_semantics_version {
            return Err(DurabilityError::new(
                RecoveryFailureClass::SchemaMismatch,
                "recovery envelope descriptor semantics version mismatch",
            )
            .with_authority_continuity_mismatch(
                RecoveryAuthorityContinuityMismatch::DescriptorSemanticsVersion {
                    expected: plan.descriptor_semantics_version,
                    found: envelope.descriptor_semantics_version,
                },
            ));
        }

        if let Some(found) = envelope
            .schema_continuation_descriptor
            .as_ref()
            .map(|descriptor| descriptor.bridge.canonical_basis_version)
            .into_iter()
            .chain(
                envelope
                    .schema_reconciliation_descriptor
                    .as_ref()
                    .map(|descriptor| descriptor.canonical_basis_version),
            )
            .find(|version| !canonical_basis_policy.supports(*version))
        {
            return Err(DurabilityError::new(
                RecoveryFailureClass::SchemaMismatch,
                "recovery envelope descriptor canonical basis version mismatch",
            )
            .with_authority_continuity_mismatch(
                RecoveryAuthorityContinuityMismatch::DescriptorCanonicalBasisVersion {
                    expected: runtime_canonical_basis_version,
                    found,
                },
            ));
        }

        validate_schema_continuity_bundle(envelope)
            .map_err(|issue| schema_continuity_recovery_error(envelope, issue))?;
    }

    Ok(())
}

fn schema_continuity_recovery_error(
    envelope: &CanonicalCommitEnvelope,
    issue: SchemaContinuityBundleIssue,
) -> DurabilityError {
    let detail = issue.detail();
    let mismatch = match issue {
        SchemaContinuityBundleIssue::IncompleteBundle => {
            RecoveryAuthorityContinuityMismatch::SchemaTransitionArtifact {
                commit_id: envelope.commit.commit_id.0,
                detail,
            }
        }
        SchemaContinuityBundleIssue::ContinuationDescriptorDrift {
            boundary_fingerprint,
        } => RecoveryAuthorityContinuityMismatch::ContinuationDescriptor {
            commit_id: envelope.commit.commit_id.0,
            boundary_fingerprint,
            detail,
        },
        SchemaContinuityBundleIssue::ContinuationBoundaryFingerprintMismatch {
            boundary_fingerprint,
        } => RecoveryAuthorityContinuityMismatch::ContinuationDescriptor {
            commit_id: envelope.commit.commit_id.0,
            boundary_fingerprint: Some(boundary_fingerprint),
            detail,
        },
        SchemaContinuityBundleIssue::DescriptorSemanticsVersionMismatch { expected, found } => {
            RecoveryAuthorityContinuityMismatch::DescriptorSemanticsVersion { expected, found }
        }
        SchemaContinuityBundleIssue::DescriptorCanonicalBasisVersionMismatch {
            expected,
            found,
        } => {
            RecoveryAuthorityContinuityMismatch::DescriptorCanonicalBasisVersion { expected, found }
        }
        SchemaContinuityBundleIssue::VisibleBridgeProofMismatch => {
            RecoveryAuthorityContinuityMismatch::ContinuationDescriptor {
                commit_id: envelope.commit.commit_id.0,
                boundary_fingerprint: envelope
                    .schema_continuation_descriptor
                    .as_ref()
                    .map(|descriptor| descriptor.boundary_fingerprint),
                detail,
            }
        }
        SchemaContinuityBundleIssue::ReconciliationDescriptorDrift => {
            RecoveryAuthorityContinuityMismatch::ReconciliationDescriptor {
                commit_id: envelope.commit.commit_id.0,
                detail,
            }
        }
        SchemaContinuityBundleIssue::TargetSchemaVersionMismatch => {
            RecoveryAuthorityContinuityMismatch::SchemaTransitionArtifact {
                commit_id: envelope.commit.commit_id.0,
                detail,
            }
        }
        SchemaContinuityBundleIssue::LineageSchemaVersionMismatch => {
            RecoveryAuthorityContinuityMismatch::SchemaLineage {
                commit_id: envelope.commit.commit_id.0,
                detail,
            }
        }
        SchemaContinuityBundleIssue::HistoricalReinterpretationViolation => {
            RecoveryAuthorityContinuityMismatch::ContinuationDescriptor {
                commit_id: envelope.commit.commit_id.0,
                boundary_fingerprint: envelope
                    .schema_continuation_descriptor
                    .as_ref()
                    .map(|descriptor| descriptor.boundary_fingerprint),
                detail,
            }
        }
    };
    DurabilityError::new(
        RecoveryFailureClass::SchemaMismatch,
        "recovery schema continuity authority failure",
    )
    .with_authority_continuity_mismatch(mismatch)
}
