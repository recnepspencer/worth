//! The exact proven-no-effect subject travels with its refusal.
use super::*;
use worth_proof::ProvenNoEffectPublication;
#[derive(Debug)]
pub(crate) struct ActualWorldNoEffectPublication {
    subject: WorldDurableArtifact,
}
impl AuthorityMarker for ActualWorldNoEffectPublication {}
impl ActualWorldNoEffectPublication {
    pub(crate) fn subject(&self) -> &WorldDurableArtifact {
        &self.subject
    }
}
#[derive(Debug)]
pub(crate) struct WorldPublicationRefusal {
    proof: ProvenNoEffectPublication<ActualWorldNoEffectPublication>,
    cause: WorldDurableMediumDenial,
}
impl WorldPublicationRefusal {
    pub(super) fn new(subject: WorldDurableArtifact, cause: WorldDurableMediumDenial) -> Self {
        Self {
            proof: ProvenNoEffectPublication::from_owner(ActualWorldNoEffectPublication {
                subject,
            }),
            cause,
        }
    }
    pub(crate) fn proof(&self) -> &ProvenNoEffectPublication<ActualWorldNoEffectPublication> {
        &self.proof
    }
    pub(crate) fn into_parts(
        self,
    ) -> (
        ProvenNoEffectPublication<ActualWorldNoEffectPublication>,
        WorldDurableMediumDenial,
    ) {
        (self.proof, self.cause)
    }
    pub(crate) fn into_cause(self) -> WorldDurableMediumDenial {
        self.cause
    }
}
