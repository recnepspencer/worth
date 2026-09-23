use worth_ui::facade::app::{
    UiChangeProfileInstalled, UiIntentWiringSatisfied, WorthUiApplicationBuilder,
};
use worth_ui::facade::declaration::{
    MeasurementConstraint, MeasurementValue, MosaicChildRule, MosaicClippingPosture,
    MosaicFocusScopeKind, MosaicHitTestPosture, MosaicMeasurementAuthority, MosaicOverflowBehavior,
    MosaicParentGrowthBehavior, MosaicPlacementAction, MosaicPlacementConflictBehavior,
    MosaicPlacementPersistence, MosaicPlacementPolicyDescriptor, MosaicPlacementPolicyId,
    MosaicPlacementReloadReconciliation, MosaicPlacementSource, MosaicPlacementTarget,
    MosaicRegionKindDescriptor, MosaicRegionKindId, MosaicRegionPersistence, MosaicRegionRole,
    MosaicResizePermission, MosaicScrollOwnership, MosaicSizingBehavior,
    MosaicSizingContractDescriptor, MosaicSizingContractId, MosaicSizingKind,
    MosaicSizingPersistence, MosaicStableIdentityBehavior, MosaicStateOwnerIdentity,
    MosaicStatePersistencePolicy, MosaicStateReplacementRule, MosaicStateSlotDescriptor,
    MosaicStateSlotId, MosaicStateSlotKind, MosaicStateTruthPosture, MosaicViewportConstraint,
    NamedMeasurementDefinition, NamedMeasurementToken, SurfacePlacementClass, UiScrollAxisSupport,
    UiScrollChromeContract, UiScrollLineExtent,
};
use worth_ui_platform_pulse::product_world::{
    DashboardScrollPanel, PlatformPulseMosaicRegion, PlatformPulseMosaicSizing,
    PLATFORM_PULSE_EVIDENCE_PLACEMENT, PLATFORM_PULSE_FOCUSED_REGION_STATE,
    PLATFORM_PULSE_SERVICE_PLACEMENT, PLATFORM_PULSE_STATUS_PLACEMENT,
};

use super::presentation::scroll_chrome;

pub(super) fn register_mosaic(
    builder: WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied>,
) -> WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied> {
    let builder = PlatformPulseMosaicRegion::ALL
        .into_iter()
        .fold(builder, |builder, region| {
            builder.register_mosaic_region_kind(region_descriptor(region))
        });

    let builder = PlatformPulseMosaicSizing::ALL
        .into_iter()
        .fold(builder, |builder, sizing| {
            builder.register_mosaic_sizing_contract(sizing_descriptor(sizing))
        });
    builder
        .register_mosaic_placement_policy(placement_policy(
            PLATFORM_PULSE_EVIDENCE_PLACEMENT,
            SurfacePlacementClass::auxiliary_region(),
            MosaicRegionRole::side(),
            MosaicPlacementAction::dock(),
        ))
        .register_mosaic_placement_policy(placement_policy(
            PLATFORM_PULSE_SERVICE_PLACEMENT,
            SurfacePlacementClass::primary_region(),
            MosaicRegionRole::primary(),
            MosaicPlacementAction::dock(),
        ))
        .register_mosaic_placement_policy(placement_policy(
            PLATFORM_PULSE_STATUS_PLACEMENT,
            SurfacePlacementClass::status_region(),
            MosaicRegionRole::status(),
            MosaicPlacementAction::status_projection(),
        ))
        .register_mosaic_state_slot(service_focus_state())
}

