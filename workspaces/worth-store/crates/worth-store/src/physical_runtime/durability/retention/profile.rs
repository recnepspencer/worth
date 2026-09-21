/// Hard retained-storage ceilings with progress headroom withheld inside them.
///
/// Growth may use only the allowance that remains after headroom and current
/// charges. The headroom itself is reserved for one checkpoint or cleanup cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct PhysicalRetentionProfile {
    candidate_bytes: u64,
    obligation_entries: u32,
    progress_headroom_bytes: u64,
    progress_headroom_entries: u32,
}

impl PhysicalRetentionProfile {
    pub(in crate::physical_runtime) const fn new(
        candidate_bytes: u64,
        obligation_entries: u32,
        progress_headroom_bytes: u64,
        progress_headroom_entries: u32,
    ) -> Option<Self> {
        if candidate_bytes == 0
            || obligation_entries == 0
            || progress_headroom_bytes == 0
            || progress_headroom_entries == 0
            || progress_headroom_bytes >= candidate_bytes
            || progress_headroom_entries >= obligation_entries
        {
            return None;
        }
        Some(Self {
            candidate_bytes,
            obligation_entries,
            progress_headroom_bytes,
            progress_headroom_entries,
        })
    }

    pub(in crate::physical_runtime) const fn store_default() -> Self {
        Self {
            candidate_bytes: 8 * 1024 * 1024,
            obligation_entries: 4_096,
            progress_headroom_bytes: 64 * 1024,
            progress_headroom_entries: 8,
        }
    }

    pub(in crate::physical_runtime) const fn growth_bytes(self) -> u64 {
        self.candidate_bytes - self.progress_headroom_bytes
    }

    pub(in crate::physical_runtime) const fn growth_entries(self) -> u32 {
        self.obligation_entries - self.progress_headroom_entries
    }
}