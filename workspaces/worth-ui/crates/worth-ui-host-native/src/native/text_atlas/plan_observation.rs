//! Bounded native-owner evidence for one admitted atlas plan.

use super::transaction_plan_snapshot::UiNativeTextAtlasTransactionPlanSnapshot;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiNativeTextAtlasPlanObservation {
    host_session: u64,
    attempt: u64,
    surface: u64,
    binding: u64,
    work: [u64; 10],
}

impl UiNativeTextAtlasPlanObservation {
    #[cfg(test)]
    pub(crate) const fn empty() -> Self {
        Self {
            host_session: 0,
            attempt: 0,
            surface: 0,
            binding: 0,
            work: [0; 10],
        }
    }

    pub(crate) fn from_admitted_plan(
        basis: crate::native::physical_work_signal::UiNativePhysicalPresentationBasis,
        plan: UiNativeTextAtlasTransactionPlanSnapshot,
    ) -> Self {
        Self {
            host_session: basis.host_session_identity(),
            attempt: basis.attempt().diagnostic_value(),
            surface: basis.surface().diagnostic_value(),
            binding: basis.binding().diagnostic_value(),
            work: [
                u64::from(plan.key_lookups()),
                u64::from(plan.hits()),
                u64::from(plan.misses()),
                u64::from(plan.page_probes()),
                u64::from(plan.placement_probes()),
                u64::from(plan.eviction_candidates()),
                u64::from(plan.evictions()),
                plan.staged_bytes(),
                plan.physical_staged_bytes(),
                u64::from(plan.peak_entries()),
            ],
        }
    }

    pub const fn host_session(self) -> u64 {
        self.host_session
    }
    pub const fn attempt(self) -> u64 {
        self.attempt
    }
    pub const fn surface(self) -> u64 {
        self.surface
    }
    pub const fn binding(self) -> u64 {
        self.binding
    }
    pub const fn key_lookups(self) -> u64 {
        self.work[0]
    }
    pub const fn hits(self) -> u64 {
        self.work[1]
    }
    pub const fn misses(self) -> u64 {
        self.work[2]
    }
    pub const fn page_probes(self) -> u64 {
        self.work[3]
    }
    pub const fn placement_probes(self) -> u64 {
        self.work[4]
    }
    pub const fn eviction_candidates(self) -> u64 {
        self.work[5]
    }
    pub const fn evictions(self) -> u64 {
        self.work[6]
    }
    pub const fn staged_bytes(self) -> u64 {
        self.work[7]
    }
    pub const fn physical_staged_bytes(self) -> u64 {
        self.work[8]
    }
    pub const fn peak_entries(self) -> u64 {
        self.work[9]
    }
}
