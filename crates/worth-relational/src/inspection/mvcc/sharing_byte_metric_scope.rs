use super::observation::RelationalBranchSharingObservation;

/// Version of the sharing-observation contract: which metrics exist, which
/// truth source each one is read from, and which byte scope governs the live
/// authoritative totals.
///
/// Any change to what an existing metric means requires a new version rather
/// than a silent redefinition. Version 5 retains the complete-owner scope and
/// visibility commitments, but identifies payloads at shared node/value
/// granularity rather than treating a whole partition as one allocation.
pub const RELATIONAL_SHARING_INSPECTION_VERSION: u16 = 5;

/// Explicit scope of the live authoritative byte totals in a sharing
/// observation.
///
/// This scope governs only the `logical_branch_*` and `unique_physical_*`
/// totals and their excluded-lane companions. It does not govern
/// `branch_metadata_bytes`, which is a selection-lane value, and it does not
/// govern any recorded cost counter, which reports work performed rather than
/// storage currently reachable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalSharingByteMetricScope {
    /// Every owner-defined authoritative allocation reachable from the selected
    /// branches: partition payloads, partition-state and root-region objects,
    /// root metadata and schema authority, persistent reachability structures
    /// and their replacement/removal storage, and the canonical commit
    /// artifact, payload, and envelope storage.
    ///
    /// Branch reference metadata, diagnostics, retention metadata, allocator
    /// bookkeeping, and optional caches are excluded from these totals and are
    /// reported on their own separate metrics.
    CompleteAuthoritativeOwnerAllocations,
    /// Historical scope under which the byte totals covered authoritative
    /// partition payloads only.
    ///
    /// Inspection versions 4 and later never produce this variant; they report
    /// `CompleteAuthoritativeOwnerAllocations`. The variant remains public so
    /// that callers matching on the scope keep compiling across the change.
    #[deprecated(
        note = "inspection versions 4 and later report complete owner allocations; this scope is never produced"
    )]
    AuthoritativePartitionPayloadsOnly,
}

impl RelationalBranchSharingObservation {
    /// Version of the sharing-observation contract this artifact was built
    /// against.
    ///
    /// Truth source: the `RELATIONAL_SHARING_INSPECTION_VERSION` constant,
    /// stamped at assembly time. It is independent of the selection, of live
    /// owner state, and of every recorded counter.
    pub const fn inspection_version(&self) -> u16 {
        self.inspection_version
    }

    /// Scope governing this artifact's live authoritative byte totals.
    ///
    /// Truth source: a compile-time constant of the assembling inspection
    /// module, not a runtime policy value and not a property of the selected
    /// branches. Two observations of different selections on the same build
    /// always report the same scope.
    ///
    /// Byte scope: see `RelationalSharingByteMetricScope`. The scope applies to
    /// the live authoritative totals only; selection-lane and recorded-lane
    /// byte metrics are outside it.
    pub const fn byte_metric_scope(&self) -> RelationalSharingByteMetricScope {
        self.byte_metric_scope
    }
}