fn region_descriptor(region: PlatformPulseMosaicRegion) -> MosaicRegionKindDescriptor {
    let (role, sizing, scroll, focus, children, surface_class) = match region {
        PlatformPulseMosaicRegion::Viewport => (
            MosaicRegionRole::viewport(),
            MosaicSizingBehavior::fills_available_space(),
            MosaicScrollOwnership::viewport_owned(),
            MosaicFocusScopeKind::region_scope(),
            MosaicChildRule::accepts_regions(),
            None,
        ),
        PlatformPulseMosaicRegion::Masthead => (
            MosaicRegionRole::toolbar(),
            MosaicSizingBehavior::viewport_bounded(),
            MosaicScrollOwnership::no_scrolling(),
            MosaicFocusScopeKind::toolbar_scope(),
            MosaicChildRule::leaf_only(),
            None,
        ),
        PlatformPulseMosaicRegion::EvidenceRail => (
            MosaicRegionRole::side(),
            MosaicSizingBehavior::viewport_bounded(),
            MosaicScrollOwnership::region_owned(),
            MosaicFocusScopeKind::region_scope(),
            MosaicChildRule::accepts_surfaces(),
            Some(SurfacePlacementClass::auxiliary_region()),
        ),
        PlatformPulseMosaicRegion::ServiceStage => (
            MosaicRegionRole::primary(),
            MosaicSizingBehavior::fills_available_space(),
            MosaicScrollOwnership::region_owned(),
            MosaicFocusScopeKind::active_surface_scope(),
            MosaicChildRule::accepts_surfaces(),
            Some(SurfacePlacementClass::primary_region()),
        ),
        PlatformPulseMosaicRegion::StatusBand => (
            MosaicRegionRole::status(),
            MosaicSizingBehavior::viewport_bounded(),
            MosaicScrollOwnership::no_scrolling(),
            MosaicFocusScopeKind::status_scope(),
            MosaicChildRule::accepts_surfaces(),
            Some(SurfacePlacementClass::status_region()),
        ),
        PlatformPulseMosaicRegion::ServiceList | PlatformPulseMosaicRegion::ActivityList => (
            MosaicRegionRole::auxiliary(),
            MosaicSizingBehavior::viewport_bounded(),
            MosaicScrollOwnership::region_owned(),
            MosaicFocusScopeKind::region_scope(),
            MosaicChildRule::leaf_only(),
            None,
        ),
        PlatformPulseMosaicRegion::ServiceTile | PlatformPulseMosaicRegion::NativeTile => (
            MosaicRegionRole::auxiliary(),
            MosaicSizingBehavior::fills_available_space(),
            MosaicScrollOwnership::no_scrolling(),
            MosaicFocusScopeKind::region_scope(),
            MosaicChildRule::leaf_only(),
            None,
        ),
    };
    let descriptor = MosaicRegionKindDescriptor::new(region_id(region), role)
        .with_sizing_behavior(sizing)
        .with_scroll_ownership(scroll)
        .with_focus_scope(focus)
        .with_child_rule(children)
        .with_persistence(MosaicRegionPersistence::restorable())
        .with_clipping(MosaicClippingPosture::clip_to_region())
        .with_hit_test(MosaicHitTestPosture::pass_through())
        .with_label(region.id());
    let descriptor = match surface_class {
        Some(surface_class) => descriptor.with_allowed_surface_class(surface_class),
        None => descriptor,
    };
    match scrolled_panel(region) {
        Some(panel) => descriptor
            .with_scroll_line_extent(line_extent(panel))
            .with_scroll_chrome(chrome_contract(panel)),
        None => descriptor,
    }
}

/// The dashboard scroll panel this region kind carries, when it carries one.
fn scrolled_panel(region: PlatformPulseMosaicRegion) -> Option<DashboardScrollPanel> {
    DashboardScrollPanel::ALL
        .into_iter()
        .find(|panel| panel.region() == region.id())
}

/// What one wheel line or keyboard step is worth in this region, taken from the
/// panel that authors the content. The number lives in one place only.
fn line_extent(panel: DashboardScrollPanel) -> UiScrollLineExtent {
    UiScrollLineExtent::logical_points(panel.line_extent_logical_points())
        .expect("a Pulse panel declares a positive line extent within the named maximum")
}

/// Track and thumb for the axes this panel really overflows on. Service health
/// is as wide as its region and only travels in block, so declaring inline
/// chrome for it would reserve a gutter for a bar that can never move.
fn chrome_contract(panel: DashboardScrollPanel) -> UiScrollChromeContract {
    let axes = match panel {
        DashboardScrollPanel::ServiceHealth => UiScrollAxisSupport::Block,
        DashboardScrollPanel::RecentActivity => UiScrollAxisSupport::Both,
    };
    UiScrollChromeContract::new(
        axes,
        scroll_chrome::track_role_identity(),
        scroll_chrome::thumb_role_identity(),
    )
    .expect("the Pulse track and thumb are two distinct registered roles")
}

