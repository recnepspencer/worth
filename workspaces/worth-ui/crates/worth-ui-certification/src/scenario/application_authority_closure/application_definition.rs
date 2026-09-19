use worth_ui::facade::app::WorthUi;
use worth_ui::facade::declaration::{
    CommandDescriptor, CommandId, ComponentCanvasSpatialContract, ComponentChildPolicy,
    ComponentDescriptor, ComponentId, ComponentPropSchema, ComponentRealtimeOverlayContract,
    ComponentRealtimeOverlayPriority, ComponentStateOwnership, MeasurementConstraint,
    MeasurementValue, MosaicChildRule, MosaicClippingPosture, MosaicFocusScopeKind,
    MosaicHitTestPosture, MosaicMeasurementAuthority, MosaicOverflowBehavior,
    MosaicParentGrowthBehavior, MosaicRegionKindDescriptor, MosaicRegionKindId,
    MosaicRegionPersistence, MosaicRegionRole, MosaicResizePermission, MosaicScrollOwnership,
    MosaicSizingBehavior, MosaicSizingContractDescriptor, MosaicSizingContractId, MosaicSizingKind,
    MosaicSizingPersistence, MosaicStateOwnerIdentity, MosaicStatePersistencePolicy,
    MosaicStateReplacementRule, MosaicStateSlotDescriptor, MosaicStateSlotId, MosaicStateSlotKind,
    MosaicStateTruthPosture, MosaicViewportConstraint, NamedMeasurementDefinition,
    NamedMeasurementToken, SurfaceDescriptor, SurfaceId, SurfaceKind, SurfacePlacementClass,
    SurfaceStateClass, ThemeTokenAlias, ThemeTokenDescriptor, ThemeTokenFamily, ThemeTokenId,
    ThemeTokenSource, ThemeTokenValue, UiThemeColor, ViewBindingId,
};
use worth_ui::facade::graph::UiGraphWorldProfile;
use worth_ui::facade::query_binding::WorthUiQueryViewRegistration;
use worth_ui_host_headless::WorthUiHeadlessHost;
use worth_ui_query_binding::certification::WorthUiInstalledQueryTestFixture;
use worth_ui_test_support::WorthUiApplicationBuilderCertificationExt;

use super::fixed_application_builder::FixedCertificationApplicationBuilder;
use super::fixed_host::FixedCertificationHostBinding;

type WorthUiApplicationBuilder = FixedCertificationApplicationBuilder;

pub(crate) const CURRENT_COMPONENT: &str = "workspace.component.authority_current";
pub(crate) const CANDIDATE_COMPONENT: &str = "workspace.component.authority_candidate";
pub(crate) const IMPORTED_CURRENT_COMPONENT: &str =
    "workspace.component.authority_imported_current";
pub(crate) const IMPORTED_CANDIDATE_COMPONENT: &str =
    "workspace.component.authority_imported_candidate";
pub(crate) const REGION: &str = "workspace.region.authority_primary";
pub(crate) const SIZING: &str = "workspace.sizing.authority_primary";
pub(crate) const COMMAND: &str = "workspace.command.authority_save";
pub(crate) const SURFACE: &str = "workspace.surface.authority_main";
pub(crate) const STATE_SLOT: &str = "workspace.state.authority_scroll";
pub(crate) const TOKEN: &str = "theme.text.authority_default";
pub(crate) const QUERY_BINDING: &str = "inspector.measurements";
pub(crate) const CROSS_LANE_CANVAS: &str = "workspace.component.cross_lane_canvas";
pub(crate) const CROSS_LANE_REALTIME: &str = "workspace.component.cross_lane_realtime";
pub(crate) const PREVIEW_COMPONENT: &str = "workspace.component.preview_content";
pub(crate) const PREVIEW_SURFACE: &str = "workspace.surface.preview_splitter";
pub(crate) const PREVIEW_REGION: &str = "workspace.region.preview_split";
pub(crate) const PREVIEW_SIZING: &str = "workspace.sizing.preview_splitter";
pub(crate) const PREVIEW_STATE_SLOT: &str = "workspace.state.preview_splitter_position";
pub(crate) const PREVIEW_SCROLL_STATE_SLOT: &str = "workspace.state.preview_scroll_position";
pub(crate) use super::platform_pulse_application::{
    platform_pulse_application_builder_with_host,
    platform_pulse_application_builder_with_host_and_unrelated_width, unrelated_component_id,
    PLATFORM_PULSE_BACKGROUND_COMPONENT, PLATFORM_PULSE_BLUE_TOKEN, PLATFORM_PULSE_FILL_TOKEN,
    PLATFORM_PULSE_GREEN_TOKEN, PLATFORM_PULSE_IDENTITY_TARGET_COMPONENT,
    PLATFORM_PULSE_IDENTITY_TARGET_FILL_TOKEN, PLATFORM_PULSE_SURFACE, PLATFORM_PULSE_YELLOW_TOKEN,
};

