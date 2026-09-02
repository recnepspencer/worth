use super::*;

#[test]
fn omitted_motion_is_admitted_as_no_motion() {
    let declaration = UiBackdropDeclarationAuthoring::new(
        "scrim",
        "surface",
        UiAppearanceRoleIdentity::new("scrim.role").unwrap(),
        UiAppearanceRoleRevision::new(1).unwrap(),
    )
    .unwrap()
    .with_scope(UiStaticBackdropScope::SurfaceSingleton)
    .with_extent(UiStaticBackdropExtent::SurfaceViewport("surface".into()))
    .with_presence(UiStaticBackdropPresence::Always)
    .with_placement(UiStaticBackdropPlacement::AboveSurfaceContent)
    .admit()
    .unwrap();

    assert_eq!(declaration.motion(), &UiStaticBackdropMotion::None);
}
