//! Which top-level window the pointer is over. Without a window manager the
//! root's direct child under the pointer is the client's top-level itself,
//! so `QueryPointer(root).child` is the hit test; no ancestor walk needed.
use x11rb::protocol::xproto::{ConnectionExt as _, Window};

use crate::external_observation::NativeInputProbeKind;
use crate::native_platform::NativePlatformFailure;

use super::connection::{input_failure, X11Observation};
use super::input_environment::LinuxX11InputEnvironmentDenial;

pub(super) fn require_before_effect(
    x11: &X11Observation,
    window: Window,
) -> Result<(), NativePlatformFailure> {
    classify_pointer_target(
        window,
        window_under_pointer(x11)?,
        PointerTargetCheckPhase::BeforeEffect,
    )
}

pub(super) fn require_after_effect(
    x11: &X11Observation,
    window: Window,
    kind: NativeInputProbeKind,
    delivered_event_count: u32,
) -> Result<(), NativePlatformFailure> {
    let hit = window_under_pointer(x11).map_err(|failure| {
        super::input_delivery::post_effect_failure(kind, delivered_event_count, failure.to_string())
    })?;
    classify_pointer_target(
        window,
        hit,
        PointerTargetCheckPhase::AfterEffect {
            kind,
            delivered_event_count,
        },
    )
}

fn window_under_pointer(x11: &X11Observation) -> Result<Window, NativePlatformFailure> {
    x11.connection()
        .query_pointer(x11.root())
        .map_err(input_failure)?
        .reply()
        .map(|reply| reply.child)
        .map_err(input_failure)
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
    expected_window: Window,
    observed_window: Window,
    phase: PointerTargetCheckPhase,
) -> Result<(), NativePlatformFailure> {
    if observed_window == expected_window {
        return Ok(());
    }
    match phase {
        PointerTargetCheckPhase::BeforeEffect => Err(NativePlatformFailure::InputEnvironment(
            LinuxX11InputEnvironmentDenial::PointerTargetMismatch {
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

#[test]
fn hostile_pointer_target_is_classified_by_effect_boundary() {
    assert!(matches!(
        classify_pointer_target(0x11, 0x22, PointerTargetCheckPhase::BeforeEffect),
        Err(NativePlatformFailure::InputEnvironment(_))
    ));
    assert!(matches!(
        classify_pointer_target(
            0x11,
            0x22,
            PointerTargetCheckPhase::AfterEffect {
                kind: NativeInputProbeKind::Pointer,
                delivered_event_count: 2,
            },
        ),
        Err(NativePlatformFailure::InputDeliveryIndeterminate {
            kind: NativeInputProbeKind::Pointer,
            delivered_event_count: 2,
            ..
        })
    ));
}
