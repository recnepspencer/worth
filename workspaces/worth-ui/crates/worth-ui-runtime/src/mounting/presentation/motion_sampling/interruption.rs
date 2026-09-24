use super::velocity::{UiPresentationOutgoingCurve, UiPresentationSampleVelocity};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum UiPresentationMotionInstallation {
    Install {
        geometry: Option<[f32; 4]>,
        opacity_units: u16,
        start_velocity: UiPresentationSampleVelocity,
        duration_ticks: u32,
        /// The tick the track's curve starts at. A retarget departs from a
        /// sample already on screen, so its clock runs from that sample's tick
        /// and its first tick moves on from it. A fresh track starts at the
        /// first tick that samples it, whose frame shows its starting point.
        start_tick: Option<u64>,
    },
    SnapToTarget,
}

/// The sample a successor track is displacing, plus the curve that produced it.
/// Position alone cannot tell a retarget how fast the content was already
/// moving, so the outgoing curve travels with it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct UiPresentationInterruptedSample {
    pub(super) tick: u64,
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
    if reduced_motion == super::UiPresentationReducedMotionPosture::Reduce {
        // Arriving as the track installs is right for anything whose
        // installation mints the frame that shows it: the entrance carries the
        // arrived sample, so the reader sees the destination and never the
        // journey. A Scroll content group is the one thing it is wrong for. A
        // settle submits into the frame already on screen, so there is no
        // entrance to carry the sample and nothing would present it; the
        // offset the sample settles would never learn where the content went,
        // and a reader who asked for less motion would get none. The tick path
        // snaps that track on its first tick instead, which is the same
        // arrival by way of a frame that can carry it.
        if declaration.settles_directly_under_reduced_motion()
            && track.target().scope() != crate::runtime::motion::UiMotionTargetScope::ScrollContents
        {
            return UiPresentationMotionInstallation::SnapToTarget;
        }
        if declaration.shortens_under_reduced_motion() {
            return UiPresentationMotionInstallation::Install {
                geometry: semantic_predecessor(track),
                opacity_units: predecessor_opacity_units(track),
                start_velocity: UiPresentationSampleVelocity::RESTING,
                duration_ticks: 1,
                start_tick: None,
            };
        }
    }
    let duration_ticks = declaration.duration_ticks();
    match track.retarget() {
        None => UiPresentationMotionInstallation::Install {
            geometry: semantic_predecessor(track),
            opacity_units: predecessor_opacity_units(track),
            start_velocity: UiPresentationSampleVelocity::RESTING,
            duration_ticks,
            start_tick: None,
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
                    start_tick: None,
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
                start_tick: Some(interrupted.tick),
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
