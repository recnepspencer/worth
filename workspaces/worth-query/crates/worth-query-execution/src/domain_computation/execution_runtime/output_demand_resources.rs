use std::num::NonZeroUsize;

/// Host-owned controls for output resolution;
/// registry obligation custody, demand records, and required-key indexing have
/// separate aggregate retained-byte limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryOutputDemandResourceProfile {
    source_currentness_work: Option<NonZeroUsize>,

    settlement_attempts: Option<NonZeroUsize>,
    registry_obligation_retained_bytes: Option<NonZeroUsize>,
    registry_record_retained_bytes: Option<NonZeroUsize>,
    registry_required_retained_bytes: Option<NonZeroUsize>,
    lineage_retained_bytes: Option<NonZeroUsize>,
}

/// Limits carried through source selection, production, and recovery. Admission
/// intersects them with its own host profile; they do not grant runtime authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryOutputDemandLimits {
    source_currentness_work: Option<usize>,

    settlement_attempts: Option<usize>,
}

impl WorthQueryOutputDemandResourceProfile {
    pub const fn standard() -> Self {
        Self {
            source_currentness_work: None,
            settlement_attempts: None,
            registry_obligation_retained_bytes: None,
            registry_record_retained_bytes: None,
            registry_required_retained_bytes: None,
            lineage_retained_bytes: None,
        }
    }

    /// Install an explicit convenience settlement-attempt policy.
    pub const fn new(settlement_attempts: NonZeroUsize) -> Self {
        Self {
            settlement_attempts: Some(settlement_attempts),
            ..Self::standard()
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
        self.registry_obligation_retained_bytes = Some(maximum);
        self
    }

    pub fn registry_obligation_retained_bytes(self) -> Option<usize> {
        self.registry_obligation_retained_bytes
            .map(NonZeroUsize::get)
    }

    pub const fn with_registry_record_retained_bytes(mut self, maximum: NonZeroUsize) -> Self {
        self.registry_record_retained_bytes = Some(maximum);
        self
    }

    pub fn registry_record_retained_bytes(self) -> Option<usize> {
        self.registry_record_retained_bytes.map(NonZeroUsize::get)
    }

    pub const fn with_registry_required_retained_bytes(mut self, maximum: NonZeroUsize) -> Self {
        self.registry_required_retained_bytes = Some(maximum);
        self
    }

    pub fn registry_required_retained_bytes(self) -> Option<usize> {
        self.registry_required_retained_bytes.map(NonZeroUsize::get)
    }

    pub const fn with_lineage_retained_bytes(mut self, maximum: NonZeroUsize) -> Self {
        self.lineage_retained_bytes = Some(maximum);
        self
    }

    pub fn lineage_retained_bytes(self) -> Option<usize> {
        self.lineage_retained_bytes.map(NonZeroUsize::get)
    }

    pub fn limits(self) -> WorthQueryOutputDemandLimits {
        WorthQueryOutputDemandLimits {
            source_currentness_work: self.source_currentness_work.map(NonZeroUsize::get),

            settlement_attempts: self.settlement_attempts.map(NonZeroUsize::get),
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

        settlement_attempts: Option<usize>,
    ) -> Self {
        Self {
            source_currentness_work: self
                .source_currentness_work
                .into_iter()
                .chain(source_currentness_work)
                .min(),

            settlement_attempts: self
                .settlement_attempts
                .into_iter()
                .chain(settlement_attempts)
                .min(),
        }
    }

    pub const fn source_currentness_work(self) -> Option<usize> {
        self.source_currentness_work
    }
    pub const fn settlement_attempts(self) -> Option<usize> {
        self.settlement_attempts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_currentness_and_settlement_restrictions_cannot_widen_host_policy() {
        let ordinary = WorthQueryOutputDemandResourceProfile::standard();
        let narrow = ordinary.limits().restricted(Some(9), Some(3));
        assert_eq!(ordinary.constrain(narrow), narrow);
        let small = WorthQueryOutputDemandResourceProfile::bounded(
            NonZeroUsize::new(2).unwrap(),
            NonZeroUsize::new(1).unwrap(),
        );
        assert_eq!(small.constrain(ordinary.limits()), small.limits());
    }
}
