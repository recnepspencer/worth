/// Closed Runtime World dependency vocabulary for one exact component-basis
/// pin. A count is semantic usage, not an owner lease or a history record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ComponentClaimKind {
    ProductBranchHead,
    RetainedCompositeHistory,
    AdmittedObservation,
    ActivePublicationAttempt,
    ProductUnpublishedOwnerEffects,
}

impl ComponentClaimKind {
    const fn index(self) -> usize {
        match self {
            Self::ProductBranchHead => 0,
            Self::RetainedCompositeHistory => 1,
            Self::AdmittedObservation => 2,
            Self::ActivePublicationAttempt => 3,
            Self::ProductUnpublishedOwnerEffects => 4,
        }
    }
}

/// Claim counts owned by one live exact pin.
///
/// Every mutation is checked. A count cannot wrap into an earlier state, and
/// a release cannot manufacture a count for a dependency it did not own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ComponentClaimCounts {
    counts: [usize; 5],
}

impl ComponentClaimCounts {
    pub(crate) const fn zero() -> Self {
        Self { counts: [0; 5] }
    }

    pub const fn get(self, class: ComponentClaimKind) -> usize {
        self.counts[class.index()]
    }

    pub(crate) const fn is_zero(self) -> bool {
        self.total() == 0
    }

    pub const fn total(self) -> usize {
        let mut total: usize = 0;
        let mut index = 0;
        while index < self.counts.len() {
            total = total.saturating_add(self.counts[index]);
            index += 1;
        }
        total
    }

    pub(crate) fn increment(&mut self, class: ComponentClaimKind) -> Option<usize> {
        let count = self.counts[class.index()];
        let next = count.checked_add(1)?;
        self.counts[class.index()] = next;
        Some(next)
    }

    pub(crate) fn decrement(&mut self, class: ComponentClaimKind) -> Option<usize> {
        let count = self.counts[class.index()];
        let next = count.checked_sub(1)?;
        self.counts[class.index()] = next;
        Some(next)
    }
}
