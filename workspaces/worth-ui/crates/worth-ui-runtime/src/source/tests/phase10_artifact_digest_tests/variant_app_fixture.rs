use crate::capability::{
    CommandDescriptor, CommandId, CommandProjectionCommandReference, CommandProjectionDescriptor,
    CommandProjectionId, CommandProjectionSurface, ComponentChildPolicy, ComponentDescriptor,
    ComponentExecutionLane, ComponentId, ComponentPropSchema, ComponentStateOwnership,
    IconDescriptor, IconFamily, IconId, IconSourceDescriptor, MeasurementConstraint,
    MeasurementValue, MosaicChildRule, MosaicClippingPosture, MosaicFocusScopeKind,
    MosaicHitTestPosture, MosaicMeasurementAuthority, MosaicOverflowBehavior,
    MosaicParentGrowthBehavior, MosaicPlacementAction, MosaicPlacementConflictBehavior,
    MosaicPlacementPersistence, MosaicPlacementPolicyDescriptor, MosaicPlacementPolicyId,
    MosaicPlacementReloadReconciliation, MosaicPlacementSource, MosaicPlacementSupport,
    MosaicPlacementTarget, MosaicRegionKindDescriptor, MosaicRegionKindId, MosaicRegionPersistence,
    MosaicRegionRole, MosaicResizePermission, MosaicScrollOwnership, MosaicSizingBehavior,
    MosaicSizingContractDescriptor, MosaicSizingContractId, MosaicSizingKind,
    MosaicSizingPersistence, MosaicStableIdentityBehavior, MosaicStateOwnerIdentity,
    MosaicStatePersistencePolicy, MosaicStateReplacementRule, MosaicStateSlotDescriptor,
    MosaicStateSlotId, MosaicStateSlotKind, MosaicStateTruthPosture, MosaicViewportConstraint,
    NamedMeasurementDefinition, NamedMeasurementToken, SurfaceDescriptor, SurfaceId, SurfaceKind,
    SurfacePlacementClass, SurfaceStateClass, ThemeTokenAlias, ThemeTokenDescriptor,
    ThemeTokenFamily, ThemeTokenId, ThemeTokenSource, ThemeTokenValue, UiThemeColor,
    ViewBindingDescriptor, ViewBindingId,
};
use crate::facade::{WorthUi, WorthUiApp};
pub(super) fn component_descriptor_variant_app() -> WorthUiApp {
    phase10_test_app(Phase10AppVariant::ComponentDescriptor)
}

pub(super) fn surface_descriptor_variant_app() -> WorthUiApp {
    phase10_test_app(Phase10AppVariant::SurfaceDescriptor)
}

pub(super) fn theme_token_alias_chain_variant_app() -> WorthUiApp {
    phase10_test_app(Phase10AppVariant::ThemeTokenAliasChain)
}

#[derive(Clone, Copy)]
enum Phase10AppVariant {
    ComponentDescriptor,
    SurfaceDescriptor,
    ThemeTokenAliasChain,
}

