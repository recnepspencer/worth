/// Every retained record is installed into exactly one recovery slot and
/// occupies it until the record is cleaned up.
const RECOVERY_SLOT: usize = 1;

/// A retained record's live obligations, counted once from its own custody
/// when the record is installed and divided by scope. The component half is
/// the exact pin pair the record holds or reserved; the composite half is the
/// recovery slot it occupies plus its reserved history capacity or installed
/// successor history protection. The halves are one value
/// because they are only meaningful together: nothing outside this type may
/// name a pair that does not sum to the record's own count, and no route may
/// restate the count as a literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProductUnpublishedLiveObligations {
    component: usize,
    composite: usize,
}

impl ProductUnpublishedLiveObligations {
    pub(crate) fn with_observation(
        mut self,
        _observation: &crate::branch::ProductBranchObservation,
    ) -> Self {
        self.component += 2;
        self.composite += 1;
        self
    }

    pub(crate) fn from_custody(component_pins: usize, history_custody_held: bool) -> Self {
        Self {
            component: component_pins,
            composite: RECOVERY_SLOT + usize::from(history_custody_held),
        }
    }

    pub(crate) const fn component(self) -> usize {
        self.component
    }

    pub(crate) const fn composite(self) -> usize {
        self.composite
    }

    pub(crate) const fn total(self) -> usize {
        self.component + self.composite
    }
}
