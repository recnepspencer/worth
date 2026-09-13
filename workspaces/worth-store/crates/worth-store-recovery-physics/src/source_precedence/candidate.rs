use worth_store_physical_format::{DurablePhysicalRootManifest, DurableRootSelector};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalRootSourceCandidate {
    selector: DurableRootSelector,
    manifest: DurablePhysicalRootManifest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalRootSlotObservation {
    Absent,
    SelectorRejected(PhysicalRootSelectorDenial),
    RootRejected {
        denial: PhysicalRootManifestDenial,
        selector: DurableRootSelector,
    },
    Candidate(PhysicalRootSourceCandidate),
}

impl PhysicalRootSlotObservation {
    pub fn rejection(&self) -> Option<(PhysicalRootCandidateDenial, Option<DurableRootSelector>)> {
        match self {
            Self::SelectorRejected(denial) => Some(((*denial).into(), None)),
            Self::RootRejected { denial, selector } => Some(((*denial).into(), Some(*selector))),
            Self::Absent | Self::Candidate(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRootSelectorDenial {
    Integrity,
    AuthorityMismatch,
    Conflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRootManifestDenial {
    FormatMismatch,
    GenerationMismatch,
    Integrity,
    Conflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRootCandidateDenial {
    RootFormatMismatch,
    RootGenerationMismatch,
    SelectorIntegrity,
    SelectorAuthorityMismatch,
    RootIntegrity,
    SelectorConflict,
    RootConflict,
}

impl From<PhysicalRootSelectorDenial> for PhysicalRootCandidateDenial {
    fn from(denial: PhysicalRootSelectorDenial) -> Self {
        match denial {
            PhysicalRootSelectorDenial::Integrity => Self::SelectorIntegrity,
            PhysicalRootSelectorDenial::AuthorityMismatch => Self::SelectorAuthorityMismatch,
            PhysicalRootSelectorDenial::Conflict => Self::SelectorConflict,
        }
    }
}

impl From<PhysicalRootManifestDenial> for PhysicalRootCandidateDenial {
    fn from(denial: PhysicalRootManifestDenial) -> Self {
        match denial {
            PhysicalRootManifestDenial::FormatMismatch => Self::RootFormatMismatch,
            PhysicalRootManifestDenial::GenerationMismatch => Self::RootGenerationMismatch,
            PhysicalRootManifestDenial::Integrity => Self::RootIntegrity,
            PhysicalRootManifestDenial::Conflict => Self::RootConflict,
        }
    }
}

impl PhysicalRootSourceCandidate {
    pub(super) fn from_structured_observation(
        selector: DurableRootSelector,
        manifest: DurablePhysicalRootManifest,
        manifest_format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    ) -> Result<Self, PhysicalRootCandidateDenial> {
        if selector.format() != manifest_format {
            return Err(PhysicalRootCandidateDenial::RootFormatMismatch);
        }
        if selector.root_generation() != manifest.generation() {
            return Err(PhysicalRootCandidateDenial::RootGenerationMismatch);
        }
        Ok(Self { selector, manifest })
    }

    pub const fn selector(&self) -> DurableRootSelector {
        self.selector
    }

    pub const fn manifest(&self) -> &DurablePhysicalRootManifest {
        &self.manifest
    }
}
