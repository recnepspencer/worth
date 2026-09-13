use worth_store_physical_format::{CheckpointStreamFooter, PhysicalCheckpointIdentity};

use super::super::super::{PhysicalArtifactScope, UntrustedPhysicalArtifact};

/// Checksum-valid footer framing used only to route the records whose
/// selective aggregates still require validation.
#[derive(Debug)]
pub struct IntegrityValidatedCheckpointFooterEnvelope<'media> {
    scope: PhysicalArtifactScope,
    footer: CheckpointStreamFooter,
    inspected: UntrustedPhysicalArtifact<'media>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckpointFooterRoutingProjection {
    footer: CheckpointStreamFooter,
    footer_offset: u64,
}

impl<'media> IntegrityValidatedCheckpointFooterEnvelope<'media> {
    pub(crate) fn new(
        scope: PhysicalArtifactScope,
        footer: CheckpointStreamFooter,
        inspected: UntrustedPhysicalArtifact<'media>,
    ) -> Option<Self> {
        if !scope.is_checkpoint_footer()
            || footer.identity() != scope.checkpoint_identity()?
            || inspected.byte_count() != scope.byte_range().length()
        {
            return None;
        }
        Some(Self {
            scope,
            footer,
            inspected,
        })
    }

    pub const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }

    pub const fn checkpoint_identity(&self) -> PhysicalCheckpointIdentity {
        self.footer.identity()
    }

    pub const fn routing_projection(&self) -> CheckpointFooterRoutingProjection {
        CheckpointFooterRoutingProjection {
            footer: self.footer,
            footer_offset: self.scope.byte_range().offset(),
        }
    }

    pub fn matches_input(&self, input: UntrustedPhysicalArtifact<'media>) -> bool {
        self.inspected.same_incarnation(input)
    }
}

impl CheckpointFooterRoutingProjection {
    pub const fn footer(self) -> CheckpointStreamFooter {
        self.footer
    }

    pub const fn footer_offset(self) -> u64 {
        self.footer_offset
    }
}