fn sizing_descriptor(sizing: PlatformPulseMosaicSizing) -> MosaicSizingContractDescriptor {
    let kind = match sizing {
        PlatformPulseMosaicSizing::Viewport | PlatformPulseMosaicSizing::ServiceStage => {
            MosaicSizingKind::fill()
        }
        PlatformPulseMosaicSizing::Masthead
        | PlatformPulseMosaicSizing::EvidenceRail
        | PlatformPulseMosaicSizing::DashboardList
        | PlatformPulseMosaicSizing::StatusBand => MosaicSizingKind::fixed(),
    };
    let descriptor = MosaicSizingContractDescriptor::new(sizing_id(sizing), kind)
        .with_measurement_authority(MosaicMeasurementAuthority::runtime_token())
        .with_resize_permission(MosaicResizePermission::fixed_by_runtime())
        .with_persistence(MosaicSizingPersistence::restorable())
        .with_overflow_behavior(MosaicOverflowBehavior::clip())
        .with_parent_growth_behavior(MosaicParentGrowthBehavior::does_not_force_parent())
        .with_viewport_constraint(MosaicViewportConstraint::clamp_to_viewport())
        .with_label(sizing.id());
    match sizing.named_measurement() {
        Some((token, value)) => descriptor.with_named_measurement(NamedMeasurementDefinition::new(
            NamedMeasurementToken::new(token).expect("valid Pulse measurement token"),
            MeasurementValue::logical_pixels(value),
            MeasurementConstraint::between(
                MeasurementValue::logical_pixels(value),
                MeasurementValue::logical_pixels(value),
            ),
        )),
        None => descriptor,
    }
}

fn placement_policy(
    identity: &str,
    source: SurfacePlacementClass,
    target: MosaicRegionRole,
    action: MosaicPlacementAction,
) -> MosaicPlacementPolicyDescriptor {
    MosaicPlacementPolicyDescriptor::new(
        MosaicPlacementPolicyId::new(identity).expect("valid Pulse placement identity"),
        action,
    )
    .with_source(MosaicPlacementSource::surface_class(source))
    .with_target(MosaicPlacementTarget::region_role(target))
    .with_persistence(MosaicPlacementPersistence::restorable())
    .with_stable_identity_behavior(MosaicStableIdentityBehavior::preserve_surface_identity())
    .with_conflict_behavior(MosaicPlacementConflictBehavior::reject_conflict())
    .with_reload_reconciliation(MosaicPlacementReloadReconciliation::restore_when_possible())
}

fn service_focus_state() -> MosaicStateSlotDescriptor {
    MosaicStateSlotDescriptor::new(
        MosaicStateSlotId::new(PLATFORM_PULSE_FOCUSED_REGION_STATE)
            .expect("valid Pulse Mosaic state identity"),
        MosaicStateSlotKind::focused_region(),
    )
    .with_owner_identity(MosaicStateOwnerIdentity::mosaic_region_kind(region_id(
        PlatformPulseMosaicRegion::ServiceStage,
    )))
    .with_persistence_policy(MosaicStatePersistencePolicy::restore_across_hot_reload())
    .with_replacement_rule(MosaicStateReplacementRule::preserve_when_owner_matches())
    .with_truth_posture(MosaicStateTruthPosture::ui_runtime_state())
}

fn region_id(region: PlatformPulseMosaicRegion) -> MosaicRegionKindId {
    MosaicRegionKindId::new(region.id()).expect("valid Pulse Mosaic region identity")
}

fn sizing_id(sizing: PlatformPulseMosaicSizing) -> MosaicSizingContractId {
    MosaicSizingContractId::new(sizing.id()).expect("valid Pulse Mosaic sizing identity")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Only the two panels that really overflow carry chrome. A fixed region
    /// with a declared track would reserve a gutter for a bar that cannot move.
    #[test]
    fn only_the_overflowing_regions_declare_chrome_and_a_line_extent() {
        for region in PlatformPulseMosaicRegion::ALL {
            let descriptor = region_descriptor(region);
            match scrolled_panel(region) {
                Some(panel) => {
                    assert_eq!(
                        descriptor
                            .scroll_line_extent()
                            .map(|extent| extent.logical_points_value()),
                        Some(panel.line_extent_logical_points()),
                    );
                    assert_eq!(
                        descriptor.scroll_chrome().map(UiScrollChromeContract::axes),
                        Some(chrome_contract(panel).axes()),
                    );
                }
                None => {
                    assert!(descriptor.scroll_line_extent().is_none(), "{region:?}");
                    assert!(descriptor.scroll_chrome().is_none(), "{region:?}");
                }
            }
        }
    }

    /// The track and the thumb are two distinct registered chrome roles, named
    /// once by the appearance owner and quoted here.
    #[test]
    fn both_panels_name_the_same_two_distinct_chrome_roles() {
        for panel in DashboardScrollPanel::ALL {
            let contract = chrome_contract(panel);
            assert_eq!(contract.track_role(), scroll_chrome::track_role_identity());
            assert_eq!(contract.thumb_role(), scroll_chrome::thumb_role_identity());
            assert_ne!(contract.track_role(), contract.thumb_role());
        }
    }
}
