#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhysicalRecoveryRootProtocolCounters {
    successor_root_integrity_admissions: u64,
    successor_root_interpretations: u64,
    staged_selector_integrity_admissions: u64,
    closeout_selector_interpretations: u64,
}

impl PhysicalRecoveryRootProtocolCounters {
    pub const fn successor_root_integrity_admissions(self) -> u64 {
        self.successor_root_integrity_admissions
    }

    pub const fn successor_root_interpretations(self) -> u64 {
        self.successor_root_interpretations
    }

    pub const fn staged_selector_integrity_admissions(self) -> u64 {
        self.staged_selector_integrity_admissions
    }

    pub const fn closeout_selector_interpretations(self) -> u64 {
        self.closeout_selector_interpretations
    }

    pub(crate) fn record_successor_root_integrity_admission(&mut self) {
        self.successor_root_integrity_admissions =
            self.successor_root_integrity_admissions.saturating_add(1);
    }

    pub(crate) fn record_successor_root_interpretation(&mut self) {
        self.successor_root_interpretations = self.successor_root_interpretations.saturating_add(1);
    }

    pub(crate) const fn with_successor_root_route(mut self, route: Self) -> Self {
        self.successor_root_integrity_admissions = route.successor_root_integrity_admissions;
        self.successor_root_interpretations = route.successor_root_interpretations;
        self
    }

    pub(crate) const fn with_staged_selector_closeout(mut self) -> Self {
        self.staged_selector_integrity_admissions = 1;
        self.closeout_selector_interpretations = 1;
        self
    }

    pub(crate) const fn with_staged_selector_route(mut self, route: Self) -> Self {
        self.staged_selector_integrity_admissions = route.staged_selector_integrity_admissions;
        self.closeout_selector_interpretations = route.closeout_selector_interpretations;
        self
    }
}
