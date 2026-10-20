use std::num::NonZeroUsize;

/// Host-owned controls for output resolution;
/// registry obligation custody, demand records, and required-key indexing have
/// separate aggregate retained-byte limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryOutputDemandResourceProfile {
    source_currentness_work: Option<NonZeroUsize>,

    settlement_attempts: NonZeroUsize,
    registry_obligation_retained_bytes: NonZeroUsize,
    registry_record_retained_bytes: NonZeroUsize,
    registry_required_retained_bytes: NonZeroUsize,
    lineage_retained_bytes: NonZeroUsize,
}

/// Limits carried through source selection, production, and recovery. Admission
/// intersects them with its own host profile; they do not grant runtime authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryOutputDemandLimits {
    source_currentness_work: Option<usize>,

    settlement_attempts: usize,
}

impl WorthQueryOutputDemandResourceProfile {
    pub const fn standard() -> Self {
        Self::new(NonZeroUsize::new(64).unwrap())
    }

    /// Install a convenience settlement-attempt policy.
    /// Ordinary source/currentness work has no intrinsic ceiling.
    pub const fn new(settlement_attempts: NonZeroUsize) -> Self {
        Self {
            source_currentness_work: None,

            settlement_attempts,
            registry_obligation_retained_bytes: NonZeroUsize::new(4 * 1_024 * 1_024).unwrap(),
            registry_record_retained_bytes: NonZeroUsize::new(64 * 1_024 * 1_024).unwrap(),
            registry_required_retained_bytes: NonZeroUsize::new(4 * 1_024 * 1_024).unwrap(),
            lineage_retained_bytes: NonZeroUsize::new(64 * 1_024 * 1_024).unwrap(),
        }
    }

    /// An explicit host choice of finite work limits.
    pub const fn bounded(
        source_currentness_work: NonZeroUsize,

        settlement_attempts: NonZeroUsize,
    ) -> Self {
        Self {
            source_currentness_work: Some(source_currentness_work),

            ..Self::new(settlement_attempts)
        }
    }

    pub const fn with_registry_obligation_retained_bytes(mut self, maximum: NonZeroUsize) -> Self {
        self.registry_obligation_retained_bytes = maximum;
        self
    }

    pub const fn registry_obligation_retained_bytes(self) -> usize {
        self.registry_obligation_retained_bytes.get()
    }

    pub const fn with_registry_record_retained_bytes(mut self, maximum: NonZeroUsize) -> Self {
        self.registry_record_retained_bytes = maximum;
        self
    }

    pub const fn registry_record_retained_bytes(self) -> usize {
        self.registry_record_retained_bytes.get()
    }

    pub const fn with_registry_required_retained_bytes(mut self, maximum: NonZeroUsize) -> Self {
        self.registry_required_retained_bytes = maximum;
        self
    }

    pub const fn registry_required_retained_bytes(self) -> usize {
        self.registry_required_retained_bytes.get()
    }

    pub const fn with_lineage_retained_bytes(mut self, maximum: NonZeroUsize) -> Self {
        self.lineage_retained_bytes = maximum;
        self
    }

    pub const fn lineage_retained_bytes(self) -> usize {
        self.lineage_retained_bytes.get()
    }

    pub fn limits(self) -> WorthQueryOutputDemandLimits {
        WorthQueryOutputDemandLimits {
            source_currentness_work: self.source_currentness_work.map(NonZeroUsize::get),

            settlement_attempts: self.settlement_attempts.get(),
        }
    }

    pub fn constrain(
        self,
        requested: WorthQueryOutputDemandLimits,
    ) -> WorthQueryOutputDemandLimits {
        self.limits().restricted(
            requested.source_currentness_work,
            requested.settlement_attempts,
        )
    }
}

impl Default for WorthQueryOutputDemandResourceProfile {
    fn default() -> Self {
        Self::standard()
    }
}

impl WorthQueryOutputDemandLimits {
    pub fn restricted(
        self,
        source_currentness_work: Option<usize>,

        settlement_attempts: usize,
    ) -> Self {
        Self {
            source_currentness_work: self
                .source_currentness_work
                .into_iter()
                .chain(source_currentness_work)
                .min(),

            settlement_attempts: self.settlement_attempts.min(settlement_attempts),
        }
    }

    pub const fn source_currentness_work(self) -> Option<usize> {
        self.source_currentness_work
    }
    pub const fn settlement_attempts(self) -> usize {
        self.settlement_attempts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_currentness_and_settlement_restrictions_cannot_widen_host_policy() {
        let ordinary = WorthQueryOutputDemandResourceProfile::standard();
        let narrow = ordinary.limits().restricted(Some(9), 3);
        assert_eq!(ordinary.constrain(narrow), narrow);
        let small = WorthQueryOutputDemandResourceProfile::bounded(
            NonZeroUsize::new(2).unwrap(),
            NonZeroUsize::new(1).unwrap(),
        );
        assert_eq!(small.constrain(ordinary.limits()), small.limits());
    }
}
