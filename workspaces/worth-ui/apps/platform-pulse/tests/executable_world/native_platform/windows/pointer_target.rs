use winsafe::{co, HWND, POINT};

use crate::external_observation::NativeInputProbeKind;

use super::NativePlatformFailure;

pub(super) fn require_before_effect(
    window: &HWND,
    screen_point: (i32, i32),
) -> Result<(), NativePlatformFailure> {
    classify_pointer_target(
        window.ptr() as usize,
        point_root_window(screen_point),
        PointerTargetCheckPhase::BeforeEffect,
    )
}

pub(super) fn require_after_effect(
    window: &HWND,
    screen_point: (i32, i32),
    kind: NativeInputProbeKind,
    delivered_event_count: u32,
) -> Result<(), NativePlatformFailure> {
    classify_pointer_target(
        window.ptr() as usize,
        point_root_window(screen_point),
        PointerTargetCheckPhase::AfterEffect {
            kind,
            delivered_event_count,
        },
    )
}

fn point_root_window(screen_point: (i32, i32)) -> usize {
    HWND::WindowFromPoint(POINT {
        x: screen_point.0,
        y: screen_point.1,
    })
    .and_then(|child| child.GetAncestor(co::GA::ROOT))
    .map_or(0, |handle| handle.ptr() as usize)
}

#[derive(Clone, Copy)]
pub(super) enum PointerTargetCheckPhase {
    BeforeEffect,
    AfterEffect {
        kind: NativeInputProbeKind,
        delivered_event_count: u32,
    },
}

pub(super) fn classify_pointer_target(
    expected_window: usize,
    observed_window: usize,
    phase: PointerTargetCheckPhase,
) -> Result<(), NativePlatformFailure> {
    if observed_window == expected_window {
        return Ok(());
    }
    match phase {
        PointerTargetCheckPhase::BeforeEffect => Err(NativePlatformFailure::InputEnvironment(
            super::input_environment::WindowsInputEnvironmentDenial::PointerTargetMismatch {
                target_window: expected_window,
                hit_window: observed_window,
            },
        )),
        PointerTargetCheckPhase::AfterEffect {
            kind,
            delivered_event_count,
        } => Err(super::input_delivery::post_effect_failure(
            kind,
            delivered_event_count,
            format!(
                "pointer hit window {observed_window:#x} instead of bound window {expected_window:#x}"
            ),
        )),
    }
}
