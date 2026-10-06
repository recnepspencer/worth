//! Why the checkpoint-source custody went unobserved: the public denial, and
//! the limit with its own counts where observation met one.

use crate::entry::{
    PhysicalRecoveryLimitDimension, PhysicalRecoveryReleaseHeadWalkDenial as WalkDenial,
    PhysicalRecoverySelectedReleaseHeadDenial as Denial,
};
use crate::orchestration::planning::page_observation::PageLimit;
use crate::orchestration::planning::selected_source_inventory::{
    ResidentTraceDenial, RoutesFailure,
};

/// A denial, and the limit observation met where it carries the counts.
#[derive(Debug, PartialEq)]
pub(super) struct Refused {
    pub(super) denial: Denial,
    pub(super) limit: Option<PageLimit>,
}

impl From<Denial> for Refused {
    fn from(denial: Denial) -> Self {
        Self {
            denial,
            limit: None,
        }
    }
}

/// Why the source routes went unobserved: what observation says of them, or
/// the room this phase had no allowance to hold. The second is never an
/// entry limit.
pub(super) fn routes_denial(failure: RoutesFailure<ResidentTraceDenial>) -> Refused {
    match failure {
        RoutesFailure::Observation(failure) => match failure.evidence() {
            Ok(page) => Denial::SourceRoutes(page).into(),
            Err(limit) => Refused {
                denial: limit_denial(limit),
                limit: Some(limit),
            },
        },
        RoutesFailure::Held(ResidentTraceDenial::ResidentBoundExceeded) => {
            Denial::ResidentBoundExceeded.into()
        }
        // The head walk's allocation denial is the one this phase can name.
        RoutesFailure::Held(ResidentTraceDenial::Allocation { requested, cause }) => {
            Denial::HeadWalk(WalkDenial::Allocation { requested, cause }).into()
        }
    }
}

/// The public denial of a limit the routes met. Only the entry budget and the
/// reader's bytes refuse a route observation.
fn limit_denial(limit: PageLimit) -> Denial {
    match limit {
        PageLimit::Reader(_) => Denial::ObservationByteLimit,
        PageLimit::Recovery(limit) => {
            use PhysicalRecoveryLimitDimension as Dimension;
            match limit.dimension() {
                Dimension::ManifestEntries => Denial::ManifestEntryLimit,
                Dimension::ObservationBytes => Denial::ObservationByteLimit,
                Dimension::SelectorCandidates
                | Dimension::ManifestBytes
                | Dimension::WalSegments
                | Dimension::WalFrames
                | Dimension::WalBytes
                | Dimension::DistinctPagesAndExtents
                | Dimension::OperationBindings
                | Dimension::RedoTargets
                | Dimension::RedoBytes
                | Dimension::StagingBytes
                | Dimension::RecoveryMemoryBytes
                | Dimension::DirtyFrames
                | Dimension::PublicationEffects => Denial::WalkLimits,
            }
        }
    }
}
