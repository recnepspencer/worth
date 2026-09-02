use super::cursor;
use super::mounted_mechanic_fixtures::pointer;
use worth_ui_host_contract::UiPointerAffordanceFamily;

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