pub(crate) fn application_builder(
    query: &WorthUiInstalledQueryTestFixture,
) -> WorthUiApplicationBuilder {
    application_builder_with_change_profile(
        query,
        worth_ui::facade::rebind::UiChangeProfile::platform_pulse(),
    )
}

pub(crate) fn application_builder_with_change_profile(
    query: &WorthUiInstalledQueryTestFixture,
    profile: worth_ui::facade::rebind::UiChangeProfile,
) -> WorthUiApplicationBuilder {
    application_builder_with_host_and_change_profile(query, WorthUiHeadlessHost, profile)
}

pub(crate) fn application_builder_with_host<Host>(
    query: &WorthUiInstalledQueryTestFixture,
    host: Host,
) -> WorthUiApplicationBuilder
where
    Host: FixedCertificationHostBinding,
{
    application_builder_with_host_and_change_profile(
        query,
        host,
        worth_ui::facade::rebind::UiChangeProfile::platform_pulse(),
    )
}

fn application_builder_with_host_and_change_profile<Host>(
    query: &WorthUiInstalledQueryTestFixture,
    host: Host,
    profile: worth_ui::facade::rebind::UiChangeProfile,
) -> WorthUiApplicationBuilder
where
    Host: FixedCertificationHostBinding,
{
    let builder = WorthUi::app()
        .with_change_profile(profile)
        .with_graph_world_profile(UiGraphWorldProfile::settled_query_binding(
            ViewBindingId::new(QUERY_BINDING).expect("valid Query view binding id"),
            query.binding_reference(),
        ))
        .register_component(component(CURRENT_COMPONENT))
        .register_component(component(CANDIDATE_COMPONENT))
        .register_component(component(IMPORTED_CURRENT_COMPONENT))
        .register_component(component(IMPORTED_CANDIDATE_COMPONENT))
        .register_command(CommandDescriptor::new(
            CommandId::new(COMMAND).expect("valid scenario command id"),
            "Save",
        ))
        .register_surface(
            SurfaceDescriptor::new(
                SurfaceId::new(SURFACE).expect("valid scenario surface id"),
                SurfaceKind::primary_content(),
                ComponentId::new(CURRENT_COMPONENT).expect("valid scenario component id"),
                SurfacePlacementClass::primary_region(),
                SurfaceStateClass::restorable(),
            )
            .with_command_slot(CommandId::new(COMMAND).expect("valid scenario command id")),
        )
        .register_theme_token(ThemeTokenDescriptor::define(
            ThemeTokenId::new("theme.text.authority_primary").expect("valid primary token id"),
            ThemeTokenFamily::text(),
            ThemeTokenSource::application(),
            ThemeTokenValue::color(UiThemeColor::parse("#101820").expect("valid theme color")),
        ))
        .register_theme_token(ThemeTokenDescriptor::alias(
            ThemeTokenId::new(TOKEN).expect("valid scenario token id"),
            ThemeTokenFamily::text(),
            ThemeTokenSource::application(),
            ThemeTokenAlias::to(
                ThemeTokenId::new("theme.text.authority_primary").expect("valid primary token id"),
            ),
        ))
        .register_mosaic_region_kind(region())
        .register_mosaic_sizing_contract(sizing())
        .register_mosaic_state_slot(state_slot())
        .register_query_view(WorthUiQueryViewRegistration::new(query.installed_view()))
        .expect("installed Query view should register through the production builder");
    FixedCertificationApplicationBuilder::new(builder, host)
}

pub(crate) fn cross_lane_application_builder_with_host<Host>(
    query: &WorthUiInstalledQueryTestFixture,
    host: Host,
) -> WorthUiApplicationBuilder
where
    Host: FixedCertificationHostBinding,
{
    application_builder_with_host(query, host)
        .register_component(
            component(CROSS_LANE_CANVAS).with_canvas_spatial_contract(
                ComponentCanvasSpatialContract::new(64, 2, 1)
                    .expect("cross-lane spatial contract is bounded"),
            ),
        )
        .register_component(
            component(CROSS_LANE_REALTIME).with_realtime_overlay_contract(
                ComponentRealtimeOverlayContract::new(
                    2,
                    1,
                    16,
                    ComponentRealtimeOverlayPriority::HudOverlay,
                )
                .expect("cross-lane realtime contract fits its frame budget"),
            ),
        )
}

