/// Closed Runtime World dependency vocabulary for one exact component-basis
/// pin. A count is semantic usage, not an owner lease or a history record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ComponentBasisDependencyClass {
    ProductBranchHead,
    RetainedCompositeHistory,
    AdmittedObservation,
    ActivePublicationAttempt,
    ProductUnpublishedOwnerEffects,
    HistoricalInspection,
}

impl ComponentBasisDependencyClass {
    #[cfg(test)]
    pub(crate) const ALL: [Self; 6] = [
        Self::ProductBranchHead,
        Self::RetainedCompositeHistory,
        Self::AdmittedObservation,
        Self::ActivePublicationAttempt,
        Self::ProductUnpublishedOwnerEffects,
        Self::HistoricalInspection,
    ];

    const fn index(self) -> usize {
        match self {
            Self::ProductBranchHead => 0,
            Self::RetainedCompositeHistory => 1,
            Self::AdmittedObservation => 2,
            Self::ActivePublicationAttempt => 3,
            Self::ProductUnpublishedOwnerEffects => 4,
            Self::HistoricalInspection => 5,
        }
    }
}

/// Six independent exact dependency counters owned by one registry entry.
///
/// Every mutation is checked. A count cannot wrap into an earlier state, and
/// a release cannot manufacture a count for a dependency it did not own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentBasisDependencyCounts {
    counts: [usize; 6],
}

impl ComponentBasisDependencyCounts {
    pub(crate) const fn zero() -> Self {
        Self { counts: [0; 6] }
    }

    pub const fn get(self, class: ComponentBasisDependencyClass) -> usize {
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

    pub(crate) fn increment(&mut self, class: ComponentBasisDependencyClass) -> Option<usize> {
        let count = self.counts[class.index()];
        let next = count.checked_add(1)?;
        self.counts[class.index()] = next;
        Some(next)
    }

    pub(crate) fn decrement(&mut self, class: ComponentBasisDependencyClass) -> Option<usize> {
        let count = self.counts[class.index()];
        let next = count.checked_sub(1)?;
        self.counts[class.index()] = next;
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::{ComponentBasisDependencyClass, ComponentBasisDependencyCounts};

    #[test]
    fn every_dependency_class_has_an_independent_checked_slot() {
        let mut counts = ComponentBasisDependencyCounts::zero();
        for class in ComponentBasisDependencyClass::ALL {
            assert_eq!(counts.increment(class), Some(1));
        }
        assert_eq!(counts.total(), ComponentBasisDependencyClass::ALL.len());
        for class in ComponentBasisDependencyClass::ALL {
            assert_eq!(counts.get(class), 1);
            assert_eq!(counts.decrement(class), Some(0));
        }
        assert!(counts.is_zero());
        assert_eq!(
            counts.decrement(ComponentBasisDependencyClass::ProductBranchHead),
            None
        );
    }

    #[test]
    fn count_overflow_is_a_denial_without_mutating_the_slot() {
        let mut counts = ComponentBasisDependencyCounts {
            counts: [usize::MAX; 6],
        };
        assert_eq!(
            counts.increment(ComponentBasisDependencyClass::HistoricalInspection),
            None
        );
        assert_eq!(
            counts.get(ComponentBasisDependencyClass::HistoricalInspection),
            usize::MAX
        );
    }
}
