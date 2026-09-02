use super::backdrop_pipeline::UiNativeBackdropPipeline;
use super::command::UiNativeAppearanceCommand;
use super::geometry::UiNativeAppearanceScale;
use super::retained::UiNativeAppearanceRetained;
use super::tests::{backdrop, pointer, text_foreground};
use worth_ui_host_contract::UiPointerAffordanceFamily;

#[test]
fn cursor_and_text_rows_are_retained_as_distinct_non_surface_families() {
    let scale = UiNativeAppearanceScale::qualified(1_000).unwrap();
    let mut retained = UiNativeAppearanceRetained::new(scale);
    let text_key = retained
        .insert(
            UiNativeAppearanceCommand::TextForeground(text_foreground(7)),
            None,
        )
        .unwrap();
    let pointer_key = retained
        .insert(
            UiNativeAppearanceCommand::PointerAffordance(pointer(
                UiPointerAffordanceFamily::Activation,
            )),
            Some(text_key),
        )
        .unwrap();
    assert_eq!(retained.ordered_keys().as_ref(), &[text_key, pointer_key]);
    assert!(retained
        .command(pointer_key)
        .unwrap()
        .damage_rect(scale)
        .unwrap()
        .is_none());
}

#[test]
fn backdrop_pipeline_preserves_identity_extent_clip_color_and_opacity() {
    let mechanic = backdrop(4);
    let primitive = UiNativeBackdropPipeline::prepare(
        &mechanic,
        UiNativeAppearanceScale::qualified(1_250).unwrap(),
    )
    .unwrap();
    assert_eq!(primitive.identity(), mechanic.identity());
    assert_eq!(primitive.ordinal(), 4);
    assert_eq!(primitive.background().straight_srgba(), [0, 0, 0, 128]);
    assert_eq!(primitive.opacity(), u16::MAX);
    assert!(primitive.paints_in_order(0, 0));
    assert_eq!(primitive.damage_rect().left, 0);
}

#[test]
fn pointer_cursor_port_only_accepts_the_sealed_family() {
    let default = pointer(UiPointerAffordanceFamily::Default);
    let activation = pointer(UiPointerAffordanceFamily::Activation);
    assert_eq!(
        super::cursor::cursor_icon(default),
        winit::window::CursorIcon::Default
    );
    assert_eq!(
        super::cursor::cursor_icon(activation),
        winit::window::CursorIcon::Pointer
    );
    assert_eq!(super::geometry::PHYSICAL_MICROS_PER_PIXEL, 1_000_000);
}
