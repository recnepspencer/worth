use crate::history::data::{CanonicalCommitEnvelope, CanonicalCommitEnvelopeBody};

use super::errors::SchemaContinuityBundleIssue;

#[derive(Debug, Clone, Copy)]
pub struct ValidatedSchemaContinuityBundle<'a> {
    envelope: &'a CanonicalCommitEnvelope,
    transition: Option<&'a crate::schema::data::SchemaTransitionArtifact>,
    continuation: Option<&'a crate::schema::data::SchemaContinuationDescriptor>,
    reconciliation: Option<&'a crate::schema::data::SchemaReconciliationDescriptor>,
}

impl<'a> ValidatedSchemaContinuityBundle<'a> {
    pub fn envelope(&self) -> &'a CanonicalCommitEnvelope {
        self.envelope
    }

    pub fn transition(&self) -> Option<&'a crate::schema::data::SchemaTransitionArtifact> {
        self.transition
    }

    pub fn continuation(&self) -> Option<&'a crate::schema::data::SchemaContinuationDescriptor> {
        self.continuation
    }

    pub fn reconciliation(
        &self,
    ) -> Option<&'a crate::schema::data::SchemaReconciliationDescriptor> {
        self.reconciliation
    }
}

pub fn validate_schema_continuity_bundle(
    envelope: &CanonicalCommitEnvelope,
) -> Result<ValidatedSchemaContinuityBundle<'_>, SchemaContinuityBundleIssue> {
    validate_schema_continuity_body(envelope)?;
    Ok(ValidatedSchemaContinuityBundle {
        envelope,
        transition: envelope.schema_transition.as_ref(),
        continuation: envelope.schema_continuation_descriptor.as_ref(),
        reconciliation: envelope.schema_reconciliation_descriptor.as_ref(),
    })
}

pub(crate) fn validate_schema_continuity_body(
    envelope: &CanonicalCommitEnvelopeBody,
) -> Result<(), SchemaContinuityBundleIssue> {
    reject_incomplete_schema_continuity_bundle(envelope)?;
    let Some(transition) = &envelope.schema_transition else {
        return Ok(());
    };

    let continuation = envelope
        .schema_continuation_descriptor
        .as_ref()
        .ok_or(SchemaContinuityBundleIssue::IncompleteBundle)?;
    let reconciliation = envelope
        .schema_reconciliation_descriptor
        .as_ref()
        .ok_or(SchemaContinuityBundleIssue::IncompleteBundle)?;

    reject_transition_descriptor_drift(transition, continuation, reconciliation)?;
    reject_descriptor_version_drift(envelope, continuation, reconciliation)?;
    reject_invalid_visible_bridge_proof(continuation)?;
    reject_envelope_schema_version_drift(envelope, transition, reconciliation)?;
    reject_historical_reinterpretation_violation(continuation)?;

    Ok(())
}

fn reject_incomplete_schema_continuity_bundle(
    envelope: &CanonicalCommitEnvelopeBody,
) -> Result<(), SchemaContinuityBundleIssue> {
    let has_transition = envelope.schema_transition.is_some();
    let has_continuation = envelope.schema_continuation_descriptor.is_some();
    let has_reconciliation = envelope.schema_reconciliation_descriptor.is_some();
    if has_transition != has_continuation || has_transition != has_reconciliation {
        return Err(SchemaContinuityBundleIssue::IncompleteBundle);
    }
    Ok(())
}

fn reject_transition_descriptor_drift(
    transition: &crate::schema::data::SchemaTransitionArtifact,
    continuation: &crate::schema::data::SchemaContinuationDescriptor,
    reconciliation: &crate::schema::data::SchemaReconciliationDescriptor,
) -> Result<(), SchemaContinuityBundleIssue> {
    if transition.continuation_descriptor != *continuation {
        return Err(SchemaContinuityBundleIssue::ContinuationDescriptorDrift {
            boundary_fingerprint: Some(continuation.boundary_fingerprint),
        });
    }
    if transition.reconciliation_descriptor != *reconciliation {
        return Err(SchemaContinuityBundleIssue::ReconciliationDescriptorDrift);
    }
    if continuation.boundary_fingerprint != continuation.bridge.boundary_fingerprint {
        return Err(
            SchemaContinuityBundleIssue::ContinuationBoundaryFingerprintMismatch {
                boundary_fingerprint: continuation.boundary_fingerprint,
            },
        );
    }
    Ok(())
}

fn reject_descriptor_version_drift(
    envelope: &CanonicalCommitEnvelopeBody,
    continuation: &crate::schema::data::SchemaContinuationDescriptor,
    reconciliation: &crate::schema::data::SchemaReconciliationDescriptor,
) -> Result<(), SchemaContinuityBundleIssue> {
    if continuation.bridge.semantics_version != envelope.descriptor_semantics_version {
        return Err(
            SchemaContinuityBundleIssue::DescriptorSemanticsVersionMismatch {
                expected: envelope.descriptor_semantics_version,
                found: continuation.bridge.semantics_version,
            },
        );
    }
    if reconciliation.semantics_version != envelope.descriptor_semantics_version {
        return Err(
            SchemaContinuityBundleIssue::DescriptorSemanticsVersionMismatch {
                expected: envelope.descriptor_semantics_version,
                found: reconciliation.semantics_version,
            },
        );
    }
    if continuation.bridge.canonical_basis_version != reconciliation.canonical_basis_version {
        return Err(
            SchemaContinuityBundleIssue::DescriptorCanonicalBasisVersionMismatch {
                expected: continuation.bridge.canonical_basis_version,
                found: reconciliation.canonical_basis_version,
            },
        );
    }
    Ok(())
}

fn reject_invalid_visible_bridge_proof(
    continuation: &crate::schema::data::SchemaContinuationDescriptor,
) -> Result<(), SchemaContinuityBundleIssue> {
    if continuation.bridge.continuation
        == crate::schema::data::SchemaContinuationClassification::ContinueWithVisibleBridge
        && continuation.bridge.boundary_visibility
            != crate::schema::data::SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable
    {
        return Err(SchemaContinuityBundleIssue::VisibleBridgeProofMismatch);
    }
    Ok(())
}

fn reject_envelope_schema_version_drift(
    envelope: &CanonicalCommitEnvelopeBody,
    transition: &crate::schema::data::SchemaTransitionArtifact,
    reconciliation: &crate::schema::data::SchemaReconciliationDescriptor,
) -> Result<(), SchemaContinuityBundleIssue> {
    if transition.target_schema_version_id != envelope.schema_version {
        return Err(SchemaContinuityBundleIssue::TargetSchemaVersionMismatch);
    }
    if reconciliation.resulting_lineage.resulting_schema_version_id != envelope.schema_version {
        return Err(SchemaContinuityBundleIssue::LineageSchemaVersionMismatch);
    }
    Ok(())
}

fn reject_historical_reinterpretation_violation(
    continuation: &crate::schema::data::SchemaContinuationDescriptor,
) -> Result<(), SchemaContinuityBundleIssue> {
    if continuation.bridge.historical_interpretation
        != crate::schema::data::HistoricalInterpretationSensitivity::NotSensitive
        && matches!(
            continuation.bridge.continuation,
            crate::schema::data::SchemaContinuationClassification::ContinueUnchanged
                | crate::schema::data::SchemaContinuationClassification::ContinueWithTransparentBridge
        )
    {
        return Err(SchemaContinuityBundleIssue::HistoricalReinterpretationViolation);
    }
    Ok(())
}
