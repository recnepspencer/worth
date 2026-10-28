use crate::validation::{PhysicalArtifactScope, UntrustedPhysicalArtifact};

/// One pure integrity-inspection window. Scheduling, allocation, cancellation,
/// and runtime lifetime remain Store responsibilities.
#[derive(Debug, Clone, Copy)]
pub struct PhysicalIntegrityScrubWindow<'media> {
    ordinal: u64,
    scope: PhysicalArtifactScope,
    artifact: UntrustedPhysicalArtifact<'media>,
    checkpoint_certificates: Option<UntrustedPhysicalArtifact<'media>>,
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
            checkpoint_certificates: None,
        }
    }

    /// A footer window whose immediately preceding custody certificates were
    /// acquired with it. These bytes remain untrusted and grant no authority.
    pub fn checkpoint_footer_with_certificates(
        ordinal: u64,
        scope: PhysicalArtifactScope,
        artifact: UntrustedPhysicalArtifact<'media>,
        certificates: UntrustedPhysicalArtifact<'media>,
    ) -> Option<Self> {
        if !scope.is_checkpoint_footer()
            || certificates.byte_count()
                > worth_store_physical_format::MAX_CHECKPOINT_CERTIFICATE_BYTES
            || certificates.byte_count() > scope.byte_range().offset()
        {
            return None;
        }
        Some(Self {
            ordinal,
            scope,
            artifact,
            checkpoint_certificates: Some(certificates),
        })
    }

    pub(crate) fn checkpoint_certificates(self) -> &'media [u8] {
        self.checkpoint_certificates
            .map_or(&[], |input| input.bytes())
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
        PhysicalIntegrityScrubWindow {
            ordinal: self.ordinal,
            scope: self.scope,
            artifact: UntrustedPhysicalArtifact::from_bounded_bytes(self.artifact.bytes()),
            checkpoint_certificates: self
                .checkpoint_certificates
                .map(|input| UntrustedPhysicalArtifact::from_bounded_bytes(input.bytes())),
        }
    }
}
