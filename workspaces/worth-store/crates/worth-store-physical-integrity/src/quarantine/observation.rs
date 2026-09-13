use core::num::NonZeroU64;

use crate::localization::PhysicalDamageLocalization;
use crate::validation::PhysicalArtifactScope;

use super::posture::PhysicalQuarantinePosture;

/// Descriptive quarantine observation with no path, media capability, repair,
/// release, or reachability mutation surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalQuarantineObservation {
    scope: PhysicalArtifactScope,
    localization: PhysicalDamageLocalization,
    posture: PhysicalQuarantinePosture,
    observed_at: NonZeroU64,
}

impl PhysicalQuarantineObservation {
    pub const fn new(
        localization: PhysicalDamageLocalization,
        posture: PhysicalQuarantinePosture,
        observed_at: NonZeroU64,
    ) -> Self {
        Self {
            scope: localization.scope(),
            localization,
            posture,
            observed_at,
        }
    }

    pub const fn scope(self) -> PhysicalArtifactScope {
        self.scope
    }

    pub const fn localization(self) -> PhysicalDamageLocalization {
        self.localization
    }

    pub const fn posture(self) -> PhysicalQuarantinePosture {
        self.posture
    }

    pub const fn observed_at(self) -> NonZeroU64 {
        self.observed_at
    }
}
