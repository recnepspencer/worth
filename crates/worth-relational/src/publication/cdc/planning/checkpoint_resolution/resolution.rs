use crate::capabilities::PatchStreamSource;
use crate::history::data::PositionedCanonicalCommit;
use crate::publication::cdc::data::{
    SubscriberCheckpoint, SubscriberStreamFailure, SubscriberStreamFailureClass,
};
use crate::publication::cdc::diagnostics::{checkpoint_resolution_artifact, rejection_artifact};
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::RelationalRuntime;
use worth_relational_publication::internal_port::publication::cdc::data::subscriber_failure::SubscriberStreamFailureOwnerPort as _;

use super::basis::latest_available_checkpoint;
use super::durable::{durable_checkpoint_envelope, durable_envelopes};
use super::validation::validate_checkpoint_against_envelope;

pub(crate) fn resolve_checkpoint(
    runtime: &RelationalRuntime,
    checkpoint: Option<&SubscriberCheckpoint>,
    durable_envelopes: Option<&[PositionedCanonicalCommit]>,
) -> Result<
    (
        Option<PatchStreamPosition>,
        Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
    ),
    SubscriberStreamFailure,
> {
    let latest = latest_available_checkpoint(runtime);
    let mut diagnostics = vec![checkpoint_resolution_artifact(checkpoint)];

    let Some(checkpoint) = checkpoint else {
        return Ok((None, diagnostics));
    };

    if checkpoint.replay_schema_version().0 != 1 {
        let detail = format!(
            "subscriber checkpoint replay schema version {} is unsupported by runtime replay schema version 1",
            checkpoint.replay_schema_version().0
        );
        diagnostics.push(rejection_artifact(
            SubscriberStreamFailureClass::SchemaUnsupported,
            &detail,
        ));
        return Err(SubscriberStreamFailure::new(
            SubscriberStreamFailureClass::SchemaUnsupported,
            detail,
            latest,
            diagnostics,
        ));
    }

    if let Some(envelope) = runtime.commit_envelope_at_patch_stream_position(checkpoint.position())
    {
        validate_checkpoint_against_envelope(
            checkpoint,
            &envelope,
            "retained canonical",
            latest.clone(),
            &mut diagnostics,
        )?;

        return Ok((Some(checkpoint.position()), diagnostics));
    }

    if let Some(envelope) = durable_checkpoint_envelope(runtime, checkpoint, durable_envelopes) {
        validate_checkpoint_against_envelope(
            checkpoint,
            envelope.envelope(),
            "durable canonical",
            latest.clone(),
            &mut diagnostics,
        )?;
        return Ok((Some(checkpoint.position()), diagnostics));
    }

    let detail = if runtime.contains_patch_stream_position(checkpoint.position()) {
        format!(
            "subscriber checkpoint {} is retained in patch-stream coverage but has no retained canonical envelope or durable recovery coverage",
            checkpoint.position().0
        )
    } else {
        format!(
            "subscriber checkpoint {} is not present in canonical history or durable recovery coverage",
            checkpoint.position().0
        )
    };
    diagnostics.push(rejection_artifact(
        SubscriberStreamFailureClass::DurableCoverageGap,
        &detail,
    ));
    Err(SubscriberStreamFailure::new(
        SubscriberStreamFailureClass::DurableCoverageGap,
        detail,
        latest,
        diagnostics,
    ))
}

pub(crate) fn preloaded_durable_envelopes_for_checkpoint_gap(
    runtime: &RelationalRuntime,
    checkpoint: Option<&SubscriberCheckpoint>,
) -> Option<Vec<PositionedCanonicalCommit>> {
    let checkpoint = checkpoint?;
    if runtime
        .commit_envelope_at_patch_stream_position(checkpoint.position())
        .is_some()
    {
        return None;
    }
    Some(durable_envelopes(runtime))
}
