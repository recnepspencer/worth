use crate::validation::{PhysicalArtifactScope, UntrustedPhysicalArtifact};

/// One pure integrity-inspection window. Scheduling, allocation, cancellation,
/// and runtime lifetime remain Store responsibilities.
#[derive(Debug, Clone, Copy)]
pub struct PhysicalIntegrityScrubWindow<'media> {
    ordinal: u64,
    scope: PhysicalArtifactScope,
    artifact: UntrustedPhysicalArtifact<'media>,
}

impl<'media> PhysicalIntegrityScrubWindow<'media> {
    pub const fn new(
        ordinal: u64,
        scope: PhysicalArtifactScope,
        artifact: UntrustedPhysicalArtifact<'media>,
    ) -> Self {
        Self {
            ordinal,
            scope,
            artifact,
        }
    }

    pub const fn ordinal(self) -> u64 {
        self.ordinal
    }

    pub const fn scope(self) -> PhysicalArtifactScope {
        self.scope
    }

    pub const fn artifact(self) -> UntrustedPhysicalArtifact<'media> {
        self.artifact
    }

    /// Reborrows this window for one Store-managed dispatch call.
    pub fn reborrow(&self) -> PhysicalIntegrityScrubWindow<'_> {
        PhysicalIntegrityScrubWindow::new(
            self.ordinal,
            self.scope,
            UntrustedPhysicalArtifact::from_bounded_bytes(self.artifact.bytes()),
        )
    }
}