pub(crate) fn preview_application_builder_with_host<Host>(
    query: &WorthUiInstalledQueryTestFixture,
    host: Host,
) -> WorthUiApplicationBuilder
where
    Host: FixedCertificationHostBinding,
{
    register_preview_contracts(application_builder_with_host(query, host))
}

pub(crate) fn preview_cross_lane_application_builder_with_host<Host>(
    query: &WorthUiInstalledQueryTestFixture,
    host: Host,
) -> WorthUiApplicationBuilder
where
    Host: FixedCertificationHostBinding,
{
    register_preview_contracts(cross_lane_application_builder_with_host(query, host))
}

fn register_preview_contracts(builder: WorthUiApplicationBuilder) -> WorthUiApplicationBuilder {
    builder
        .register_component(component(PREVIEW_COMPONENT))
        .register_surface(SurfaceDescriptor::new(
            SurfaceId::new(PREVIEW_SURFACE).expect("valid preview surface id"),
            SurfaceKind::primary_content(),
            ComponentId::new(PREVIEW_COMPONENT).expect("valid preview component id"),
            SurfacePlacementClass::primary_region(),
            SurfaceStateClass::restorable(),
        ))
        .register_mosaic_region_kind(preview_region())
        .register_mosaic_sizing_contract(preview_sizing())
        .register_mosaic_state_slot(preview_state_slot())
        .register_mosaic_state_slot(preview_scroll_state_slot())
}

pub(crate) fn scaled_canvas_application_builder_with_host<Host>(
    query: &WorthUiInstalledQueryTestFixture,
    host: Host,
    canvas_count: usize,
) -> WorthUiApplicationBuilder
where
    Host: FixedCertificationHostBinding,
{
    let mut builder = application_builder_with_host(query, host);
    for index in 0..canvas_count {
        let identity = format!("workspace.component.scaled_canvas_{index:04}");
        builder = builder.register_component(
            component(&identity).with_canvas_spatial_contract(
                ComponentCanvasSpatialContract::new(64, 2, 1)
                    .expect("scaled spatial contract is bounded"),
            ),
        );
    }
    builder
}

pub(super) fn application_builder_with_capability_drift(
    query: &WorthUiInstalledQueryTestFixture,
) -> WorthUiApplicationBuilder {
    application_builder(query)
        .register_component(component("workspace.component.authority_capability_drift"))
}

