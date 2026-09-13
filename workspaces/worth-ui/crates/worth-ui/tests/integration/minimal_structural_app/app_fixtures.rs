use worth_ui::facade::{
    app::{WorthUi, WorthUiApplicationBuilder},
    declaration::{
        CommandCategory, CommandDescriptor, CommandId, ComponentChildPolicy, ComponentDescriptor,
        ComponentId, ComponentPropSchema, ComponentStateOwnership, IconAccessibilityPosture,
        IconDescriptor, IconFamily, IconId, IconSourceDescriptor, IconThemePosture,
        MeasurementConstraint, MeasurementValue, MosaicChildRule, MosaicClippingPosture,
        MosaicFocusScopeKind, MosaicHitTestPosture, MosaicMeasurementAuthority,
        MosaicOverflowBehavior, MosaicParentGrowthBehavior, MosaicPlacementAction,
        MosaicPlacementConflictBehavior, MosaicPlacementPersistence,
        MosaicPlacementPolicyDescriptor, MosaicPlacementPolicyId,
        MosaicPlacementReloadReconciliation, MosaicPlacementSource, MosaicPlacementTarget,
        MosaicRegionKindDescriptor, MosaicRegionKindId, MosaicRegionPersistence, MosaicRegionRole,
        MosaicResizePermission, MosaicScrollOwnership, MosaicSizingBehavior,
        MosaicSizingContractDescriptor, MosaicSizingContractId, MosaicSizingKind,
        MosaicSizingPersistence, MosaicStableIdentityBehavior, MosaicStateOwnerIdentity,
        MosaicStatePersistencePolicy, MosaicStateReplacementRule, MosaicStateSlotDescriptor,
        MosaicStateSlotId, MosaicStateSlotKind, MosaicStateTruthPosture, MosaicViewportConstraint,
        NamedMeasurementDefinition, NamedMeasurementToken, PluginCapabilityPermission,
        PluginContributionFamily, PluginSlotDescriptor, PluginSlotDiagnostics, PluginSlotId,
        PluginSlotOrdering, PluginSlotSupportPosture, SurfaceDescriptor, SurfaceId, SurfaceKind,
        SurfacePlacementClass, SurfaceStateClass, ThemeTokenDescriptor, ThemeTokenFamily,
        ThemeTokenId, ThemeTokenSource, ThemeTokenValue, UiThemeColor,
    },
};

pub(crate) fn minimal_app_builder() -> WorthUiApplicationBuilder {
    WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_command(minimal_command_descriptor())
        .register_component(minimal_component_descriptor())
        .register_surface(minimal_surface_descriptor())
        .register_mosaic_region_kind(minimal_mosaic_region_descriptor())
        .register_mosaic_placement_policy(minimal_mosaic_placement_policy())
        .register_mosaic_sizing_contract(minimal_mosaic_sizing_contract("minimal.sizing.primary"))
        .register_mosaic_state_slot(minimal_mosaic_state_slot_descriptor())
        .register_theme_token(minimal_theme_token_descriptor())
        .register_icon(minimal_icon_descriptor())
        .register_plugin_slot(minimal_plugin_slot_descriptor())
}

pub(crate) fn minimal_command_descriptor() -> CommandDescriptor {
    CommandDescriptor::new(command_id("minimal.command.save"), "Save")
        .with_category(CommandCategory::Workspace)
}

pub(crate) fn minimal_mosaic_sizing_contract(id: &str) -> MosaicSizingContractDescriptor {
    MosaicSizingContractDescriptor::new(mosaic_sizing_contract_id(id), MosaicSizingKind::bounded())
        .with_measurement_authority(MosaicMeasurementAuthority::runtime_token())
        .with_resize_permission(MosaicResizePermission::user_resizable())
        .with_persistence(MosaicSizingPersistence::restorable())
        .with_overflow_behavior(MosaicOverflowBehavior::scroll_when_constrained())
        .with_parent_growth_behavior(MosaicParentGrowthBehavior::does_not_force_parent())
        .with_viewport_constraint(MosaicViewportConstraint::clamp_to_viewport())
        .with_named_measurement(primary_width_measurement())
}

pub(crate) fn minimal_illegal_mosaic_placement_policy(id: &str) -> MosaicPlacementPolicyDescriptor {
    complete_placement_policy(id, MosaicPlacementAction::dock())
        .with_source(MosaicPlacementSource::surface_class(
            SurfacePlacementClass::primary_region(),
        ))
        .with_target(MosaicPlacementTarget::region_role(
            MosaicRegionRole::toolbar(),
        ))
}

