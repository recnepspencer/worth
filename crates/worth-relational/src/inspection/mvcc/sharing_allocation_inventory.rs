use crate::identity::data::PartitionId;

/// Which owner-defined authoritative allocation an inventory entry describes.
///
/// These kinds partition the authoritative storage owned by a Relational root
/// and its canonical commit. They are the deduplication axis for every
/// `unique_physical_*` byte total: two selected branches that share one root
/// contribute one entry per distinct locator, not one entry per branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelationalAuthoritativeAllocationKind {
    /// One native shared storage allocation reachable through a partition.
    PartitionPayload,
    /// The partition-state object that owns one region's payload.
    PartitionStateObject,
    /// The root-side wrapper object that binds one region into a root.
    RootRegionObject,
    /// Root-owned metadata other than schema authority.
    RootMetadata,
    /// The schema authority allocation owned by one exact root.
    RootSchemaAuthority,
    /// A persistent region-set object in the root's reachability structure.
    RootReachabilitySetObject,
    /// A persistent region-map node in the root's reachability structure.
    RootReachabilityStructure,
    /// Replacement storage retained by the persistent reachability structure.
    RootReplacementStorage,
    /// Removal storage retained by the persistent reachability structure.
    RootRemovalStorage,
    /// The canonical commit artifact object itself.
    CanonicalCommitArtifact,
    /// The canonical payload owned by one commit artifact.
    CanonicalCommitPayload,
    /// The commit envelope object owned by one commit artifact.
    CanonicalCommitEnvelope,
    /// Nested owner storage reachable from one commit envelope.
    CanonicalCommitEnvelopeNested,
}

/// Runtime-affine identity of one authoritative allocation.
///
/// This locator is the sameness basis for physical deduplication. Equal
/// locators denote one allocation reached through possibly several branches;
/// unequal locators denote distinct allocations even when their byte counts
/// match. The runtime instance id keeps allocations of a cloned runtime
/// distinct from those of its source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelationalAuthoritativeAllocationLocator {
    runtime_instance_id: u64,
    kind: RelationalAuthoritativeAllocationKind,
    owner_id: u64,
    creation_owner_id: u64,
    partition_id: Option<PartitionId>,
}

impl RelationalAuthoritativeAllocationLocator {
    pub(in crate::inspection::mvcc) const fn new(
        runtime_instance_id: u64,
        kind: RelationalAuthoritativeAllocationKind,
        owner_id: u64,
        creation_owner_id: u64,
        partition_id: Option<PartitionId>,
    ) -> Self {
        Self {
            runtime_instance_id,
            kind,
            owner_id,
            creation_owner_id,
            partition_id,
        }
    }

    /// Instance id of the runtime that owns this allocation.
    ///
    /// Truth source: the observing runtime, stamped during the owner walk.
    pub const fn runtime_instance_id(self) -> u64 {
        self.runtime_instance_id
    }

    /// Which authoritative allocation this locator names.
    ///
    /// Truth source: the owner walk's classification of the visited root,
    /// region, or commit allocation.
    pub const fn kind(self) -> RelationalAuthoritativeAllocationKind {
        self.kind
    }

    /// Owner-issued id of the allocation itself: a region id for
    /// region-scoped kinds, a root allocation id for root-scoped kinds, and a
    /// commit id for canonical-commit kinds.
    ///
    /// Truth source: the owning root, region, or commit artifact.
    pub const fn owner_id(self) -> u64 {
        self.owner_id
    }

    /// Owner-issued id of the owner that first created this allocation.
    ///
    /// Truth source: the region's creation root for region-scoped kinds, and
    /// the allocation's own owner id otherwise. It differs from
    /// [`Self::owner_id`] exactly when storage created by one root is still
    /// shared by a later root, which is how structural sharing stays visible.
    pub const fn creation_owner_id(self) -> u64 {
        self.creation_owner_id
    }

    /// Partition this allocation belongs to, when the kind is
    /// partition-scoped.
    ///
    /// Truth source: the visited storage region. Root-scoped and
    /// canonical-commit kinds carry `None`.
    pub const fn partition_id(self) -> Option<PartitionId> {
        self.partition_id
    }
}

/// One authoritative allocation together with the authoritative byte count it
/// holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationalAuthoritativeAllocationObservation {
    locator: RelationalAuthoritativeAllocationLocator,
    authoritative_bytes: u64,
}

impl RelationalAuthoritativeAllocationObservation {
    pub(in crate::inspection::mvcc) const fn new(
        locator: RelationalAuthoritativeAllocationLocator,
        authoritative_bytes: u64,
    ) -> Self {
        Self {
            locator,
            authoritative_bytes,
        }
    }

    /// Runtime-affine identity of the observed allocation.
    ///
    /// Truth source: the owner walk. This is also the deduplication key behind
    /// every `unique_physical_*` total.
    pub const fn locator(self) -> RelationalAuthoritativeAllocationLocator {
        self.locator
    }

    /// Authoritative bytes held by this one allocation.
    ///
    /// Truth source: the owning root, region, or commit artifact, read live at
    /// observation time.
    ///
    /// Byte scope: the authoritative storage of this allocation only.
    /// Diagnostics, retention metadata, allocator bookkeeping, and optional
    /// caches belonging to the same owner are excluded.
    pub const fn authoritative_bytes(self) -> u64 {
        self.authoritative_bytes
    }
}

/// Runtime-affine, owner-issued identity for one immutable storage region.
///
/// This is the region-object projection of
/// [`RelationalAuthoritativeAllocationLocator`]: exactly the locators whose
/// kind is [`RelationalAuthoritativeAllocationKind::RootRegionObject`],
/// re-expressed as a region identity. Payload allocations may be shared across
/// distinct regions. This identity opens no authority
/// and cannot be turned back into a root, a branch, or a transaction binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelationalStorageRegionLocator {
    runtime_instance_id: u64,
    creation_root_id: u64,
    region_id: u64,
    partition_id: PartitionId,
}

impl RelationalStorageRegionLocator {
    pub(super) const fn new(
        runtime_instance_id: u64,
        creation_root_id: u64,
        region_id: u64,
        partition_id: PartitionId,
    ) -> Self {
        Self {
            runtime_instance_id,
            creation_root_id,
            region_id,
            partition_id,
        }
    }

    /// Instance id of the runtime that owns this region.
    ///
    /// Truth source: the observing runtime. Two runtimes that issue equal
    /// region ids still produce unequal locators.
    pub const fn runtime_instance_id(self) -> u64 {
        self.runtime_instance_id
    }

    /// Owner-issued id of the root that first created this region.
    ///
    /// Truth source: the region's creation root, not the selected root. A
    /// region shared by a later root keeps reporting its creating root, which
    /// is how region reuse stays observable.
    pub const fn root_id(self) -> u64 {
        self.creation_root_id
    }

    /// Owner-issued id of the region itself.
    ///
    /// Truth source: the visited storage region.
    pub const fn region_id(self) -> u64 {
        self.region_id
    }

    /// Partition whose authoritative payload this region holds.
    ///
    /// Truth source: the visited storage region.
    pub const fn partition_id(self) -> PartitionId {
        self.partition_id
    }
}
