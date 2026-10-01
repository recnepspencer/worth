use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_physical_format::BTREE_NODE_VERSION;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum RegisteredPhysicalLayout {
    C5InlineSlottedBTree,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum RegisteredFrameKind {
    BTreeNodeV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum RegisteredSourceAuthority {
    SelectedPublicationDeclarationTreeAndClaimClosure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum RegisteredIntegrityClass {
    C9ProtectedBTreeNode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum RegisteredRetention {
    SelectedDirectoryCowPathRetirement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum RegisteredRecoveryParticipation {
    ClassifiedNodeAndDirectoryWalRedo,
}

/// The source closure from which a derived family can be rebuilt after all
/// of its nodes are discarded. Neither variant names a derived root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum DerivedFamilyRebuildBasis {
    SelectedBlobGenerationPublications,
    SelectedChunkOccurrenceClaims,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum RegisteredIndexShape {
    Point,
    Range,
    Prefix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum RegisteredFamilyDenial {
    UnregisteredFamily(DurableArtifactFamilyId),
    ShapeNotAdmitted {
        family: DurableArtifactFamilyId,
        shape: RegisteredIndexShape,
    },
}

/// A Store-owned registration. `family_code` is an explicit on-media code,
/// never a Rust enum discriminant or an input supplied by a caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct RegisteredDerivedFamily {
    family: DurableArtifactFamilyId,
    family_code: u16,
    physical_layout: RegisteredPhysicalLayout,
    frame_kind: RegisteredFrameKind,
    frame_version: u8,
    cell_shape: (usize, usize),
    integrity_class: RegisteredIntegrityClass,
    rebuild_basis: DerivedFamilyRebuildBasis,
    source_authority: RegisteredSourceAuthority,
    retention: RegisteredRetention,
    recovery: RegisteredRecoveryParticipation,
    live_writer: bool,
    admitted_shapes: &'static [RegisteredIndexShape],
}

impl RegisteredDerivedFamily {
    pub(in crate::physical_runtime) const fn family(self) -> DurableArtifactFamilyId {
        self.family
    }

    pub(in crate::physical_runtime) const fn family_code(self) -> u16 {
        self.family_code
    }

    pub(in crate::physical_runtime) const fn physical_layout(self) -> RegisteredPhysicalLayout {
        self.physical_layout
    }

    pub(in crate::physical_runtime) const fn frame_version(self) -> u8 {
        self.frame_version
    }

    pub(in crate::physical_runtime) const fn frame_kind(self) -> RegisteredFrameKind {
        self.frame_kind
    }

    pub(in crate::physical_runtime) const fn source_authority(self) -> RegisteredSourceAuthority {
        self.source_authority
    }

    pub(in crate::physical_runtime) const fn cell_shape(self) -> (usize, usize) {
        self.cell_shape
    }

    pub(in crate::physical_runtime) const fn integrity_class(self) -> RegisteredIntegrityClass {
        self.integrity_class
    }

    pub(in crate::physical_runtime) const fn retention(self) -> RegisteredRetention {
        self.retention
    }

    pub(in crate::physical_runtime) const fn recovery(self) -> RegisteredRecoveryParticipation {
        self.recovery
    }

    pub(in crate::physical_runtime) const fn live_writer(self) -> bool {
        self.live_writer
    }

    pub(in crate::physical_runtime) const fn rebuild_basis(self) -> DerivedFamilyRebuildBasis {
        self.rebuild_basis
    }

    pub(in crate::physical_runtime) fn admit_shape(
        self,
        shape: RegisteredIndexShape,
    ) -> Result<(), RegisteredFamilyDenial> {
        self.admitted_shapes.contains(&shape).then_some(()).ok_or(
            RegisteredFamilyDenial::ShapeNotAdmitted {
                family: self.family,
                shape,
            },
        )
    }
}

const BLOB_CATALOG_SHAPES: &[RegisteredIndexShape] = &[
    RegisteredIndexShape::Point,
    RegisteredIndexShape::Range,
    RegisteredIndexShape::Prefix,
];

const REGISTERED_FAMILIES: [RegisteredDerivedFamily; 2] = [
    RegisteredDerivedFamily {
        family: DurableArtifactFamilyId::BlobCatalog,
        family_code: 1,
        physical_layout: RegisteredPhysicalLayout::C5InlineSlottedBTree,
        frame_kind: RegisteredFrameKind::BTreeNodeV1,
        frame_version: BTREE_NODE_VERSION,
        cell_shape: (24, 24),
        integrity_class: RegisteredIntegrityClass::C9ProtectedBTreeNode,
        rebuild_basis: DerivedFamilyRebuildBasis::SelectedBlobGenerationPublications,
        source_authority:
            RegisteredSourceAuthority::SelectedPublicationDeclarationTreeAndClaimClosure,
        retention: RegisteredRetention::SelectedDirectoryCowPathRetirement,
        recovery: RegisteredRecoveryParticipation::ClassifiedNodeAndDirectoryWalRedo,
        live_writer: true,
        admitted_shapes: BLOB_CATALOG_SHAPES,
    },
    RegisteredDerivedFamily {
        family: DurableArtifactFamilyId::DedupeIndex,
        family_code: 2,
        physical_layout: RegisteredPhysicalLayout::C5InlineSlottedBTree,
        frame_kind: RegisteredFrameKind::BTreeNodeV1,
        frame_version: BTREE_NODE_VERSION,
        cell_shape: (64, 56),
        integrity_class: RegisteredIntegrityClass::C9ProtectedBTreeNode,
        rebuild_basis: DerivedFamilyRebuildBasis::SelectedChunkOccurrenceClaims,
        source_authority:
            RegisteredSourceAuthority::SelectedPublicationDeclarationTreeAndClaimClosure,
        retention: RegisteredRetention::SelectedDirectoryCowPathRetirement,
        recovery: RegisteredRecoveryParticipation::ClassifiedNodeAndDirectoryWalRedo,
        live_writer: true,
        admitted_shapes: &[RegisteredIndexShape::Point],
    },
];

/// Live construction-owned inventory. Durable roots are intentionally absent:
/// they are selected and protected from the C.5 root on each read.
pub(in crate::physical_runtime) struct PhysicalArtifactFamilyRegistry {
    families: [RegisteredDerivedFamily; REGISTERED_FAMILIES.len()],
}

impl PhysicalArtifactFamilyRegistry {
    pub(in crate::physical_runtime) const fn install() -> Self {
        Self {
            families: REGISTERED_FAMILIES,
        }
    }

    pub(in crate::physical_runtime) fn btree(
        &self,
        family: DurableArtifactFamilyId,
    ) -> Result<RegisteredDerivedFamily, RegisteredFamilyDenial> {
        self.families
            .iter()
            .copied()
            .find(|registered| registered.family == family && registered.live_writer)
            .ok_or(RegisteredFamilyDenial::UnregisteredFamily(family))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_registry_distinguishes_derived_families_from_authoritative_blob_records() {
        let registry = PhysicalArtifactFamilyRegistry::install();
        let catalog = registry
            .btree(DurableArtifactFamilyId::BlobCatalog)
            .unwrap();
        let dedupe = registry
            .btree(DurableArtifactFamilyId::DedupeIndex)
            .unwrap();
        assert_ne!(catalog.family_code(), dedupe.family_code());
        assert_eq!(
            catalog.rebuild_basis(),
            DerivedFamilyRebuildBasis::SelectedBlobGenerationPublications
        );
        assert_eq!(
            dedupe.rebuild_basis(),
            DerivedFamilyRebuildBasis::SelectedChunkOccurrenceClaims
        );
        for (family, shape) in [(catalog, (24, 24)), (dedupe, (64, 56))] {
            assert!(family.live_writer());
            assert_eq!(
                family.physical_layout(),
                RegisteredPhysicalLayout::C5InlineSlottedBTree
            );
            assert_eq!(family.frame_version(), BTREE_NODE_VERSION);
            assert_eq!(family.frame_kind(), RegisteredFrameKind::BTreeNodeV1);
            assert_eq!(
                family.source_authority(),
                RegisteredSourceAuthority::SelectedPublicationDeclarationTreeAndClaimClosure
            );
            assert_eq!(family.cell_shape(), shape);
            assert_eq!(
                family.integrity_class(),
                RegisteredIntegrityClass::C9ProtectedBTreeNode
            );
            assert_eq!(
                family.retention(),
                RegisteredRetention::SelectedDirectoryCowPathRetirement
            );
            assert_eq!(
                family.recovery(),
                RegisteredRecoveryParticipation::ClassifiedNodeAndDirectoryWalRedo
            );
        }
        assert_eq!(
            registry.btree(DurableArtifactFamilyId::BlobManifest),
            Err(RegisteredFamilyDenial::UnregisteredFamily(
                DurableArtifactFamilyId::BlobManifest
            ))
        );
    }
}
