use worth_ui::facade::appearance::{
    UiAppearanceAspect, UiAppearanceCell, UiAppearancePartitionAuthoring, UiAppearanceRole,
    UiAppearanceRoleIdentity, UiDslComponentReference, UiThemeSlotIdentity, UiThemeValueKind,
};
use worth_ui::facade::declaration::{
    ComponentChildPolicy, ComponentDescriptor, ComponentId, ComponentPropSchema,
    ComponentSemanticTextContract, ComponentSemanticTextFlow, ComponentStateOwnership,
    MosaicLayoutCell, MosaicLayoutContract, MosaicRegionKindDescriptor, MosaicRegionKindId,
    MosaicRegionRole, MosaicResponsiveLayout, MosaicScrollOwnership, MosaicTrack,
    MosaicViewportWidthInterval, ThemeTokenId, UiScrollAxisSupport, UiScrollChromeContract,
    UiScrollLineExtent,
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

/// The guide's container layout example: two weighted columns from 1200
/// points wide, one stacked column below, with the same members in both.
#[test]
fn documented_container_layout_selects_tracks_by_width() {
    assert!(GUIDE.contains("compiled-example:container-layout-rust"));
    let title = ComponentId::new("app.component.title").unwrap();
    let chart = ComponentId::new("app.component.chart").unwrap();
    let health = ComponentId::new("app.component.health").unwrap();
    let wide = MosaicLayoutContract::grid(
        [
            MosaicTrack::flex(2, 480).unwrap(),
            MosaicTrack::flex(1, 320).unwrap(),
        ],
        [
            MosaicTrack::fixed(66).unwrap(),
            MosaicTrack::flex(1, 348).unwrap(),
        ],
    )
    .unwrap()
    .with_gaps(20, 20)
    .with_padding(24, 24)
    .with_member(
        title.clone(),
        MosaicLayoutCell::spanning(0, 0, 2, 1).unwrap(),
    )
    .unwrap()
    .with_member(chart.clone(), MosaicLayoutCell::at(0, 1))
    .unwrap()
    .with_member(health.clone(), MosaicLayoutCell::at(1, 1))
    .unwrap();
    let stacked = MosaicLayoutContract::rows([
        MosaicTrack::fixed(66).unwrap(),
        MosaicTrack::flex(1, 348).unwrap(),
        MosaicTrack::flex(1, 348).unwrap(),
    ])
    .unwrap()
    .with_gaps(20, 20)
    .with_padding(24, 24)
    .with_member(title, MosaicLayoutCell::at(0, 0))
    .unwrap()
    .with_member(chart, MosaicLayoutCell::at(0, 1))
    .unwrap()
    .with_member(health, MosaicLayoutCell::at(0, 2))
    .unwrap();
    let layout = MosaicResponsiveLayout::new(stacked)
        .with_variant(MosaicViewportWidthInterval::at_least(1200), wide)
        .unwrap();
    assert_eq!(layout.select(1200.0).column_tracks().len(), 2);
    assert_eq!(layout.select(1199.5).column_tracks().len(), 1);
    let page = ComponentDescriptor::new(
        ComponentId::new("app.component.page").unwrap(),
        ComponentPropSchema::named("app.component.page.props"),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
    .with_layout(layout);
    let caption = ComponentSemanticTextContract::body_default(
        ThemeTokenId::new("app.theme.text").unwrap(),
        1,
    )
    .with_flow(ComponentSemanticTextFlow::single_line_ellipsis());
    let _ = (page, caption);
}
