//! Preserve native publication posture when a source supplies an envelope.
use crate::adapter::{RelationalBridgeSourceError, RelationalBridgeSourceErrorTag as Kind};
use crate::input::envelope::BridgeCommittedPatchEnvelope;
use crate::relational_source::RelationalBridgePublicationOutcome;
use worth_proof::TransitionOutcome;

pub(super) fn publication_envelope(
    outcome: RelationalBridgePublicationOutcome,
) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
    match outcome {
        TransitionOutcome::Success(publication) => Ok(publication.into_bridge_envelope()),
        TransitionOutcome::Denied(denial) => {
            let message = format!("Relational publication envelope was denied: {denial}");
            Err(RelationalBridgeSourceError::new(
                Kind::PublicationDenied(denial),
                message,
            ))
        }
        TransitionOutcome::Deferred(cause) => Err(RelationalBridgeSourceError::new(
            Kind::PublicationDeferred(cause),
            "Relational publication is deferred",
        )),
        TransitionOutcome::Stale(cause) => Err(RelationalBridgeSourceError::new(
            Kind::PublicationStale(cause),
            "Relational publication authority is stale",
        )),
        TransitionOutcome::RebindRequired(cause) => Err(RelationalBridgeSourceError::new(
            Kind::PublicationRebindRequired(cause),
            "Relational publication requires graph rebind",
        )),
        TransitionOutcome::Failed(never) => match never {},
    }
}
