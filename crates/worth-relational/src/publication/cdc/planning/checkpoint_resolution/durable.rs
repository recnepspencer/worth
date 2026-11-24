use crate::history::data::PositionedCanonicalCommit;
use crate::publication::cdc::data::SubscriberCheckpoint;
use crate::publication::{durable_canonical_envelopes, retained_canonical_envelope_at_position};
use crate::runtime::RelationalRuntime;

pub(crate) fn durable_checkpoint_envelope(
    runtime: &RelationalRuntime,
    checkpoint: &SubscriberCheckpoint,
    preloaded_envelopes: Option<&[PositionedCanonicalCommit]>,
) -> Option<PositionedCanonicalCommit> {
    retained_canonical_envelope_at_position(
        &runtime.history,
        runtime.durability.canonical_coverage(),
        checkpoint.position(),
        preloaded_envelopes,
    )
}

pub(crate) fn durable_envelopes(runtime: &RelationalRuntime) -> Vec<PositionedCanonicalCommit> {
    durable_canonical_envelopes(runtime.durability.canonical_coverage())
}