fn component(id: &str) -> ComponentDescriptor {
    ComponentDescriptor::new(
        ComponentId::new(id).expect("valid scenario component id"),
        ComponentPropSchema::named(format!("{id}.props")),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
}

fn region() -> MosaicRegionKindDescriptor {
    MosaicRegionKindDescriptor::new(
        MosaicRegionKindId::new(REGION).expect("valid scenario region id"),
        MosaicRegionRole::primary(),
    )
    .with_sizing_behavior(MosaicSizingBehavior::fills_available_space())
    .with_scroll_ownership(MosaicScrollOwnership::region_owned())
    .with_focus_scope(MosaicFocusScopeKind::active_surface_scope())
    .with_child_rule(MosaicChildRule::accepts_surfaces())
    .with_allowed_surface_class(SurfacePlacementClass::primary_region())
    .with_persistence(MosaicRegionPersistence::restorable())
    .with_clipping(MosaicClippingPosture::clip_to_region())
    .with_hit_test(MosaicHitTestPosture::participates())
}

fn sizing() -> MosaicSizingContractDescriptor {
    MosaicSizingContractDescriptor::new(
        MosaicSizingContractId::new(SIZING).expect("valid scenario sizing id"),
        MosaicSizingKind::fill(),
    )
    .with_measurement_authority(MosaicMeasurementAuthority::runtime_token())
    .with_resize_permission(MosaicResizePermission::user_resizable())
    .with_persistence(MosaicSizingPersistence::restorable())
    .with_overflow_behavior(MosaicOverflowBehavior::scroll_when_constrained())
    .with_parent_growth_behavior(MosaicParentGrowthBehavior::does_not_force_parent())
    .with_viewport_constraint(MosaicViewportConstraint::clamp_to_viewport())
    .with_named_measurement(NamedMeasurementDefinition::new(
        NamedMeasurementToken::new("workspace.measurement.authority_primary")
            .expect("valid scenario measurement token"),
        MeasurementValue::logical_pixels(320),
        MeasurementConstraint::between(
            MeasurementValue::logical_pixels(200),
            MeasurementValue::logical_pixels(640),
        ),
    ))
}

fn state_slot() -> MosaicStateSlotDescriptor {
    MosaicStateSlotDescriptor::new(
        MosaicStateSlotId::new(STATE_SLOT).expect("valid scenario state slot id"),
        MosaicStateSlotKind::scroll_position(),
    )
    .with_owner_identity(MosaicStateOwnerIdentity::mosaic_region_kind(
        MosaicRegionKindId::new(REGION).expect("valid scenario region id"),
    ))
    .with_persistence_policy(MosaicStatePersistencePolicy::restore_across_hot_reload())
    .with_replacement_rule(MosaicStateReplacementRule::preserve_when_owner_matches())
    .with_truth_posture(MosaicStateTruthPosture::ui_runtime_state())
}

fn preview_region() -> MosaicRegionKindDescriptor {
    MosaicRegionKindDescriptor::new(
        MosaicRegionKindId::new(PREVIEW_REGION).expect("valid preview region id"),
        MosaicRegionRole::split(),
    )
    .with_sizing_behavior(MosaicSizingBehavior::fills_available_space())
    .with_scroll_ownership(MosaicScrollOwnership::region_owned())
    .with_focus_scope(MosaicFocusScopeKind::active_surface_scope())
    .with_child_rule(MosaicChildRule::accepts_surfaces())
    .with_allowed_surface_class(SurfacePlacementClass::primary_region())
    .with_persistence(MosaicRegionPersistence::restorable())
    .with_clipping(MosaicClippingPosture::clip_to_region())
    .with_hit_test(MosaicHitTestPosture::participates())
}

fn preview_sizing() -> MosaicSizingContractDescriptor {
    MosaicSizingContractDescriptor::new(
        MosaicSizingContractId::new(PREVIEW_SIZING).expect("valid preview sizing id"),
        MosaicSizingKind::fill(),
    )
    .with_measurement_authority(MosaicMeasurementAuthority::runtime_token())
    .with_resize_permission(MosaicResizePermission::user_resizable())
    .with_persistence(MosaicSizingPersistence::restorable())
    .with_overflow_behavior(MosaicOverflowBehavior::scroll_when_constrained())
    .with_parent_growth_behavior(MosaicParentGrowthBehavior::does_not_force_parent())
    .with_viewport_constraint(MosaicViewportConstraint::clamp_to_viewport())
    .with_named_measurement(NamedMeasurementDefinition::new(
        NamedMeasurementToken::new("workspace.measurement.preview_splitter")
            .expect("valid preview measurement token"),
        MeasurementValue::logical_pixels(320),
        MeasurementConstraint::between(
            MeasurementValue::logical_pixels(200),
            MeasurementValue::logical_pixels(640),
        ),
    ))
}

fn preview_state_slot() -> MosaicStateSlotDescriptor {
    MosaicStateSlotDescriptor::new(
        MosaicStateSlotId::new(PREVIEW_STATE_SLOT).expect("valid preview state slot id"),
        MosaicStateSlotKind::splitter_position(),
    )
    .with_owner_identity(MosaicStateOwnerIdentity::mosaic_region_kind(
        MosaicRegionKindId::new(PREVIEW_REGION).expect("valid preview region id"),
    ))
    .with_persistence_policy(MosaicStatePersistencePolicy::restore_across_hot_reload())
    .with_replacement_rule(MosaicStateReplacementRule::preserve_when_owner_matches())
    .with_truth_posture(MosaicStateTruthPosture::ui_runtime_state())
}

fn preview_scroll_state_slot() -> MosaicStateSlotDescriptor {
    MosaicStateSlotDescriptor::new(
        MosaicStateSlotId::new(PREVIEW_SCROLL_STATE_SLOT)
            .expect("valid preview scroll state slot id"),
        MosaicStateSlotKind::scroll_position(),
    )
    .with_owner_identity(MosaicStateOwnerIdentity::mosaic_region_kind(
        MosaicRegionKindId::new(PREVIEW_REGION).expect("valid preview region id"),
    ))
    .with_persistence_policy(MosaicStatePersistencePolicy::restore_across_hot_reload())
    .with_replacement_rule(MosaicStateReplacementRule::preserve_when_owner_matches())
    .with_truth_posture(MosaicStateTruthPosture::ui_runtime_state())
}