fn phase10_test_app(variant: Phase10AppVariant) -> WorthUiApp {
    let dashboard_component = match variant {
        Phase10AppVariant::ComponentDescriptor => component("workspace.component.dashboard")
            .with_execution_lane(ComponentExecutionLane::Interactive),
        _ => component("workspace.component.dashboard"),
    };
    let inspector_surface = match variant {
        Phase10AppVariant::SurfaceDescriptor => surface(
            "workspace.surface.inspector",
            "workspace.component.dashboard",
            SurfacePlacementClass::overlay_layer(),
        )
        .with_command_slot(CommandId::new("workspace.command.inspect").unwrap())
        .with_icon(IconId::new("workspace.icon.surface.inspector").unwrap())
        .with_view_binding(ViewBindingId::new("workspace.view_binding.selection").unwrap()),
        _ => default_inspector_surface(),
    };
    let mut app = WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .register_command(
            CommandDescriptor::new(
                CommandId::new("workspace.command.inspect").unwrap(),
                "Inspect",
            )
            .with_icon(IconId::new("workspace.icon.inspect").unwrap())
            .with_projection_eligibility(
                CommandProjectionId::new("workspace.command_projection.inspect_actions").unwrap(),
            ),
        )
        .register_component(dashboard_component)
        .register_component(component("workspace.component.inspector_panel"))
        .register_surface(surface(
            "workspace.surface.main",
            "workspace.component.dashboard",
            SurfacePlacementClass::primary_region(),
        ))
        .register_surface(surface(
            "workspace.surface.overlay",
            "workspace.component.inspector_panel",
            SurfacePlacementClass::overlay_layer(),
        ))
        .register_surface(inspector_surface)
        .register_icon(IconDescriptor::new(
            IconId::new("workspace.icon.inspect").unwrap(),
            IconFamily::command(),
            IconSourceDescriptor::symbol("inspect"),
        ))
        .register_icon(IconDescriptor::new(
            IconId::new("workspace.icon.surface.inspector").unwrap(),
            IconFamily::surface(),
            IconSourceDescriptor::symbol("panel"),
        ))
        .register_command_projection(
            CommandProjectionDescriptor::new(
                CommandProjectionId::new("workspace.command_projection.inspect_actions").unwrap(),
                CommandProjectionSurface::toolbar(),
            )
            .with_command_reference(CommandProjectionCommandReference::command(
                CommandId::new("workspace.command.inspect").unwrap(),
            )),
        )
        .register_view_binding(query_owned_view_binding_descriptor())
        .register_theme_token(ThemeTokenDescriptor::define(
            ThemeTokenId::new("theme.text.primary").unwrap(),
            ThemeTokenFamily::text(),
            ThemeTokenSource::application(),
            ThemeTokenValue::color(UiThemeColor::parse("#101820").unwrap()),
        ))
        .register_mosaic_region_kind(primary_region())
        .register_mosaic_region_kind(overlay_region())
        .register_mosaic_placement_policy(primary_placement())
        .register_mosaic_placement_policy(overlay_placement())
        .register_mosaic_sizing_contract(fill_sizing())
        .register_mosaic_sizing_contract(overlay_sizing())
        .register_mosaic_state_slot(region_scroll_state())
        .register_mosaic_state_slot(overlay_pinned_state())
        .register_mosaic_state_slot(primary_surface_state());

    if matches!(variant, Phase10AppVariant::ThemeTokenAliasChain) {
        app = app.register_theme_token(ThemeTokenDescriptor::alias(
            ThemeTokenId::new("theme.text.secondary").unwrap(),
            ThemeTokenFamily::text(),
            ThemeTokenSource::application(),
            ThemeTokenAlias::to(ThemeTokenId::new("theme.text.primary").unwrap()),
        ));
        app = app.register_theme_token(ThemeTokenDescriptor::alias(
            ThemeTokenId::new("theme.text.default").unwrap(),
            ThemeTokenFamily::text(),
            ThemeTokenSource::application(),
            ThemeTokenAlias::to(ThemeTokenId::new("theme.text.secondary").unwrap()),
        ));
    } else {
        app = app.register_theme_token(ThemeTokenDescriptor::alias(
            ThemeTokenId::new("theme.text.default").unwrap(),
            ThemeTokenFamily::text(),
            ThemeTokenSource::application(),
            ThemeTokenAlias::to(ThemeTokenId::new("theme.text.primary").unwrap()),
        ));
    }

    app.freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("application preparation should succeed")
}

fn default_inspector_surface() -> SurfaceDescriptor {
    surface(
        "workspace.surface.inspector",
        "workspace.component.dashboard",
        SurfacePlacementClass::primary_region(),
    )
    .with_command_slot(CommandId::new("workspace.command.inspect").unwrap())
    .with_icon(IconId::new("workspace.icon.surface.inspector").unwrap())
    .with_view_binding(ViewBindingId::new("workspace.view_binding.selection").unwrap())
}

fn query_owned_view_binding_descriptor() -> ViewBindingDescriptor {
    let definition = worth_ui_query_binding::WorthUiQueryViewDefinition::measurement_snapshot(
        "workspace.view_binding.selection",
    )
    .expect("query definition should admit");
    ViewBindingDescriptor::from_definition(
        ViewBindingId::new("workspace.view_binding.selection").unwrap(),
        crate::capability::ViewBindingFamily::collection(),
        definition,
    )
}

