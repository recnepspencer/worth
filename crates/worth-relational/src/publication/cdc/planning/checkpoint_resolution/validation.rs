use crate::history::data::CanonicalCommitEnvelope;
use crate::publication::cdc::data::{
    SubscriberCheckpoint, SubscriberStreamFailure, SubscriberStreamFailureClass,
};
use crate::publication::cdc::diagnostics::rejection_artifact;

pub(crate) fn validate_checkpoint_against_envelope(
    checkpoint: &SubscriberCheckpoint,
    envelope: &CanonicalCommitEnvelope,
    authority_label: &str,
    latest: Option<SubscriberCheckpoint>,
    diagnostics: &mut Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
) -> Result<(), SubscriberStreamFailure> {
    if envelope.schema_version != checkpoint.schema_version() {
        return checkpoint_validation_error(
            SubscriberStreamFailureClass::SchemaUnsupported,
            format!(
                "subscriber checkpoint schema version {} does not match {authority_label} schema version {}",
                checkpoint.schema_version().0,
                envelope.schema_version.0
            ),
            latest,
            diagnostics,
        );
    }

    if envelope.descriptor_semantics_version != checkpoint.descriptor_semantics_version() {
        return checkpoint_validation_error(
            SubscriberStreamFailureClass::DescriptorVersionMismatch,
            format!(
                "subscriber checkpoint descriptor semantics version {} does not match {authority_label} descriptor semantics version {}",
                checkpoint.descriptor_semantics_version().0,
                envelope.descriptor_semantics_version.0
            ),
            latest,
            diagnostics,
        );
    }

    if checkpoint
        .normalized_continuation_proof()
        .descriptor_semantics_version()
        != checkpoint.descriptor_semantics_version()
    {
        return checkpoint_validation_error(
            SubscriberStreamFailureClass::DescriptorVersionMismatch,
            format!(
                "subscriber checkpoint normalized proof descriptor semantics version {} does not match checkpoint descriptor semantics version {}",
                checkpoint
                    .normalized_continuation_proof()
                    .descriptor_semantics_version()
                    .0,
                checkpoint.descriptor_semantics_version().0
            ),
            latest,
            diagnostics,
        );
    }

    if checkpoint.continuation_summary().contract_id != checkpoint.subscriber_contract_id() {
        return checkpoint_validation_error(
            SubscriberStreamFailureClass::CheckpointContinuitySummaryMismatch,
            format!(
                "subscriber checkpoint continuation summary contract {} does not match checkpoint contract {}",
                checkpoint.continuation_summary().contract_id,
                checkpoint.subscriber_contract_id()
            ),
            latest,
            diagnostics,
        );
    }

    if checkpoint
        .continuation_summary()
        .descriptor_semantics_version
        != checkpoint.descriptor_semantics_version()
    {
        return checkpoint_validation_error(
            SubscriberStreamFailureClass::CheckpointContinuitySummaryMismatch,
            format!(
                "subscriber checkpoint continuation summary descriptor semantics version {} does not match checkpoint descriptor semantics version {}",
                checkpoint.continuation_summary().descriptor_semantics_version.0,
                checkpoint.descriptor_semantics_version().0
            ),
            latest,
            diagnostics,
        );
    }

    if checkpoint.continuation_summary().normalized_boundary_count
        != checkpoint
            .normalized_continuation_proof()
            .normalized_boundary_count()
    {
        return checkpoint_validation_error(
            SubscriberStreamFailureClass::CheckpointContinuitySummaryMismatch,
            format!(
                "subscriber checkpoint continuation summary normalized boundary count {} does not match checkpoint proof normalized boundary count {}",
                checkpoint.continuation_summary().normalized_boundary_count,
                checkpoint
                    .normalized_continuation_proof()
                    .normalized_boundary_count()
            ),
            latest,
            diagnostics,
        );
    }

    match &envelope.schema_continuation_descriptor {
        Some(descriptor) => {
            if checkpoint.authoritative_boundary_fingerprint()
                != Some(descriptor.boundary_fingerprint)
            {
                return checkpoint_validation_error(
                    SubscriberStreamFailureClass::CheckpointContinuitySummaryMismatch,
                    format!(
                        "subscriber checkpoint authoritative boundary fingerprint {:?} does not match {authority_label} continuation boundary {:?}",
                        checkpoint.authoritative_boundary_fingerprint(),
                        descriptor.boundary_fingerprint
                    ),
                    latest,
                    diagnostics,
                );
            }
            if checkpoint.authoritative_descriptor_continuation()
                != Some(descriptor.bridge.continuation)
            {
                return checkpoint_validation_error(
                    SubscriberStreamFailureClass::CheckpointContinuitySummaryMismatch,
                    format!(
                        "subscriber checkpoint authoritative descriptor continuation {:?} does not match {authority_label} continuation {:?}",
                        checkpoint.authoritative_descriptor_continuation(),
                        descriptor.bridge.continuation
                    ),
                    latest,
                    diagnostics,
                );
            }
            if checkpoint.authoritative_contract_consumes_boundary()
                && !checkpoint
                    .normalized_continuation_proof()
                    .boundary_fingerprints()
                    .contains(&descriptor.boundary_fingerprint)
            {
                return checkpoint_validation_error(
                    SubscriberStreamFailureClass::CheckpointContinuitySummaryMismatch,
                    format!(
                        "subscriber checkpoint normalized continuation proof does not contain authoritative boundary {:?}",
                        descriptor.boundary_fingerprint
                    ),
                    latest,
                    diagnostics,
                );
            }
            if checkpoint
                .authoritative_subscriber_outcome()
                .is_some_and(|outcome| {
                    continuation_priority(checkpoint.continuation_summary().continuation_outcome)
                        < continuation_priority(outcome)
                })
            {
                return checkpoint_validation_error(
                    SubscriberStreamFailureClass::CheckpointContinuitySummaryMismatch,
                    format!(
                        "subscriber checkpoint continuation outcome {:?} is weaker than authoritative boundary outcome {:?}",
                        checkpoint.continuation_summary().continuation_outcome,
                        checkpoint.authoritative_subscriber_outcome()
                    ),
                    latest,
                    diagnostics,
                );
            }
        }
        None => {
            if checkpoint.authoritative_boundary_fingerprint().is_some()
                || checkpoint.authoritative_descriptor_continuation().is_some()
                || checkpoint.authoritative_subscriber_outcome().is_some()
                || checkpoint.authoritative_contract_consumes_boundary()
            {
                return checkpoint_validation_error(
                    SubscriberStreamFailureClass::CheckpointContinuitySummaryMismatch,
                    format!(
                        "subscriber checkpoint carries authoritative boundary binding for position {} but {authority_label} canonical envelope has no continuation descriptor",
                        checkpoint.position().0
                    ),
                    latest,
                    diagnostics,
                );
            }
        }
    }

    Ok(())
}

fn checkpoint_validation_error(
    class: SubscriberStreamFailureClass,
    detail: String,
    latest: Option<SubscriberCheckpoint>,
    diagnostics: &mut Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
) -> Result<(), SubscriberStreamFailure> {
    diagnostics.push(rejection_artifact(class, &detail));
    Err(SubscriberStreamFailure::new(
        class,
        detail,
        latest,
        diagnostics.clone(),
    ))
}

fn continuation_priority(
    classification: crate::schema::data::SchemaContinuationClassification,
) -> u8 {
    match classification {
        crate::schema::data::SchemaContinuationClassification::ContinueUnchanged => 0,
        crate::schema::data::SchemaContinuationClassification::ContinueWithTransparentBridge => 1,
        crate::schema::data::SchemaContinuationClassification::ContinueWithVisibleBridge => 2,
        crate::schema::data::SchemaContinuationClassification::ContinueWithContractUpgrade => 3,
        crate::schema::data::SchemaContinuationClassification::RequireRenegotiation => 4,
        crate::schema::data::SchemaContinuationClassification::Rejected => 5,
        _ => unreachable!("unhandled Relational contract variant"),
    }
}
