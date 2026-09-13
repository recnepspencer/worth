use super::{PhysicalIntegrityChecksumDeclaration, PhysicalIntegrityFormatVersion};

/// Canonical current physical artifact family identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhysicalIntegrityArtifactFamily {
    NamespaceIdentity,
    PhysicalWorkObligation,
    PageFrame,
    ExtentChunk,
    WalFrame,
    CheckpointStreamHeader,
    CheckpointDirtyBasis,
    CheckpointBindingCompaction,
    CheckpointBinding,
    CheckpointFooter,
    BootstrapCatalog,
    CurrentRootSelector,
    PreviousRootSelector,
    RootManifest,
    RootRoutingBlock,
    SegmentMembership,
    ExtentManifest,
    FreeSpaceHeader,
    FreeSpaceMembershipBlock,
}

/// Complete descriptive checksum declaration for one current artifact family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicalIntegrityFormatDeclaration {
    family: PhysicalIntegrityArtifactFamily,
    version: PhysicalIntegrityFormatVersion,
    checksums: &'static [PhysicalIntegrityChecksumDeclaration],
}

impl PhysicalIntegrityFormatDeclaration {
    pub const fn new(
        family: PhysicalIntegrityArtifactFamily,
        version: PhysicalIntegrityFormatVersion,
        checksums: &'static [PhysicalIntegrityChecksumDeclaration],
    ) -> Self {
        Self {
            family,
            version,
            checksums,
        }
    }

    pub const fn family(self) -> PhysicalIntegrityArtifactFamily {
        self.family
    }

    pub const fn version(self) -> PhysicalIntegrityFormatVersion {
        self.version
    }

    pub const fn checksums(self) -> &'static [PhysicalIntegrityChecksumDeclaration] {
        self.checksums
    }
}
