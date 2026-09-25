use super::observation::RelationalBranchSharingObservation;

/// Recorded-cost metrics: counters written by earlier fork and publication
/// work on the selected branches, summed across the selection.
///
/// Nothing in this lane is recomputed from live owner state. Each counter is
/// incremented once, when the work it describes runs, on the branch cell that
/// did the work. Consequently:
///
/// - a forked branch starts every counter at zero and does not inherit its
///   source branch's history;
/// - a branch that only reads never accumulates any of these counters,
///   however much storage it shares;
/// - these counters are monotonic per branch and are not reduced when the
///   storage they describe is later superseded or dropped.
///
/// [`Self::byte_metric_scope`] does not govern this lane: the byte counters
/// here describe work that was done, not storage that exists now.
impl RelationalBranchSharingObservation {
    /// Authoritative truth bytes copied by publications on the selected
    /// branches.
    /// Shared payload adoption contributes zero. Path detachment records the
    /// copied node payload and page entries (including transient copies), not
    /// the size of the whole region or reference-count/allocation headers.
    ///
    /// Truth source: the publication cost recorded at each successful root
    /// publication, plus the materialization cost recorded at each fork
    /// install. Not a live reading of any root.
    pub const fn copied_truth_bytes(&self) -> u64 {
        self.copied_truth_bytes
    }

    /// Commit envelopes copied by publications and fork installs on the
    /// selected branches.
    ///
    /// Truth source: the same recorded publication and fork costs. A fork that
    /// shares its source root copies no envelope and leaves this at zero.
    pub const fn copied_commit_envelopes(&self) -> u64 {
        self.copied_commit_envelopes
    }

    /// Entities materialized by fork installs on the selected branches.
    ///
    /// Truth source: the fork materialization cost recorded when each selected
    /// branch was installed. Zero for branches that were never forked and for
    /// forks that shared their source root outright.
    pub const fn fork_materialized_entity_count(&self) -> u64 {
        self.fork_materialized_entity_count
    }

    /// Relations materialized by fork installs on the selected branches.
    ///
    /// Truth source: the same recorded fork materialization cost.
    pub const fn fork_materialized_relation_count(&self) -> u64 {
        self.fork_materialized_relation_count
    }

    /// Authoritative bytes materialized by fork installs on the selected
    /// branches.
    ///
    /// Truth source: the same recorded fork materialization cost. This is what
    /// forking cost when it ran; it is not the storage the fork holds now, and
    /// it may exceed or fall short of every live total on this observation.
    pub const fn fork_materialized_authoritative_bytes(&self) -> u64 {
        self.fork_materialized_authoritative_bytes
    }

    /// Number of fork installs on the selected branches that adopted an
    /// existing root instead of materializing a new one.
    ///
    /// Truth source: the fork install path, which records exactly one
    /// acquisition per shared-root install on the new branch cell. The main
    /// branch, never having been forked, contributes zero.
    pub const fn shared_root_acquisitions(&self) -> u64 {
        self.shared_root_acquisitions
    }

    /// Storage regions touched by publications on the selected branches.
    ///
    /// Truth source: the publication cost recorded at each successful root
    /// publication. It counts publication work over time and is unrelated to
    /// [`Self::inspection_reconstructed_region_count`], which counts the
    /// current inspection walk.
    pub const fn publication_touched_region_count(&self) -> u64 {
        self.publication_touched_region_count
    }

    /// Storage regions that publications on the selected branches reused
    /// rather than rebuilt.
    ///
    /// Truth source: the same recorded publication cost. This is the recorded
    /// counterpart of structural sharing; the live counterpart is the gap
    /// between the `logical_branch_*` and `unique_physical_*` totals.
    pub const fn publication_reused_region_count(&self) -> u64 {
        self.publication_reused_region_count
    }

    /// Authoritative bytes newly allocated by publications on the selected
    /// branches.
    ///
    /// Truth source: the new-allocation figure recorded at each successful
    /// root publication. Superseded storage is never subtracted from it.
    pub const fn publication_new_authoritative_bytes(&self) -> u64 {
        self.publication_new_authoritative_bytes
    }

    /// Authoritative bytes recorded as reclaimable on the selected branches.
    ///
    /// Truth source: the branch sharing cost counters. No publication, fork,
    /// or retention path currently records reclamation, so this metric is
    /// always zero at the current milestone. It is reported as a declared,
    /// unpopulated lane rather than silently omitted; a zero here is the
    /// absence of recorded evidence, not a live finding that nothing is
    /// reclaimable.
    pub const fn reclaimable_unique_bytes(&self) -> u64 {
        self.reclaimable_unique_bytes
    }
}
