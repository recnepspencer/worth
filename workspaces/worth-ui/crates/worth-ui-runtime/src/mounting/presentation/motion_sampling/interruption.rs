use super::velocity::{UiPresentationOutgoingCurve, UiPresentationSampleVelocity};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum UiPresentationMotionInstallation {
    Install {
        geometry: Option<[f32; 4]>,
        opacity_units: u16,
        start_velocity: UiPresentationSampleVelocity,
        duration_ticks: u32,
    },
    SnapToTarget,
}

/// The sample a successor track is displacing, plus the curve that produced it.
/// Position alone cannot tell a retarget how fast the content was already
/// moving, so the outgoing curve travels with it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct UiPresentationInterruptedSample {
    pub(super) geometry: Option<[f32; 4]>,
    pub(super) opacity_units: u16,
    pub(super) outgoing: Option<UiPresentationOutgoingCurve>,
}

pub(super) fn resolve(
    track: crate::runtime::motion::UiCommittedMotionTrack,
    current: Option<UiPresentationInterruptedSample>,
    reduced_motion: super::UiPresentationReducedMotionPosture,
) -> UiPresentationMotionInstallation {
    let declaration = track.declaration();
    if reduced_motion == super::UiPresentationReducedMotionPosture::Reduce
        && declaration.reduced_motion()
            == crate::runtime::motion::UiMotionReducedMotionPolicy::SystemRespecting
    {
        if declaration.decorative() {
            return UiPresentationMotionInstallation::SnapToTarget;
        }
        return UiPresentationMotionInstallation::Install {
            geometry: semantic_predecessor(track),
            opacity_units: predecessor_opacity_units(track),
            start_velocity: UiPresentationSampleVelocity::RESTING,
            duration_ticks: 1,
        };
    }
    let duration_ticks = declaration.duration_ticks();
    match track.retarget() {
        None => UiPresentationMotionInstallation::Install {
            geometry: semantic_predecessor(track),
            opacity_units: predecessor_opacity_units(track),
            start_velocity: UiPresentationSampleVelocity::RESTING,
            duration_ticks,
        },
        Some(crate::runtime::motion::UiMotionRetargetDisposition::Install {
            predecessor:
                crate::runtime::motion::UiMotionRetargetPredecessor::CurrentPresentationSample,
        }) => {
            let Some(interrupted) = current else {
                return UiPresentationMotionInstallation::Install {
                    geometry: semantic_predecessor(track),
                    opacity_units: predecessor_opacity_units(track),
                    start_velocity: UiPresentationSampleVelocity::RESTING,
                    duration_ticks,
                };
            };
            UiPresentationMotionInstallation::Install {
                geometry: interrupted.geometry,
                opacity_units: interrupted.opacity_units,
                start_velocity: interrupted.outgoing.map_or(
                    UiPresentationSampleVelocity::RESTING,
                    UiPresentationSampleVelocity::of_outgoing_curve,
                ),
                duration_ticks,
            }
        }
    }
}

pub(super) fn semantic_predecessor(
    track: crate::runtime::motion::UiCommittedMotionTrack,
) -> Option<[f32; 4]> {
    track
        .predecessor_geometry()
        .map(crate::runtime::motion::UiMotionSemanticGeometry::components)
}

pub(super) const fn predecessor_opacity_units(
    track: crate::runtime::motion::UiCommittedMotionTrack,
) -> u16 {
    if track.predecessor_visible() {
        u16::MAX
    } else {
        0
    }
}
