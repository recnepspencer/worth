use super::*;

fn owner(plan_region_index: u32) -> crate::runtime::scroll::UiScrollOwnerIdentity {
    crate::runtime::scroll::UiScrollOwnerIdentity::declared_region(
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().expect("surface"),
        crate::graph::UiGraphNodeIdentity::new(3_161),
        1,
        plan_region_index,
    )
}

fn latch_for(
    owner: crate::runtime::scroll::UiScrollOwnerIdentity,
    axis: UiScrollChromeAxis,
    pointer: u64,
    epoch: u64,
) -> UiScrollChromeLatch {
    UiScrollChromeLatch::press(
        UiHostPointerIdentity::new(pointer),
        UiHostPointerCaptureEpoch::new(epoch),
        UiSurfaceBindingGeneration::mint_unbound().expect("binding"),
        owner,
        worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().expect("instance"),
        crate::runtime::scroll::UiScrollOwnerIncarnation::new(1).expect("incarnation"),
        axis,
        4.0,
    )
}

/// A drag owns its axis for the length of its capture, so a wheel on that
/// axis of that region is ignored while it runs. The other axis, and every
/// other region, keep scrolling.
#[test]
fn a_captured_axis_suppresses_only_its_own_axis_of_its_own_region() {
    let dragged = owner(4);
    let other = owner(5);
    let mut state = UiScrollChromeLatchState::default();
    state
        .latch(latch_for(dragged, UiScrollChromeAxis::Block, 1, 1))
        .expect("the first thumb press latches");

    assert!(state.suppresses_axis(dragged, UiScrollChromeAxis::Block));
    assert!(!state.suppresses_axis(dragged, UiScrollChromeAxis::Inline));
    assert!(!state.suppresses_axis(other, UiScrollChromeAxis::Block));
}

/// Nothing is suppressed when nothing is captured, so a released drag stops
/// silencing the wheel it was silencing.
#[test]
fn a_released_drag_suppresses_nothing() {
    let dragged = owner(4);
    let mut state = UiScrollChromeLatchState::default();
    state
        .latch(latch_for(dragged, UiScrollChromeAxis::Block, 1, 1))
        .expect("latched");
    state
        .release(
            UiHostPointerIdentity::new(1),
            UiHostPointerCaptureEpoch::new(1),
        )
        .expect("the latching pointer releases it");
    assert!(!state.suppresses_axis(dragged, UiScrollChromeAxis::Block));
}

/// Leaving the gutter changes the posture and keeps the capture: a drag
/// that wandered off the track is still the drag that owns the thumb.
#[test]
fn dragging_outside_the_gutter_keeps_the_capture() {
    let mut state = UiScrollChromeLatchState::default();
    state
        .latch(latch_for(owner(4), UiScrollChromeAxis::Block, 1, 1))
        .expect("latched");
    let moved = state
        .moved(
            UiHostPointerIdentity::new(1),
            UiHostPointerCaptureEpoch::new(1),
            UiScrollChromeDragPosture::OutsideGutter,
        )
        .expect("a move under the same capture is admitted");
    assert_eq!(moved.posture(), UiScrollChromeDragPosture::OutsideGutter);
    assert_eq!(moved.grab_offset_logical_points(), 4.0);
    assert!(state.held().is_some());
}

/// Two pointers never move one thumb, and a report from a pointer that does
/// not hold the latch is refused rather than redirected.
#[test]
fn only_the_latching_pointer_under_its_own_capture_moves_the_thumb() {
    let mut state = UiScrollChromeLatchState::default();
    state
        .latch(latch_for(owner(4), UiScrollChromeAxis::Block, 1, 1))
        .expect("latched");
    assert_eq!(
        state.latch(latch_for(owner(5), UiScrollChromeAxis::Inline, 2, 1)),
        Err(UiScrollChromeLatchDenial::AlreadyLatched)
    );
    assert_eq!(
        state.moved(
            UiHostPointerIdentity::new(2),
            UiHostPointerCaptureEpoch::new(1),
            UiScrollChromeDragPosture::InsideGutter,
        ),
        Err(UiScrollChromeLatchDenial::PointerMismatch)
    );
    assert_eq!(
        state.moved(
            UiHostPointerIdentity::new(1),
            UiHostPointerCaptureEpoch::new(2),
            UiScrollChromeDragPosture::InsideGutter,
        ),
        Err(UiScrollChromeLatchDenial::CaptureEpochChanged)
    );
}

/// A modality change ends a captured thumb exactly as it ends a press.
#[test]
fn a_cancellation_ends_the_drag() {
    let dragged = owner(4);
    let mut state = UiScrollChromeLatchState::default();
    state
        .latch(latch_for(dragged, UiScrollChromeAxis::Block, 1, 1))
        .expect("latched");
    assert!(state.cancel().is_some());
    assert!(state.held().is_none());
    assert!(!state.suppresses_axis(dragged, UiScrollChromeAxis::Block));
    assert_eq!(
        state.release(
            UiHostPointerIdentity::new(1),
            UiHostPointerCaptureEpoch::new(1)
        ),
        Err(UiScrollChromeLatchDenial::NotLatched)
    );
}
