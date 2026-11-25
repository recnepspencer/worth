use crate::history::data::PositionedCanonicalCommit;
use crate::publication::cdc::data::{
    SubscriberCheckpoint, SubscriberStreamFailure, SubscriberStreamFailureClass,
};
use crate::publication::cdc::planning::checkpoint_resolution::{
    preloaded_durable_envelopes_for_checkpoint_gap, resolve_latest_available_checkpoint,
};
use crate::publication::retained_canonical_envelopes_after;
use crate::runtime::RelationalRuntime;
use worth_relational_publication::internal_port::publication::cdc::data::subscriber_failure::SubscriberStreamFailureOwnerPort as _;

pub(super) enum AvailableEnvelopeSource {
    InMemory(Vec<PositionedCanonicalCommit>),
    Durable(Vec<PositionedCanonicalCommit>),
}

impl AvailableEnvelopeSource {
    pub(super) fn is_durable(&self) -> bool {
        matches!(self, Self::Durable(_))
    }

    pub(super) fn durable_envelopes(&self) -> Option<&[PositionedCanonicalCommit]> {
        match self {
            Self::Durable(envelopes) => Some(envelopes.as_slice()),
            Self::InMemory(_) => None,
        }
    }

    pub(super) fn into_envelopes(self) -> Vec<PositionedCanonicalCommit> {
        match self {
            Self::InMemory(envelopes) | Self::Durable(envelopes) => envelopes,
        }
    }
}

pub(super) fn load_available_envelopes(
    runtime: &RelationalRuntime,
    checkpoint: Option<&SubscriberCheckpoint>,
) -> Result<AvailableEnvelopeSource, SubscriberStreamFailure> {
    let preloaded_durable_envelopes =
        preloaded_durable_envelopes_for_checkpoint_gap(runtime, checkpoint);
    if let Some(envelopes) = preloaded_durable_envelopes {
        return Ok(AvailableEnvelopeSource::Durable(envelopes));
    }

    let start_after_position = checkpoint.map(|checkpoint| checkpoint.position());
    let retained_envelopes =
        retained_canonical_envelopes_after(&runtime.history, runtime.durability.canonical_coverage(), start_after_position, usize::MAX).map_err(
            |gap| {
                let detail = format!(
                    "subscriber stream position {} is retained for commit {} but has no retained canonical envelope or durable recovery coverage",
                    gap.position.0, gap.commit_id.0
                );
                SubscriberStreamFailure::new(
                    SubscriberStreamFailureClass::DurableCoverageGap,
                    detail.clone(),
                    resolve_latest_available_checkpoint(runtime),
                    vec![crate::publication::cdc::diagnostics::rejection_artifact(
                        SubscriberStreamFailureClass::DurableCoverageGap,
                        &detail,
                    )],
                )
            },
        )?;

    Ok(AvailableEnvelopeSource::InMemory(retained_envelopes))
}
