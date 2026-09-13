use worth_store::physical_runtime::ObservedWalArtifact;
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};

/// One exact frame range borrowed from a C.4 bounded WAL observation.
pub(crate) struct ObservedWalFrameSource<'media> {
    observed: &'media ObservedWalArtifact,
    scope: PhysicalArtifactScope,
    relative_range: PhysicalByteRange,
}

impl<'media> ObservedWalFrameSource<'media> {
    pub(crate) const fn new(
        observed: &'media ObservedWalArtifact,
        scope: PhysicalArtifactScope,
        relative_range: PhysicalByteRange,
    ) -> Self {
        Self {
            observed,
            scope,
            relative_range,
        }
    }

    pub(in crate::integrity_ingress) const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }

    pub(in crate::integrity_ingress) const fn observed(&self) -> &'media ObservedWalArtifact {
        self.observed
    }

    pub(in crate::integrity_ingress) const fn relative_range(&self) -> PhysicalByteRange {
        self.relative_range
    }
}
