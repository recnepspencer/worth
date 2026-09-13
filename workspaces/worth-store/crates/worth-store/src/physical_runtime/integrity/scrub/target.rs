use super::PhysicalIntegrityScrubRequestDenial;
use worth_store_physical_format::{PhysicalArtifactReadRange, PhysicalArtifactReadTarget};
use worth_store_physical_integrity::PhysicalArtifactScope;

/// A location and an independently supplied expected scope, not preloaded bytes
/// or read authority. Store resolves the location through its own media owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalIntegrityScrubTarget {
    scope: PhysicalArtifactScope,
    range: PhysicalArtifactReadRange,
}

impl PhysicalIntegrityScrubTarget {
    pub fn new(
        target: PhysicalArtifactReadTarget,
        scope: PhysicalArtifactScope,
    ) -> Result<Self, PhysicalIntegrityScrubRequestDenial> {
        use PhysicalArtifactReadTarget as Target;
        let matches = match target {
            Target::Record(artifact) => {
                super::super::resident_admission::artifact_matches_scope(artifact, scope)
            }
            Target::Wal(identity) => scope.wal_segment_identity() == Some(identity),
            Target::Checkpoint(identity) => scope.checkpoint_identity() == Some(identity),
            Target::PhysicalWork(identity) => {
                scope.physical_work_obligation_identity() == Some(identity)
            }
        };
        if !matches {
            return Err(PhysicalIntegrityScrubRequestDenial::TargetScopeMismatch);
        }
        let length = u32::try_from(scope.byte_range().length())
            .map_err(|_| PhysicalIntegrityScrubRequestDenial::WindowBoundExceeded)?;
        let range = PhysicalArtifactReadRange::new(target, scope.byte_range().offset(), length)
            .ok_or(PhysicalIntegrityScrubRequestDenial::TargetScopeMismatch)?;
        Ok(Self { scope, range })
    }

    pub const fn scope(self) -> PhysicalArtifactScope {
        self.scope
    }
    pub const fn range(self) -> PhysicalArtifactReadRange {
        self.range
    }
}