fn minimal_component_descriptor() -> ComponentDescriptor {
    ComponentDescriptor::new(
        component_id("minimal.component.editor"),
        ComponentPropSchema::named("minimal.component.editor.props"),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
}

fn minimal_surface_descriptor() -> SurfaceDescriptor {
    SurfaceDescriptor::new(
        surface_id("minimal.surface.editor"),
        SurfaceKind::primary_content(),
        component_id("minimal.component.editor"),
        SurfacePlacementClass::primary_region(),
        SurfaceStateClass::restorable(),
    )
}

fn minimal_mosaic_region_descriptor() -> MosaicRegionKindDescriptor {
    MosaicRegionKindDescriptor::new(
        mosaic_region_id("minimal.region.primary"),
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

fn minimal_mosaic_placement_policy() -> MosaicPlacementPolicyDescriptor {
    complete_placement_policy("minimal.placement.primary", MosaicPlacementAction::dock())
}

fn complete_placement_policy(
    id: &str,
    action: MosaicPlacementAction,
) -> MosaicPlacementPolicyDescriptor {
    MosaicPlacementPolicyDescriptor::new(mosaic_placement_policy_id(id), action)
        .with_source(MosaicPlacementSource::surface_class(
            SurfacePlacementClass::primary_region(),
        ))
        .with_target(MosaicPlacementTarget::region_role(
            MosaicRegionRole::primary(),
        ))
        .with_persistence(MosaicPlacementPersistence::restorable())
        .with_stable_identity_behavior(MosaicStableIdentityBehavior::preserve_surface_identity())
        .with_conflict_behavior(MosaicPlacementConflictBehavior::reject_conflict())
        .with_reload_reconciliation(MosaicPlacementReloadReconciliation::restore_when_possible())
}

fn minimal_mosaic_state_slot_descriptor() -> MosaicStateSlotDescriptor {
    MosaicStateSlotDescriptor::new(
        mosaic_state_slot_id("minimal.state.splitter"),
        MosaicStateSlotKind::splitter_position(),
    )
    .with_owner_identity(MosaicStateOwnerIdentity::mosaic_region_kind(
        mosaic_region_id("minimal.region.primary"),
    ))
    .with_persistence_policy(MosaicStatePersistencePolicy::restore_across_hot_reload())
    .with_replacement_rule(MosaicStateReplacementRule::preserve_when_owner_matches())
    .with_truth_posture(MosaicStateTruthPosture::ui_runtime_state())
}

fn minimal_theme_token_descriptor() -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::define(
        theme_token_id("minimal.theme.text"),
        ThemeTokenFamily::text(),
        ThemeTokenSource::application(),
        ThemeTokenValue::color(UiThemeColor::parse("#101820").expect("valid color")),
    )
}

fn minimal_icon_descriptor() -> IconDescriptor {
    IconDescriptor::new(
        icon_id("minimal.icon.save"),
        IconFamily::command(),
        IconSourceDescriptor::symbol("minimal.icon.save"),
    )
    .with_accessibility_posture(IconAccessibilityPosture::decorative())
    .with_theme_posture(IconThemePosture::inherits_text_color())
}

fn minimal_plugin_slot_descriptor() -> PluginSlotDescriptor {
    PluginSlotDescriptor::new(plugin_slot_id("minimal.plugin_slot.commands"))
        .allow_family(PluginContributionFamily::command())
        .with_permission(PluginCapabilityPermission::host_granted())
        .with_ordering(PluginSlotOrdering::stable_by_plugin_then_declaration())
        .with_diagnostics(PluginSlotDiagnostics::explain_contributions())
        .with_support(PluginSlotSupportPosture::supported())
}

fn primary_width_measurement() -> NamedMeasurementDefinition {
    NamedMeasurementDefinition::new(
        named_measurement_token("minimal.measurement.primary_width"),
        MeasurementValue::logical_pixels(320),
        MeasurementConstraint::between(
            MeasurementValue::logical_pixels(240),
            MeasurementValue::logical_pixels(520),
        ),
    )
}

fn command_id(raw_text: &str) -> CommandId {
    CommandId::new(raw_text).expect("valid command id")
}

fn component_id(raw_text: &str) -> ComponentId {
    ComponentId::new(raw_text).expect("valid component id")
}

fn surface_id(raw_text: &str) -> SurfaceId {
    SurfaceId::new(raw_text).expect("valid surface id")
}

fn mosaic_region_id(raw_text: &str) -> MosaicRegionKindId {
    MosaicRegionKindId::new(raw_text).expect("valid mosaic region id")
}

fn mosaic_placement_policy_id(raw_text: &str) -> MosaicPlacementPolicyId {
    MosaicPlacementPolicyId::new(raw_text).expect("valid mosaic placement id")
}

fn mosaic_sizing_contract_id(raw_text: &str) -> MosaicSizingContractId {
    MosaicSizingContractId::new(raw_text).expect("valid mosaic sizing id")
}

fn mosaic_state_slot_id(raw_text: &str) -> MosaicStateSlotId {
    MosaicStateSlotId::new(raw_text).expect("valid mosaic state slot id")
}

fn theme_token_id(raw_text: &str) -> ThemeTokenId {
    ThemeTokenId::new(raw_text).expect("valid theme token id")
}

fn icon_id(raw_text: &str) -> IconId {
    IconId::new(raw_text).expect("valid icon id")
}

fn plugin_slot_id(raw_text: &str) -> PluginSlotId {
    PluginSlotId::new(raw_text).expect("valid plugin slot id")
}

fn named_measurement_token(raw_text: &str) -> NamedMeasurementToken {
    NamedMeasurementToken::new(raw_text).expect("valid named measurement token")
}