fn component(id: &str) -> ComponentDescriptor {
    ComponentDescriptor::new(
        ComponentId::new(id).unwrap(),
        ComponentPropSchema::named("workspace.props"),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
}

fn surface(
    id: &str,
    component_id: &str,
    placement_class: SurfacePlacementClass,
) -> SurfaceDescriptor {
    SurfaceDescriptor::new(
        SurfaceId::new(id).unwrap(),
        SurfaceKind::primary_content(),
        ComponentId::new(component_id).unwrap(),
        placement_class,
        SurfaceStateClass::restorable(),
    )
}

fn primary_region() -> MosaicRegionKindDescriptor {
    MosaicRegionKindDescriptor::new(
        MosaicRegionKindId::new("workspace.region.primary").unwrap(),
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

fn overlay_region() -> MosaicRegionKindDescriptor {
    MosaicRegionKindDescriptor::new(
        MosaicRegionKindId::new("workspace.region.overlay").unwrap(),
        MosaicRegionRole::overlay(),
    )
    .with_sizing_behavior(MosaicSizingBehavior::overlay_anchored())
    .with_scroll_ownership(MosaicScrollOwnership::no_scrolling())
    .with_focus_scope(MosaicFocusScopeKind::active_surface_scope())
    .with_child_rule(MosaicChildRule::accepts_surfaces())
    .with_allowed_surface_class(SurfacePlacementClass::overlay_layer())
    .with_persistence(MosaicRegionPersistence::restorable())
    .with_clipping(MosaicClippingPosture::clip_to_region())
    .with_hit_test(MosaicHitTestPosture::participates())
}

fn primary_placement() -> MosaicPlacementPolicyDescriptor {
    MosaicPlacementPolicyDescriptor::new(
        MosaicPlacementPolicyId::new("workspace.placement.primary").unwrap(),
        MosaicPlacementAction::dock(),
    )
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

fn overlay_placement() -> MosaicPlacementPolicyDescriptor {
    MosaicPlacementPolicyDescriptor::new(
        MosaicPlacementPolicyId::new("workspace.placement.overlay").unwrap(),
        MosaicPlacementAction::overlay(),
    )
    .with_source(MosaicPlacementSource::surface_class(
        SurfacePlacementClass::overlay_layer(),
    ))
    .with_target(MosaicPlacementTarget::region_role(
        MosaicRegionRole::overlay(),
    ))
    .with_persistence(MosaicPlacementPersistence::restorable())
    .with_stable_identity_behavior(MosaicStableIdentityBehavior::preserve_surface_identity())
    .with_conflict_behavior(MosaicPlacementConflictBehavior::reject_conflict())
    .with_reload_reconciliation(MosaicPlacementReloadReconciliation::restore_when_possible())
    .with_support(MosaicPlacementSupport::supported())
}

fn fill_sizing() -> MosaicSizingContractDescriptor {
    MosaicSizingContractDescriptor::new(
        MosaicSizingContractId::new("workspace.sizing.fill").unwrap(),
        MosaicSizingKind::fill(),
    )
    .with_measurement_authority(MosaicMeasurementAuthority::runtime_token())
    .with_resize_permission(MosaicResizePermission::user_resizable())
    .with_persistence(MosaicSizingPersistence::restorable())
    .with_overflow_behavior(MosaicOverflowBehavior::scroll_when_constrained())
    .with_parent_growth_behavior(MosaicParentGrowthBehavior::does_not_force_parent())
    .with_viewport_constraint(MosaicViewportConstraint::clamp_to_viewport())
    .with_named_measurement(measurement("workspace.measurement.fill"))
}

fn overlay_sizing() -> MosaicSizingContractDescriptor {
    MosaicSizingContractDescriptor::new(
        MosaicSizingContractId::new("workspace.sizing.overlay").unwrap(),
        MosaicSizingKind::fixed(),
    )
    .with_measurement_authority(MosaicMeasurementAuthority::runtime_token())
    .with_resize_permission(MosaicResizePermission::user_resizable())
    .with_persistence(MosaicSizingPersistence::restorable())
    .with_overflow_behavior(MosaicOverflowBehavior::scroll_when_constrained())
    .with_parent_growth_behavior(MosaicParentGrowthBehavior::does_not_force_parent())
    .with_viewport_constraint(MosaicViewportConstraint::clamp_to_viewport())
    .with_named_measurement(measurement("workspace.measurement.overlay"))
}

fn region_scroll_state() -> MosaicStateSlotDescriptor {
    MosaicStateSlotDescriptor::new(
        MosaicStateSlotId::new("workspace.state.region_scroll").unwrap(),
        MosaicStateSlotKind::scroll_position(),
    )
    .with_owner_identity(MosaicStateOwnerIdentity::mosaic_region_kind(
        MosaicRegionKindId::new("workspace.region.primary").unwrap(),
    ))
    .with_persistence_policy(MosaicStatePersistencePolicy::restore_across_hot_reload())
    .with_replacement_rule(MosaicStateReplacementRule::preserve_when_owner_matches())
    .with_truth_posture(MosaicStateTruthPosture::ui_runtime_state())
}

fn overlay_pinned_state() -> MosaicStateSlotDescriptor {
    MosaicStateSlotDescriptor::new(
        MosaicStateSlotId::new("workspace.state.overlay_pinned").unwrap(),
        MosaicStateSlotKind::pinned_posture(),
    )
    .with_owner_identity(MosaicStateOwnerIdentity::mosaic_region_kind(
        MosaicRegionKindId::new("workspace.region.overlay").unwrap(),
    ))
    .with_persistence_policy(MosaicStatePersistencePolicy::restore_across_hot_reload())
    .with_replacement_rule(MosaicStateReplacementRule::preserve_when_owner_matches())
    .with_truth_posture(MosaicStateTruthPosture::ui_runtime_state())
}

fn primary_surface_state() -> MosaicStateSlotDescriptor {
    MosaicStateSlotDescriptor::new(
        MosaicStateSlotId::new("workspace.state.primary_surface").unwrap(),
        MosaicStateSlotKind::active_primary_surface(),
    )
    .with_owner_identity(MosaicStateOwnerIdentity::surface(
        SurfaceId::new("workspace.surface.main").unwrap(),
    ))
    .with_persistence_policy(MosaicStatePersistencePolicy::restore_across_hot_reload())
    .with_replacement_rule(MosaicStateReplacementRule::preserve_when_owner_matches())
    .with_truth_posture(MosaicStateTruthPosture::ui_runtime_state())
}

fn measurement(id: &str) -> NamedMeasurementDefinition {
    NamedMeasurementDefinition::new(
        NamedMeasurementToken::new(id).unwrap(),
        MeasurementValue::logical_pixels(320),
        MeasurementConstraint::between(
            MeasurementValue::logical_pixels(200),
            MeasurementValue::logical_pixels(640),
        ),
    )
}
