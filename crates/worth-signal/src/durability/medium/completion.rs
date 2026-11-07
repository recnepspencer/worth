use super::SignalDurableMediumDenial;
use worth_foundational::facade::{DurablePublicationFate, SignalDurableJournal};

/// Adapter observation tied to the original frame digest and journal position.
/// Only Signal's medium binding may turn a Committed answer into live proof.
#[derive(Debug)]
pub struct SignalPublicationCompletion {
    fate: DurablePublicationFate<SignalDurableMediumDenial>,
    location: Option<(SignalDurableJournal, u64, [u8; 32])>,
}
impl SignalPublicationCompletion {
    pub(crate) fn into_original_parts(
        self,
    ) -> (
        DurablePublicationFate<SignalDurableMediumDenial>,
        Option<(SignalDurableJournal, u64, [u8; 32])>,
    ) {
        (self.fate, self.location)
    }
    pub fn committed(
        journal: SignalDurableJournal,
        position: std::num::NonZeroU64,
        digest: [u8; 32],
    ) -> Self {
        Self {
            fate: DurablePublicationFate::Committed,
            location: Some((journal, position.get(), digest)),
        }
    }
    pub fn proven_no_effect(detail: SignalDurableMediumDenial) -> Self {
        Self {
            fate: DurablePublicationFate::ProvenNoEffect { detail },
            location: None,
        }
    }
    pub fn indeterminate(detail: SignalDurableMediumDenial) -> Self {
        Self {
            fate: DurablePublicationFate::Indeterminate { detail },
            location: None,
        }
    }
    pub fn fate(&self) -> &DurablePublicationFate<SignalDurableMediumDenial> {
        &self.fate
    }
    pub fn committed_location(&self) -> Option<(SignalDurableJournal, u64, [u8; 32])> {
        self.location
    }
}
