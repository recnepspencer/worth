use crate::capabilities::{PatchStreamSource, RuntimeConfigSource};
use crate::history::data::PositionedCanonicalCommit;
use crate::publication::cdc::data::{
    SubscriberCheckpoint, SubscriberCheckpointBasis, SubscriberContinuationAssessment,
};
use crate::publication::durable_canonical_envelopes;
use crate::publication::patch::data::PatchStreamPosition;
use crate::replay::data::ReplaySchemaVersion;
use crate::runtime::RelationalRuntime;
#[cfg(test)]
use crate::schema::data::SchemaVersionId;

pub(crate) fn latest_available_checkpoint(
    runtime: &RelationalRuntime,
) -> Option<SubscriberCheckpoint> {
    let basis = latest_available_checkpoint_basis(runtime)?;
    Some(SubscriberCheckpoint::from_basis_with_assessment(
        basis,
        "default.subscriber.contract".to_string(),
        &SubscriberContinuationAssessment::unchanged(
            "default.subscriber.contract".to_string(),
            runtime
                .runtime_config()
                .schema
                .descriptor_semantics_policy
                .current_write_version(),
        ),
        runtime
            .runtime_config()
            .schema
            .descriptor_semantics_policy
            .current_write_version(),
    ))
}

pub(crate) fn latest_available_checkpoint_basis(
    runtime: &RelationalRuntime,
) -> Option<SubscriberCheckpointBasis> {
    if let Some(latest_position) = runtime.latest_patch_stream_position()
        && let Some(basis) = checkpoint_basis_from_patch_position(runtime, latest_position)
    {
        return Some(basis);
    }

    durable_canonical_envelopes(runtime)
        .last()
        .map(checkpoint_basis_from_envelope)
}

pub(crate) fn checkpoint_basis_from_patch_position(
    runtime: &RelationalRuntime,
    position: PatchStreamPosition,
) -> Option<SubscriberCheckpointBasis> {
    let envelope = runtime.commit_envelope_at_patch_stream_position(position)?;
    Some(SubscriberCheckpointBasis::new(
        position,
        ReplaySchemaVersion(1),
        envelope.schema_version,
    ))
}

pub(crate) fn checkpoint_basis_from_envelope(
    envelope: &PositionedCanonicalCommit,
) -> SubscriberCheckpointBasis {
    SubscriberCheckpointBasis::new(
        envelope.position(),
        ReplaySchemaVersion(1),
        envelope.schema_version,
    )
}

#[cfg(test)]
pub(crate) fn checkpoint_for_schema_version(
    position: PatchStreamPosition,
    schema_version: SchemaVersionId,
) -> SubscriberCheckpoint {
    SubscriberCheckpoint::from_basis_with_assessment(
        SubscriberCheckpointBasis::new(position, ReplaySchemaVersion(1), schema_version),
        "default.subscriber.contract".to_string(),
        &SubscriberContinuationAssessment::unchanged(
            "default.subscriber.contract".to_string(),
            crate::schema::data::runtime_descriptor_semantics_policy().current_write_version(),
        ),
        crate::schema::data::runtime_descriptor_semantics_policy().current_write_version(),
    )
}
