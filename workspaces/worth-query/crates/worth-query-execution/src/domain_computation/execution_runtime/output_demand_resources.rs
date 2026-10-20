use std::num::NonZeroUsize;

/// Host-owned allowances for output resolution. Producer limits are per demand;
/// registry obligation custody, demand records, and required-key indexing have
/// separate aggregate retained-byte limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryOutputDemandResourceProfile {
    source_currentness_work: Option<NonZeroUsize>,
    producer_work: Option<NonZeroUsize>,
    producer_retained_bytes: NonZeroUsize,
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
    producer_work: Option<usize>,
    producer_retained_bytes: usize,
    settlement_attempts: usize,
}

impl WorthQueryOutputDemandResourceProfile {
    pub const fn standard() -> Self {
        let retained_bytes = NonZeroUsize::new(4 * 1_024 * 1_024).unwrap();
        Self::new(retained_bytes, NonZeroUsize::new(64).unwrap())
    }

    /// Install physical retention and a convenience settlement-attempt policy.
    /// Ordinary source/currentness and producer work has no intrinsic ceiling.
    pub const fn new(
        producer_retained_bytes: NonZeroUsize,
        settlement_attempts: NonZeroUsize,
    ) -> Self {
        Self {
            source_currentness_work: None,
            producer_work: None,
            producer_retained_bytes,
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
        producer_work: NonZeroUsize,
        producer_retained_bytes: NonZeroUsize,
        settlement_attempts: NonZeroUsize,
    ) -> Self {
        Self {
            source_currentness_work: Some(source_currentness_work),
            producer_work: Some(producer_work),
            ..Self::new(producer_retained_bytes, settlement_attempts)
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
            producer_work: self.producer_work.map(NonZeroUsize::get),
            producer_retained_bytes: self.producer_retained_bytes.get(),
            settlement_attempts: self.settlement_attempts.get(),
        }
    }

    pub fn constrain(
        self,
        requested: WorthQueryOutputDemandLimits,
    ) -> WorthQueryOutputDemandLimits {
        self.limits().restricted(
            requested.source_currentness_work,
            requested.producer_work,
            requested.producer_retained_bytes,
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
        producer_work: Option<usize>,
        producer_retained_bytes: usize,
        settlement_attempts: usize,
    ) -> Self {
        Self {
            source_currentness_work: self
                .source_currentness_work
                .into_iter()
                .chain(source_currentness_work)
                .min(),
            producer_work: self.producer_work.into_iter().chain(producer_work).min(),
            producer_retained_bytes: self.producer_retained_bytes.min(producer_retained_bytes),
            settlement_attempts: self.settlement_attempts.min(settlement_attempts),
        }
    }

    /// Artifact ceilings bound production, not the work of proving source currentness.
    pub fn for_artifact(
        self,
        producer_work: Option<usize>,
        producer_retained_bytes: Option<usize>,
    ) -> Self {
        Self {
            producer_work: self.producer_work.into_iter().chain(producer_work).min(),
            producer_retained_bytes: match producer_retained_bytes {
                Some(maximum) => self.producer_retained_bytes.min(maximum),
                None => self.producer_retained_bytes,
            },
            ..self
        }
    }

    pub const fn source_currentness_work(self) -> Option<usize> {
        self.source_currentness_work
    }
    pub const fn producer_work(self) -> Option<usize> {
        self.producer_work
    }
    pub const fn producer_retained_bytes(self) -> usize {
        self.producer_retained_bytes
    }
    pub const fn settlement_attempts(self) -> usize {
        self.settlement_attempts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_work_is_absent_and_child_limits_do_not_shrink_source_proof() {
        let host = WorthQueryOutputDemandResourceProfile::default();
        let child = host.limits().for_artifact(Some(17), Some(31));
        assert_eq!(child.source_currentness_work(), None);
        assert_eq!(child.producer_work(), Some(17));
        assert_eq!(child.producer_retained_bytes(), 31);
        assert_eq!(child.settlement_attempts(), 64);
    }

    #[test]
    fn explicit_restrictions_and_foreign_host_limits_cannot_widen_capacity() {
        let standard = WorthQueryOutputDemandResourceProfile::default();
        let narrow = standard.limits().restricted(Some(9), Some(7), 5, 3);
        assert_eq!(standard.constrain(narrow), narrow);
        let small = WorthQueryOutputDemandResourceProfile::bounded(
            NonZeroUsize::new(2).unwrap(),
            NonZeroUsize::new(3).unwrap(),
            NonZeroUsize::new(4).unwrap(),
            NonZeroUsize::new(1).unwrap(),
        );
        assert_eq!(small.constrain(standard.limits()), small.limits());
        assert_eq!(narrow.for_artifact(Some(100), Some(100)), narrow);
    }

    #[test]
    fn registry_obligation_limit_is_independent_of_artifact_allowances() {
        let host = WorthQueryOutputDemandResourceProfile::standard();
        let registry_limited =
            host.with_registry_obligation_retained_bytes(NonZeroUsize::new(128).unwrap());
        assert_eq!(registry_limited.registry_obligation_retained_bytes(), 128);
        assert_eq!(registry_limited.limits(), host.limits());
    }

    #[test]
    fn absent_artifact_retention_preserves_the_independent_host_and_zero_narrows() {
        let host = WorthQueryOutputDemandResourceProfile::new(
            NonZeroUsize::new(23).unwrap(),
            NonZeroUsize::new(4).unwrap(),
        )
        .limits();
        assert_eq!(host.for_artifact(None, None), host);
        assert_eq!(host.for_artifact(None, Some(100)), host);
        assert_eq!(
            host.for_artifact(None, Some(7)).producer_retained_bytes(),
            7
        );
        assert_eq!(
            host.for_artifact(None, Some(0)).producer_retained_bytes(),
            0
        );
    }
}
