//! One shared physical fate, coupled to its actual committed coordinates.
use super::WorldDurableMediumDenial;
use std::num::NonZeroU64;
use worth_foundational::facade::{CanonicalDigestId, DurablePublicationFate, WorldDurableJournal};

/// Trusted adapter completion data, never a World publication witness.
/// Only the committed constructor accepts coordinates. The binding validates
/// their original subject before minting its cutover proof.
///
/// ```compile_fail,E0451
/// use worth_runtime_world::facade::WorldPublicationCompletion;
/// use worth_foundational::facade::DurablePublicationFate;
/// let _ = WorldPublicationCompletion {
///     fate: DurablePublicationFate::Committed,
///     committed_location: None,
/// };
/// ```
#[derive(Debug)]
pub struct WorldPublicationCompletion {
    fate: DurablePublicationFate<WorldDurableMediumDenial>,
    committed_location: Option<(WorldDurableJournal, NonZeroU64, CanonicalDigestId)>,
}
impl WorldPublicationCompletion {
    pub fn committed(
        journal: WorldDurableJournal,
        position: NonZeroU64,
        digest: CanonicalDigestId,
    ) -> Self {
        Self {
            fate: DurablePublicationFate::Committed,
            committed_location: Some((journal, position, digest)),
        }
    }
    pub fn proven_no_effect(detail: WorldDurableMediumDenial) -> Self {
        Self {
            fate: DurablePublicationFate::ProvenNoEffect { detail },
            committed_location: None,
        }
    }
    pub fn indeterminate(detail: WorldDurableMediumDenial) -> Self {
        Self {
            fate: DurablePublicationFate::Indeterminate { detail },
            committed_location: None,
        }
    }
    pub fn fate(&self) -> &DurablePublicationFate<WorldDurableMediumDenial> {
        &self.fate
    }
    pub(crate) fn into_parts(
        self,
    ) -> (
        DurablePublicationFate<WorldDurableMediumDenial>,
        Option<(WorldDurableJournal, NonZeroU64, CanonicalDigestId)>,
    ) {
        (self.fate, self.committed_location)
    }
}
