use std::num::NonZeroUsize;

/// Host-owned allowances for output resolution. Producer limits are per demand;
/// registry obligation custody, demand records, and required-key indexing have
/// separate aggregate retained-byte limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryOutputDemandResourceProfile {
    source_currentness_work: NonZeroUsize,
    producer_work: NonZeroUsize,
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
    source_currentness_work: usize,
    producer_work: usize,
    producer_retained_bytes: usize,
    settlement_attempts: usize,
}

impl WorthQueryOutputDemandResourceProfile {
    pub const fn standard() -> Self {
        let work = NonZeroUsize::new(4_194_304).unwrap();
        let retained_bytes = NonZeroUsize::new(4 * 1_024 * 1_024).unwrap();
        Self::bounded(work, work, retained_bytes, NonZeroUsize::new(64).unwrap())
    }

    pub const fn bounded(
        source_currentness_work: NonZeroUsize,
        producer_work: NonZeroUsize,
        producer_retained_bytes: NonZeroUsize,
        settlement_attempts: NonZeroUsize,
    ) -> Self {
        Self {
            source_currentness_work,
            producer_work,
            producer_retained_bytes,
            settlement_attempts,
            registry_obligation_retained_bytes: NonZeroUsize::new(4 * 1_024 * 1_024).unwrap(),
            registry_record_retained_bytes: NonZeroUsize::new(64 * 1_024 * 1_024).unwrap(),
            registry_required_retained_bytes: NonZeroUsize::new(4 * 1_024 * 1_024).unwrap(),
            lineage_retained_bytes: NonZeroUsize::new(64 * 1_024 * 1_024).unwrap(),
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

    pub const fn limits(self) -> WorthQueryOutputDemandLimits {
        WorthQueryOutputDemandLimits {
            source_currentness_work: self.source_currentness_work.get(),
            producer_work: self.producer_work.get(),
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
        source_currentness_work: usize,
        producer_work: usize,
        producer_retained_bytes: usize,
        settlement_attempts: usize,
    ) -> Self {
        Self {
            source_currentness_work: self.source_currentness_work.min(source_currentness_work),
            producer_work: self.producer_work.min(producer_work),
            producer_retained_bytes: self.producer_retained_bytes.min(producer_retained_bytes),
            settlement_attempts: self.settlement_attempts.min(settlement_attempts),
        }
    }

    /// Artifact ceilings bound production, not the work of proving source currentness.
    pub fn for_artifact(self, producer_work: usize, producer_retained_bytes: usize) -> Self {
        self.restricted(
            self.source_currentness_work,
            producer_work,
            producer_retained_bytes,
            self.settlement_attempts,
        )
    }

    pub const fn source_currentness_work(self) -> usize {
        self.source_currentness_work
    }
    pub const fn producer_work(self) -> usize {
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
    fn standard_limits_are_finite_and_child_limits_do_not_shrink_source_proof() {
        let host = WorthQueryOutputDemandResourceProfile::default();
        let child = host.limits().for_artifact(17, 31);
        assert_eq!(child.source_currentness_work(), 4_194_304);
        assert_eq!(child.producer_work(), 17);
        assert_eq!(child.producer_retained_bytes(), 31);
        assert_eq!(child.settlement_attempts(), 64);
    }

    #[test]
    fn explicit_restrictions_and_foreign_host_limits_cannot_widen_capacity() {
        let standard = WorthQueryOutputDemandResourceProfile::default();
        let narrow = standard.limits().restricted(9, 7, 5, 3);
        assert_eq!(standard.constrain(narrow), narrow);
        let small = WorthQueryOutputDemandResourceProfile::bounded(
            NonZeroUsize::new(2).unwrap(),
            NonZeroUsize::new(3).unwrap(),
            NonZeroUsize::new(4).unwrap(),
            NonZeroUsize::new(1).unwrap(),
        );
        assert_eq!(small.constrain(standard.limits()), small.limits());
        assert_eq!(narrow.for_artifact(100, 100), narrow);
    }

    #[test]
    fn registry_obligation_limit_is_independent_of_artifact_allowances() {
        let host = WorthQueryOutputDemandResourceProfile::standard();
        let registry_limited =
            host.with_registry_obligation_retained_bytes(NonZeroUsize::new(128).unwrap());
        assert_eq!(registry_limited.registry_obligation_retained_bytes(), 128);
        assert_eq!(registry_limited.limits(), host.limits());
    }
}
