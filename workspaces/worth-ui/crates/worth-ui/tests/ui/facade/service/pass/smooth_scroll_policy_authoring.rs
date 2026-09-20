//! A product author declares a smooth wheel, a line extent and scroll chrome
//! through the facade alone, without reaching into any runtime module.

use worth_ui::facade::appearance::UiAppearanceRoleIdentity;
use worth_ui::facade::declaration::{
    MosaicRegionKindDescriptor, MosaicRegionKindId, MosaicRegionRole, MosaicScrollOwnership,
    UiScrollAxisSupport, UiScrollChromeContract, UiScrollLineExtent,
};
use worth_ui::facade::service::{
    UiScrollAnchorBehavior, UiScrollPolicy, UiScrollWheelBehavior,
    UI_SCROLL_WHEEL_SETTLE_TICK_CEILING,
};

fn role(text: &str) -> UiAppearanceRoleIdentity {
    UiAppearanceRoleIdentity::new(text).expect("the facade accepts a valid role identity")
}

fn chrome() -> UiScrollChromeContract {
    UiScrollChromeContract::new(
        UiScrollAxisSupport::Block,
        role("product.scroll_track"),
        role("product.scroll_thumb"),
    )
    .expect("a track and a thumb with distinct identities are admitted")
}

fn region() -> MosaicRegionKindDescriptor {
    MosaicRegionKindDescriptor::new(
        MosaicRegionKindId::new("product.activity_list")
            .expect("the facade accepts a valid region kind id"),
        MosaicRegionRole::Primary,
    )
    .with_scroll_ownership(MosaicScrollOwnership::RegionOwned)
    .with_scroll_line_extent(
        UiScrollLineExtent::logical_points(20).expect("twenty logical points is a lawful line"),
    )
    .with_scroll_chrome(chrome())
}

fn policy() -> UiScrollPolicy {
    UiScrollPolicy::nested_region()
        .with_anchor_behavior(UiScrollAnchorBehavior::RebaseStableAnchor)
        .with_wheel_behavior(
            UiScrollWheelBehavior::smooth(120).expect("120 ticks is an admitted settle horizon"),
        )
}

fn main() {
    let region = region();
    let policy = policy();

    assert_eq!(
        region
            .scroll_line_extent()
            .map(UiScrollLineExtent::logical_points_value),
        Some(20)
    );
    assert_eq!(
        region.scroll_chrome().map(UiScrollChromeContract::axes),
        Some(UiScrollAxisSupport::Block)
    );
    assert_eq!(
        policy.wheel_behavior().settle_ticks(),
        Some(120),
        "the authored settle horizon survives the policy"
    );
    assert!(120 <= UI_SCROLL_WHEEL_SETTLE_TICK_CEILING);
}
