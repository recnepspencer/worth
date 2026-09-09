#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum UiPresentationMotionInstallation {
    Install {
        geometry: Option<[f32; 4]>,
        opacity_units: u16,
        duration_ticks: u32,
    },
    SnapToTarget,
}

pub(super) fn resolve(
    track: crate::runtime::motion::UiCommittedMotionTrack,
    current: Option<(Option<[f32; 4]>, u16)>,
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
            duration_ticks: 1,
        };
    }
    let duration_ticks = declaration.duration_ticks();
    match track.retarget() {
        None => UiPresentationMotionInstallation::Install {
            geometry: semantic_predecessor(track),
            opacity_units: predecessor_opacity_units(track),
            duration_ticks,
        },
        Some(crate::runtime::motion::UiMotionRetargetDisposition::Install {
            predecessor:
                crate::runtime::motion::UiMotionRetargetPredecessor::CurrentPresentationSample,
        }) => {
            let (geometry, opacity_units) = current.unwrap_or_else(|| {
                (
                    semantic_predecessor(track),
                    predecessor_opacity_units(track),
                )
            });
            UiPresentationMotionInstallation::Install {
                geometry,
                opacity_units,
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
