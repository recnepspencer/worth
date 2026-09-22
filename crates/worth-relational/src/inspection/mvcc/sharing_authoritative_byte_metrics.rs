use super::observation::RelationalBranchSharingObservation;

/// Live authoritative byte totals, read from the selected roots, their storage
/// regions, and their canonical commit artifacts at observation time.
///
/// Each category is reported twice. The `logical_branch_*` total sums the
/// category once per selected branch, as if no storage were shared; the
/// matching `unique_physical_*` total sums each distinct allocation exactly
/// once, keyed by
/// [`RelationalAuthoritativeAllocationLocator`](super::allocation_inventory::RelationalAuthoritativeAllocationLocator).
/// The gap between the two pairs is the structural sharing.
///
/// Every total in this lane is governed by [`Self::byte_metric_scope`]. None
/// of them is a recorded counter, and none of them measures resident memory:
/// storage that the scope excludes is reported on its own `unique_*` metric
/// below rather than folded into the authoritative totals.
impl RelationalBranchSharingObservation {
    /// Partition payload bytes summed once per selected branch.
    ///
    /// Truth source: the storage regions of each selected branch's live root.
    pub const fn logical_branch_partition_payload_bytes(&self) -> u64 {
        self.logical_branch_partition_payload_bytes
    }

    /// Partition payload bytes summed once per distinct native allocation.
    ///
    /// Truth source: each region's storage owners, deduplicated by allocation
    /// locator. Reused nodes and values are counted once even across different regions.
    pub const fn unique_physical_partition_payload_bytes(&self) -> u64 {
        self.unique_physical_partition_payload_bytes
    }

    /// Root metadata and schema-authority bytes summed once per selected
    /// branch.
    ///
    /// Truth source: the root-owned authoritative allocations of each selected
    /// branch's live root.
    pub const fn logical_branch_root_metadata_bytes(&self) -> u64 {
        self.logical_branch_root_metadata_bytes
    }

    /// Root metadata bytes summed once per distinct root allocation.
    ///
    /// Truth source: the same root allocations, deduplicated by locator.
    ///
    /// Byte scope: root metadata only. Schema-authority storage contributes to
    /// the logical total above but is a separate allocation kind and is not
    /// included here; both are included in
    /// [`Self::unique_physical_authoritative_bytes`].
    pub const fn unique_physical_root_metadata_bytes(&self) -> u64 {
        self.unique_physical_root_metadata_bytes
    }

    /// Persistent reachability-structure bytes summed once per selected
    /// branch.
    ///
    /// Truth source: the region-map nodes, region-set objects, and the
    /// replacement and removal storage of each selected branch's live root.
    pub const fn logical_branch_root_reachability_bytes(&self) -> u64 {
        self.logical_branch_root_reachability_bytes
    }

    /// Persistent reachability-structure bytes summed once per distinct
    /// allocation.
    ///
    /// Truth source: the same reachability allocations, deduplicated by
    /// locator. Persistent structures shared across roots are counted once,
    /// which is how index reuse becomes observable.
    pub const fn unique_physical_root_reachability_bytes(&self) -> u64 {
        self.unique_physical_root_reachability_bytes
    }

    /// Canonical commit bytes summed once per selected branch.
    ///
    /// Truth source: the commit artifact, canonical payload, envelope, and
    /// nested envelope storage linked by each selected branch's live root.
    pub const fn logical_branch_canonical_commit_bytes(&self) -> u64 {
        self.logical_branch_canonical_commit_bytes
    }

    /// Canonical commit bytes summed once per distinct commit allocation.
    ///
    /// Truth source: the same commit allocations, deduplicated by locator.
    pub const fn unique_physical_canonical_commit_bytes(&self) -> u64 {
        self.unique_physical_canonical_commit_bytes
    }

    /// All authoritative bytes summed once per selected branch.
    ///
    /// Truth source: the complete owner walk over each selected branch's live
    /// root and commit artifact.
    ///
    /// Byte scope: this total also includes the root-region and
    /// partition-state object bytes, which have no category metric of their
    /// own. It is therefore greater than or equal to the sum of the four
    /// `logical_branch_*` category totals above.
    pub const fn logical_branch_authoritative_bytes(&self) -> u64 {
        self.logical_branch_authoritative_bytes
    }

    /// All authoritative bytes summed once per distinct allocation.
    ///
    /// Truth source: the complete owner walk, deduplicated by locator. This is
    /// the single total governed end to end by [`Self::byte_metric_scope`],
    /// and the one against which the excluded categories below are defined.
    pub const fn unique_physical_authoritative_bytes(&self) -> u64 {
        self.unique_physical_authoritative_bytes
    }

    /// Diagnostic bytes held by the observed regions and commit artifacts.
    ///
    /// Truth source: the same owner walk, deduplicated per region and per
    /// commit.
    ///
    /// Byte scope: excluded from every authoritative total above. Diagnostics
    /// are reported so that their cost stays visible, not so that they can be
    /// added back into the authoritative totals.
    pub const fn unique_diagnostic_bytes(&self) -> u64 {
        self.unique_diagnostic_bytes
    }

    /// Retention metadata bytes held by the observed regions.
    ///
    /// Truth source: the same owner walk, deduplicated per region.
    /// Immutable roots have no runtime pin counters, so their contribution is
    /// zero; live runtime pin storage is outside this selected-root inspection.
    ///
    /// Byte scope: excluded from every authoritative total above.
    pub const fn unique_retention_metadata_bytes(&self) -> u64 {
        self.unique_retention_metadata_bytes
    }

    /// Owner-reported allocator bookkeeping outside authoritative storage.
    ///
    /// Version 5 reports zero: shared allocations expose their owned payload
    /// layout, not allocator-private headers, slack, or process heap overhead.
    /// This is not evidence that those unmeasured costs are absent.
    pub const fn unique_allocator_bookkeeping_bytes(&self) -> u64 {
        self.unique_allocator_bookkeeping_bytes
    }

    /// Optional cache bytes held by the observed regions, plus the derived
    /// artifacts held for the observed commits.
    ///
    /// Truth source: the same owner walk, deduplicated per region and per
    /// commit.
    ///
    /// Byte scope: excluded from every authoritative total above. Caches are
    /// reconstructible and are never authoritative truth.
    pub const fn unique_optional_cache_bytes(&self) -> u64 {
        self.unique_optional_cache_bytes
    }
}
