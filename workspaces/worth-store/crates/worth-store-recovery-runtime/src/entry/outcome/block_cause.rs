//! Why recovery blocked: a limit it was admitted ran out, or something it
//! observed failed. The two never share a spelling. A limit is built only
//! from the refusal of the budget that owns its dimension, so it always
//! carries that budget's counts.

use worth_foundational::ExhaustedLimit;
use worth_store_recovery_physics::PhysicsBound;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryLimitDimension {
    SelectorCandidates,
    ManifestBytes,
    ManifestEntries,
    WalSegments,
    WalFrames,
    WalBytes,
    DistinctPagesAndExtents,
    ObservationBytes,
    OperationBindings,
    RedoTargets,
    RedoBytes,
    StagingBytes,
    RecoveryMemoryBytes,
    DirtyFrames,
    PublicationEffects,
}

/// A limit recovery ran out of. `observed` is at least what the refused step
/// needed, never the whole need: recovery stops at the first step that
/// crosses `admitted`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRecoveryLimitFailure {
    dimension: PhysicalRecoveryLimitDimension,
    observed: u64,
    admitted: u64,
}

impl PhysicalRecoveryLimitFailure {
    pub const fn dimension(&self) -> PhysicalRecoveryLimitDimension {
        self.dimension
    }

    pub const fn observed(&self) -> u64 {
        self.observed
    }

    pub const fn admitted(&self) -> u64 {
        self.admitted
    }

    const fn of(dimension: PhysicalRecoveryLimitDimension, observed: u64, admitted: u64) -> Self {
        Self {
            dimension,
            observed,
            admitted,
        }
    }
}

impl From<ExhaustedLimit<PhysicalRecoveryLimitDimension>> for PhysicalRecoveryLimitFailure {
    fn from(limit: ExhaustedLimit<PhysicalRecoveryLimitDimension>) -> Self {
        Self::of(limit.dimension(), limit.observed(), limit.admitted())
    }
}

impl From<ExhaustedLimit<PhysicsBound>> for PhysicalRecoveryLimitFailure {
    /// Physics is handed recovery's whole limits for page facts, and the
    /// custody checks' memory out of recovery memory.
    fn from(limit: ExhaustedLimit<PhysicsBound>) -> Self {
        let dimension = match limit.dimension() {
            PhysicsBound::ManifestEntries => PhysicalRecoveryLimitDimension::ManifestEntries,
            PhysicsBound::DistinctPagesAndExtents => {
                PhysicalRecoveryLimitDimension::DistinctPagesAndExtents
            }
            PhysicsBound::ResidentBytes | PhysicsBound::RetainedBytes => {
                PhysicalRecoveryLimitDimension::RecoveryMemoryBytes
            }
        };
        Self::of(dimension, limit.observed(), limit.admitted())
    }
}

impl From<crate::orchestration::ExceededManifestEntries> for PhysicalRecoveryLimitFailure {
    /// The walk's entry budget is handed recovery's declared manifest
    /// entries, in their own counts.
    fn from(limit: crate::orchestration::ExceededManifestEntries) -> Self {
        Self::of(
            PhysicalRecoveryLimitDimension::ManifestEntries,
            limit.observed(),
            limit.admitted(),
        )
    }
}

/// Why a block stopped recovery. A limit says nothing about the media: the
/// same media may recover under wider limits. Both name the phase that
/// stopped: one ran out of a limit, the other's observation or check failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryBlockCause {
    Limit {
        phase: super::PhysicalRecoveryBlockKind,
        limit: PhysicalRecoveryLimitFailure,
    },
    Damage(super::PhysicalRecoveryBlockKind),
}

impl PhysicalRecoveryBlockCause {
    /// `phase` ran out of `limit`, where it ran out of one; otherwise its
    /// observation or check failed.
    pub(crate) fn of(
        phase: super::PhysicalRecoveryBlockKind,
        limit: Option<PhysicalRecoveryLimitFailure>,
    ) -> Self {
        limit.map_or(Self::Damage(phase), |limit| Self::Limit { phase, limit })
    }

    /// The phase that stopped, for a limit or for damage alike.
    pub const fn phase(&self) -> super::PhysicalRecoveryBlockKind {
        match self {
            Self::Limit { phase, .. } | Self::Damage(phase) => *phase,
        }
    }

    pub const fn limit(&self) -> Option<PhysicalRecoveryLimitFailure> {
        match self {
            Self::Limit { limit, .. } => Some(*limit),
            Self::Damage(_) => None,
        }
    }

    /// The phase whose observation or check failed; `None` for a limit.
    pub const fn damage(&self) -> Option<super::PhysicalRecoveryBlockKind> {
        match self {
            Self::Limit { .. } => None,
            Self::Damage(kind) => Some(*kind),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::PhysicalRecoveryBlockKind, PhysicalRecoveryBlockCause as Cause,
        PhysicalRecoveryLimitDimension as Dimension, PhysicalRecoveryLimitFailure,
    };
    use crate::orchestration::recovery_limit_for_test;
    use worth_store_recovery_physics::{test_support::physics_limit_for_test, PhysicsBound};

    fn named(limit: PhysicalRecoveryLimitFailure) -> (Dimension, u64, u64) {
        (limit.dimension(), limit.observed(), limit.admitted())
    }

    #[test]
    fn a_physics_bound_is_recovery_s_limit_with_both_counts() {
        for (bound, dimension) in [
            (PhysicsBound::ManifestEntries, Dimension::ManifestEntries),
            (
                PhysicsBound::DistinctPagesAndExtents,
                Dimension::DistinctPagesAndExtents,
            ),
            (PhysicsBound::ResidentBytes, Dimension::RecoveryMemoryBytes),
            (PhysicsBound::RetainedBytes, Dimension::RecoveryMemoryBytes),
        ] {
            let limit = PhysicalRecoveryLimitFailure::from(physics_limit_for_test(bound, 11, 6));
            assert_eq!(named(limit), (dimension, 11, 6), "{bound:?}");
        }
    }

    #[test]
    fn a_block_with_a_limit_is_that_limit_in_its_phase_and_names_no_damage() {
        let limit = recovery_limit_for_test(Dimension::StagingBytes, 9, 8).into();
        let kind = PhysicalRecoveryBlockKind::Staging;
        let cause = Cause::of(kind, Some(limit));
        assert_eq!(cause, Cause::Limit { phase: kind, limit });
        assert_eq!(
            (cause.phase(), cause.limit(), cause.damage()),
            (kind, Some(limit), None)
        );
        let cause = Cause::of(kind, None);
        assert_eq!(cause, Cause::Damage(kind));
        assert_eq!(
            (cause.phase(), cause.limit(), cause.damage()),
            (kind, None, Some(kind))
        );
    }
}
