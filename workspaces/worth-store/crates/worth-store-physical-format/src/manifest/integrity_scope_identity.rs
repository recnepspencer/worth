use core::num::NonZeroU64;

use crate::{
    FreeSpaceBlockReference, ManifestBlockReference, PhysicalGeneration,
    SegmentManifestBlockReference,
};

/// Canonical identity shared by the routed trees published by one root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysicalTreeIdentity(NonZeroU64);

impl PhysicalTreeIdentity {
    pub const fn new(value: u64) -> Option<Self> {
        match NonZeroU64::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

/// Expected CRC32C over one complete durable child artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DurableArtifactCrc32c(u32);

impl DurableArtifactCrc32c {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootRoutingBlockScopeIdentity {
    tree: PhysicalTreeIdentity,
    reference: ManifestBlockReference,
}

impl RootRoutingBlockScopeIdentity {
    pub const fn new(tree: PhysicalTreeIdentity, reference: ManifestBlockReference) -> Self {
        Self { tree, reference }
    }

    pub const fn tree(self) -> PhysicalTreeIdentity {
        self.tree
    }

    pub const fn reference(self) -> ManifestBlockReference {
        self.reference
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentMembershipBlockScopeIdentity {
    tree: PhysicalTreeIdentity,
    reference: SegmentManifestBlockReference,
}

impl SegmentMembershipBlockScopeIdentity {
    pub const fn new(tree: PhysicalTreeIdentity, reference: SegmentManifestBlockReference) -> Self {
        Self { tree, reference }
    }

    pub const fn tree(self) -> PhysicalTreeIdentity {
        self.tree
    }

    pub const fn reference(self) -> SegmentManifestBlockReference {
        self.reference
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreeSpaceHeaderScopeIdentity {
    generation: PhysicalGeneration,
    tree: PhysicalTreeIdentity,
    root: Option<FreeSpaceBlockReference>,
    complete_child_checksum: DurableArtifactCrc32c,
}

impl FreeSpaceHeaderScopeIdentity {
    pub const fn new(
        generation: PhysicalGeneration,
        tree: PhysicalTreeIdentity,
        root: Option<FreeSpaceBlockReference>,
        complete_child_checksum: DurableArtifactCrc32c,
    ) -> Self {
        Self {
            generation,
            tree,
            root,
            complete_child_checksum,
        }
    }

    pub const fn generation(self) -> PhysicalGeneration {
        self.generation
    }

    pub const fn tree(self) -> PhysicalTreeIdentity {
        self.tree
    }

    pub const fn root(self) -> Option<FreeSpaceBlockReference> {
        self.root
    }

    pub const fn complete_child_checksum(self) -> DurableArtifactCrc32c {
        self.complete_child_checksum
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreeSpaceMembershipBlockScopeIdentity {
    tree: PhysicalTreeIdentity,
    reference: FreeSpaceBlockReference,
}

impl FreeSpaceMembershipBlockScopeIdentity {
    pub const fn new(tree: PhysicalTreeIdentity, reference: FreeSpaceBlockReference) -> Self {
        Self { tree, reference }
    }

    pub const fn tree(self) -> PhysicalTreeIdentity {
        self.tree
    }

    pub const fn reference(self) -> FreeSpaceBlockReference {
        self.reference
    }
}
