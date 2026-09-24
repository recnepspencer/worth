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
    #[cfg(any(test, feature = "certification-test-authority"))]
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

    /// Widens the byte ceiling by the declared retained WAL tail.
    ///
    /// Every sealed WAL group is charged as retained growth until its segment is
    /// reclaimed, so the artifact allowance alone would refuse a group the
    /// declared WAL policy lawfully retains.
    pub(in crate::physical_runtime) const fn covering_retained_wal_tail(
        self,
        retained_wal_tail_bytes: u64,
    ) -> Self {
        Self {
            candidate_bytes: self.candidate_bytes.saturating_add(retained_wal_tail_bytes),
            ..self
        }
    }

    pub(in crate::physical_runtime) const fn growth_bytes(self) -> u64 {
        self.candidate_bytes - self.progress_headroom_bytes
    }

    pub(in crate::physical_runtime) const fn growth_entries(self) -> u32 {
        self.obligation_entries - self.progress_headroom_entries
    }
}

#[cfg(test)]
mod tests {
    use super::PhysicalRetentionProfile;

    const MIB: u64 = 1024 * 1024;

    #[test]
    fn store_default_withholds_headroom_inside_its_hard_ceilings() {
        let profile = PhysicalRetentionProfile::store_default();
        assert_eq!(profile.growth_bytes(), 8 * MIB - 64 * 1024);
        assert_eq!(profile.growth_entries(), 4_096 - 8);
    }

    #[test]
    fn retained_wal_tail_widens_only_the_byte_ceiling() {
        let tail = 3 * MIB;
        let profile = PhysicalRetentionProfile::store_default().covering_retained_wal_tail(tail);
        assert_eq!(profile.growth_bytes(), 8 * MIB + tail - 64 * 1024);
        assert_eq!(profile.growth_entries(), 4_096 - 8);
        let saturated =
            PhysicalRetentionProfile::store_default().covering_retained_wal_tail(u64::MAX);
        assert_eq!(saturated.growth_bytes(), u64::MAX - 64 * 1024);
    }
}
