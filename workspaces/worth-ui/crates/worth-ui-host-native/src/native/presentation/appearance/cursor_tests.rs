use super::cursor;
use super::mounted_mechanic_fixtures::pointer;
use worth_ui_host_contract::UiPointerAffordanceFamily;

#[test]
fn completed_pointer_arrival_departure_and_unrelated_work_have_distinct_effects() {
    use worth_ui_host_contract::{
        UiMountedAppearanceMechanic, UiUnpublishedAppearanceFragmentIdentity,
    };
    let activation = pointer(UiPointerAffordanceFamily::Activation);
    let identity = UiUnpublishedAppearanceFragmentIdentity::SurfacePointer {
        surface: activation.surface(),
        pointer: activation.pointer(),
    };
    assert_eq!(
        cursor::completed_fragment_cursor(
            identity,
            &[UiMountedAppearanceMechanic::Pointer(activation)]
        ),
        Some(winit::window::CursorIcon::Pointer)
    );
    assert_eq!(
        cursor::completed_fragment_cursor(identity, &[]),
        Some(winit::window::CursorIcon::Default)
    );
    assert_eq!(
        cursor::completed_fragment_cursor(
            UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(activation.surface()),
            &[]
        ),
        None
    );
}

#[test]
fn pointer_cursor_port_only_accepts_the_sealed_family() {
    let default = pointer(UiPointerAffordanceFamily::Default);
    let activation = pointer(UiPointerAffordanceFamily::Activation);
    assert_eq!(
        cursor::cursor_icon(default),
        winit::window::CursorIcon::Default
    );
    assert_eq!(
        cursor::cursor_icon(activation),
        winit::window::CursorIcon::Pointer
    );
    assert_eq!(super::geometry::PHYSICAL_MICROS_PER_PIXEL, 1_000_000);
}
