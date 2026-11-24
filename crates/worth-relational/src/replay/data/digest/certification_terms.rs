use crate::diagnostics::data::RelationalDiagnosticArtifact;
use crate::publication::cdc::data::subscriber_contract::SubscriberContinuationSummary;
use crate::publication::cdc::data::subscriber_decision::SubscriberRecoveryDecision;
use crate::publication::patch::data::PublishedAuthoritativePatchEnvelope;
use crate::schema::data::{
    DescriptorSemanticsVersion, SchemaBoundaryFingerprint, SchemaContinuationDescriptor,
    SchemaReconciliationDescriptor,
};

use super::primitive_terms::ReplayDigestBuilder;
use super::{
    digest_diagnostics_surface, digest_patch_surface, digest_schema_continuation_descriptor,
    digest_schema_reconciliation_descriptor,
};

pub(crate) fn digest_schema_transition_decision(
    continuation: &SchemaContinuationDescriptor,
    reconciliation: &SchemaReconciliationDescriptor,
    semantics_version: DescriptorSemanticsVersion,
) -> [u8; 32] {
    ReplayDigestBuilder::new("worth.relational.replay.decision.schema_transition.v1")
        .digest_bytes(&digest_schema_continuation_descriptor(
            continuation,
            semantics_version,
        ))
        .digest_bytes(&digest_schema_reconciliation_descriptor(
            reconciliation,
            semantics_version,
        ))
        .finish()
}

pub(crate) fn digest_subscriber_boundary_cdc_surface(
    patches: &[PublishedAuthoritativePatchEnvelope],
    crossed_boundaries: &[SchemaBoundaryFingerprint],
    continuation_summary: &SubscriberContinuationSummary,
    recovery_decision: &SubscriberRecoveryDecision,
) -> [u8; 32] {
    let mut builder = ReplayDigestBuilder::new("worth.relational.replay.surface.schema_cdc.v1")
        .usize(patches.len());
    for patch in patches {
        builder = builder.digest_bytes(&digest_patch_surface(patch));
    }
    builder = builder.usize(crossed_boundaries.len());
    for boundary in crossed_boundaries {
        builder = builder.boundary_fingerprint(*boundary);
    }
    builder
        .digest_bytes(&digest_subscriber_continuation_summary(
            continuation_summary,
        ))
        .label(recovery_decision.disposition)
        .label(recovery_decision.source)
        .optional_patch_stream_position(recovery_decision.start_after_position)
        .finish()
}

pub(crate) fn digest_subscriber_continuation_summary(
    summary: &SubscriberContinuationSummary,
) -> [u8; 32] {
    ReplayDigestBuilder::new("worth.relational.replay.summary.subscriber_continuation.v1")
        .string(&summary.contract_id)
        .label(summary.continuation_outcome)
        .usize(summary.crossed_boundary_count)
        .usize(summary.normalized_boundary_count)
        .descriptor_semantics_version(summary.descriptor_semantics_version)
        .bool(summary.contract_upgrade_applied)
        .finish()
}

pub(crate) fn digest_patch_batch_surface(
    patches: &[PublishedAuthoritativePatchEnvelope],
) -> [u8; 32] {
    let mut builder = ReplayDigestBuilder::new("worth.relational.replay.surface.patch_batch.v1")
        .usize(patches.len());
    for patch in patches {
        builder = builder.digest_bytes(&digest_patch_surface(patch));
    }
    builder.finish()
}

pub(crate) fn digest_diagnostics_batch_surface(
    diagnostics: &[RelationalDiagnosticArtifact],
) -> [u8; 32] {
    let mut builder =
        ReplayDigestBuilder::new("worth.relational.replay.surface.diagnostics_batch.v1")
            .usize(diagnostics.len());
    for diagnostic in diagnostics {
        builder = builder.digest_bytes(&digest_diagnostics_surface(diagnostic));
    }
    builder.finish()
}

pub(crate) fn digest_subscriber_continuation_counter_pair(
    visible_bridge_count: usize,
    normalized_descriptor_compositions: usize,
) -> [u8; 32] {
    ReplayDigestBuilder::new("worth.relational.replay.summary.subscriber_continuation_counters.v1")
        .usize(visible_bridge_count)
        .usize(normalized_descriptor_compositions)
        .finish()
}
