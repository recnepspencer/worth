#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResolvedRepairRegion {
    integrity: super::integrity_classification::IntegrityRepairRegion,
    pub(super) source: std::path::PathBuf,
}

impl ResolvedRepairRegion {
    pub(super) fn new(
        integrity: super::integrity_classification::IntegrityRepairRegion,
        source: std::path::PathBuf,
    ) -> Self {
        Self { integrity, source }
    }
    pub(super) const fn integrity(&self) -> super::integrity_classification::IntegrityRepairRegion {
        self.integrity
    }
}

impl PartialOrd for ResolvedRepairRegion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ResolvedRepairRegion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.integrity
            .cmp(&other.integrity)
            .then_with(|| self.source.cmp(&other.source))
    }
}
