use worth_ui::facade::appearance::{
    UiAppearanceAspect, UiAppearanceCell, UiAppearancePartitionAuthoring, UiAppearanceRole,
    UiAppearanceRoleIdentity, UiDslComponentReference, UiThemeSlotIdentity, UiThemeValueKind,
};
use worth_ui::facade::declaration::{
    MosaicRegionKindDescriptor, MosaicRegionKindId, MosaicRegionRole, MosaicScrollOwnership,
    UiScrollAxisSupport, UiScrollChromeContract, UiScrollLineExtent,
};

const GUIDE: &str = include_str!("../../../../docs/appearance-and-themes.md");

#[test]
fn documented_rust_role_uses_the_public_facade() {
    assert!(GUIDE.contains("compiled-example:appearance-role-rust"));
    let role =
        UiAppearanceRole::authoring(UiAppearanceRoleIdentity::new("app.appearance.card").unwrap())
            .applies_to_component(UiDslComponentReference::new("app.component.card").unwrap())
            .cover(
                UiAppearanceAspect::Background,
                UiAppearancePartitionAuthoring::new([]).with_cell(
                    UiAppearanceCell::when([]).uses_slot(
                        UiThemeSlotIdentity::new("app.theme.surface").unwrap(),
                        UiThemeValueKind::Color,
                    ),
                ),
            )
            .unwrap()
            .build()
            .unwrap();
    let _ = role;
}

/// The guide's scroll chrome example. An author declares two roles and names
/// them on the region's contract; nothing here supplies a rectangle, because
/// the runtime derives the track and thumb from the accepted pose.
#[test]
fn documented_scroll_chrome_declares_roles_and_names_them_on_the_region() {
    assert!(GUIDE.contains("compiled-example:scroll-chrome-rust"));
    let track_identity = UiAppearanceRoleIdentity::new("app.appearance.scroll_track").unwrap();
    let thumb_identity = UiAppearanceRoleIdentity::new("app.appearance.scroll_thumb").unwrap();
    let thumb = UiAppearanceRole::authoring(thumb_identity.clone())
        .cover(
            UiAppearanceAspect::Background,
            UiAppearancePartitionAuthoring::new([]).with_cell(
                UiAppearanceCell::when([]).uses_slot(
                    UiThemeSlotIdentity::new("app.theme.scroll_thumb").unwrap(),
                    UiThemeValueKind::Color,
                ),
            ),
        )
        .unwrap()
        .build()
        .unwrap();
    let region = MosaicRegionKindDescriptor::new(
        MosaicRegionKindId::new("app.region.list").unwrap(),
        MosaicRegionRole::auxiliary(),
    )
    .with_scroll_ownership(MosaicScrollOwnership::region_owned())
    .with_scroll_line_extent(UiScrollLineExtent::logical_points(20).unwrap())
    .with_scroll_chrome(
        UiScrollChromeContract::new(UiScrollAxisSupport::Block, track_identity, thumb_identity)
            .unwrap(),
    );
    let _ = (thumb, region);
}
