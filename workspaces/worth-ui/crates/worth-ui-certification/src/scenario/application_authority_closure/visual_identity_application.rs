use super::fixed_application_builder::FixedCertificationApplicationBuilder;
use super::fixed_host::FixedCertificationHostBinding;
use worth_ui::facade::app::WorthUi;
use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentChildPolicy, ComponentDescriptor,
    ComponentFocusSupport, ComponentHitTestContract, ComponentHitTestInset, ComponentHitTestOrder,
    ComponentId, ComponentPortalChildContract, ComponentPropSchema, ComponentSemanticTextContract,
    ComponentStateOwnership, ComponentViewportInset, SurfaceDescriptor, SurfaceId, SurfaceKind,
    SurfacePlacementClass, SurfaceStateClass, ThemeTokenAlias, ThemeTokenDescriptor,
    ThemeTokenFamily, ThemeTokenId, ThemeTokenSource, ThemeTokenValue, UiThemeColor,
};

type WorthUiApplicationBuilder = FixedCertificationApplicationBuilder;

pub(crate) const VISUAL_PAINT_ONLY_COMPONENT: &str = "visual.identity.component.paint_only";
pub(crate) const VISUAL_HIT_ONLY_COMPONENT: &str = "visual.identity.component.hit_only";
pub(crate) const VISUAL_PAINT_AND_HIT_COMPONENT: &str = "visual.identity.component.paint_and_hit";
pub(crate) const VISUAL_NEITHER_COMPONENT: &str = "visual.identity.component.neither";
pub(crate) const VISUAL_IDENTITY_SURFACE: &str = "visual.identity.surface.main";

pub(crate) const VISUAL_PAINT_ONLY_TOKEN: &str = "theme.visual_identity.paint_only";
pub(crate) const VISUAL_PAINT_AND_HIT_TOKEN: &str = "theme.visual_identity.paint_and_hit";
pub(crate) const VISUAL_RED_TOKEN: &str = "theme.visual_identity.red";
pub(crate) const VISUAL_PURPLE_TOKEN: &str = "theme.visual_identity.purple";
pub(crate) const PHASE5_CANCELLATION_BACKGROUND: &str = "phase5.cancel.background";
pub(crate) const PHASE5_CANCELLATION_COMPONENT: &str = "phase5.cancel.component";
pub(crate) const PHASE5_CANCELLATION_SURFACE: &str = "phase5.cancel.surface";
pub(crate) const PHASE5_CANCELLATION_TOKEN: &str = "theme.phase5.cancel.foreground";
pub(crate) const PHASE5_CANCELLATION_COLOR_TOKEN: &str = "theme.phase5.cancel.color";

pub(crate) fn visual_identity_application_builder_with_host<Host>(
    host: Host,
) -> WorthUiApplicationBuilder
where
    Host: FixedCertificationHostBinding,
{
    visual_identity_builder(
        host,
        ComponentHitTestOrder::front_to_back(0),
        7,
        None,
        false,
    )
}

pub(crate) fn duplicate_hit_order_application_builder_with_host<Host>(
    host: Host,
) -> WorthUiApplicationBuilder
where
    Host: FixedCertificationHostBinding,
{
    visual_identity_builder(
        host,
        ComponentHitTestOrder::front_to_back(1),
        7,
        None,
        false,
    )
}

pub(crate) fn region_identity_application_builder_with_host<Host>(
    host: Host,
) -> WorthUiApplicationBuilder
where
    Host: FixedCertificationHostBinding,
{
    visual_identity_builder(
        host,
        ComponentHitTestOrder::front_to_back(0),
        1,
        None,
        false,
    )
}

pub(crate) fn clipped_visual_identity_application_builder_with_host<Host>(
    host: Host,
) -> WorthUiApplicationBuilder
where
    Host: FixedCertificationHostBinding,
{
    visual_identity_builder(
        host,
        ComponentHitTestOrder::front_to_back(0),
        7,
        Some(ComponentHitTestInset::symmetric(12, 8)),
        false,
    )
}

pub(crate) fn clipped_semantic_text_action_application_builder_with_host<Host>(
    host: Host,
) -> WorthUiApplicationBuilder
where
    Host: FixedCertificationHostBinding,
{
    visual_identity_builder(
        host,
        ComponentHitTestOrder::front_to_back(0),
        7,
        Some(ComponentHitTestInset::symmetric(12, 8)),
        true,
    )
}

pub(crate) fn focusable_semantic_text_action_application_builder_with_host<Host>(
    host: Host,
) -> WorthUiApplicationBuilder
where
    Host: FixedCertificationHostBinding,
{
    visual_identity_builder_with_profile(
        host,
        ComponentHitTestOrder::front_to_back(0),
        7,
        Some(ComponentHitTestInset::symmetric(12, 8)),
        true,
        true,
        worth_ui::facade::rebind::UiChangeProfile::platform_pulse(),
    )
}

pub(crate) fn portal_semantic_text_action_application_builder_with_host<Host>(
    host: Host,
) -> WorthUiApplicationBuilder
where
    Host: FixedCertificationHostBinding,
{
    visual_identity_builder_with_profile_and_portal_child(
        host,
        ComponentHitTestOrder::front_to_back(0),
        7,
        Some(ComponentHitTestInset::symmetric(12, 8)),
        true,
        true,
        worth_ui::facade::rebind::UiChangeProfile::platform_pulse(),
        Some(VISUAL_PAINT_AND_HIT_COMPONENT),
    )
}

pub(crate) fn single_semantic_text_application_builder_with_host<Host>(
    host: Host,
) -> WorthUiApplicationBuilder
where
    Host: FixedCertificationHostBinding,
{
    let text_component = component(PHASE5_CANCELLATION_COMPONENT)
        .with_allocation_measurement_contract(
            ComponentAllocationMeasurementContract::viewport_inset(
                ComponentViewportInset::symmetric(24, 16),
            ),
        )
        .with_surface_paint_order(1)
        .with_semantic_text(ComponentSemanticTextContract::body_default(
            token_id(PHASE5_CANCELLATION_TOKEN),
            2,
        ));
    let builder = WorthUi::app()
        .with_change_profile(worth_ui::facade::rebind::UiChangeProfile::platform_pulse())
        .register_component(
            component(PHASE5_CANCELLATION_BACKGROUND)
                .with_allocation_measurement_contract(
                    ComponentAllocationMeasurementContract::fill_viewport(),
                )
                .with_surface_paint_order(0),
        )
        .register_component(text_component)
        .register_surface(SurfaceDescriptor::new(
            SurfaceId::new(PHASE5_CANCELLATION_SURFACE).expect("valid cancellation surface id"),
            SurfaceKind::primary_content(),
            ComponentId::new(PHASE5_CANCELLATION_BACKGROUND)
                .expect("valid cancellation background id"),
            SurfacePlacementClass::primary_region(),
            SurfaceStateClass::ephemeral(),
        ))
        .register_theme_token(color_token(PHASE5_CANCELLATION_COLOR_TOKEN, "#d8e8ff"))
        .register_theme_token(alias_token(
            PHASE5_CANCELLATION_TOKEN,
            PHASE5_CANCELLATION_COLOR_TOKEN,
        ));
    FixedCertificationApplicationBuilder::new(builder, host)
}

pub(crate) fn clipped_semantic_text_action_application_builder_with_host_and_profile<Host>(
    host: Host,
    profile: worth_ui::facade::rebind::UiChangeProfile,
) -> WorthUiApplicationBuilder
where
    Host: FixedCertificationHostBinding,
{
    visual_identity_builder_with_profile(
        host,
        ComponentHitTestOrder::front_to_back(0),
        7,
        Some(ComponentHitTestInset::symmetric(12, 8)),
        true,
        false,
        profile,
    )
}

fn visual_identity_builder<Host>(
    host: Host,
    paint_and_hit_order: ComponentHitTestOrder,
    paint_only_order: u32,
    hit_only_clip: Option<ComponentHitTestInset>,
    paint_and_hit_semantic_text: bool,
) -> WorthUiApplicationBuilder
where
    Host: FixedCertificationHostBinding,
{
    visual_identity_builder_with_profile(
        host,
        paint_and_hit_order,
        paint_only_order,
        hit_only_clip,
        paint_and_hit_semantic_text,
        false,
        worth_ui::facade::rebind::UiChangeProfile::platform_pulse(),
    )
}

fn visual_identity_builder_with_profile<Host>(
    host: Host,
    paint_and_hit_order: ComponentHitTestOrder,
    paint_only_order: u32,
    hit_only_clip: Option<ComponentHitTestInset>,
    paint_and_hit_semantic_text: bool,
    paint_and_hit_focusable: bool,
    profile: worth_ui::facade::rebind::UiChangeProfile,
) -> WorthUiApplicationBuilder
where
    Host: FixedCertificationHostBinding,
{
    visual_identity_builder_with_profile_and_portal_child(
        host,
        paint_and_hit_order,
        paint_only_order,
        hit_only_clip,
        paint_and_hit_semantic_text,
        paint_and_hit_focusable,
        profile,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn visual_identity_builder_with_profile_and_portal_child<Host>(
    host: Host,
    paint_and_hit_order: ComponentHitTestOrder,
    paint_only_order: u32,
    hit_only_clip: Option<ComponentHitTestInset>,
    paint_and_hit_semantic_text: bool,
    paint_and_hit_focusable: bool,
    profile: worth_ui::facade::rebind::UiChangeProfile,
    portal_child_owner: Option<&str>,
) -> WorthUiApplicationBuilder
where
    Host: FixedCertificationHostBinding,
{
    let hit_only_allocation = inset_allocation(8, 8);
    let paint_and_hit_allocation = inset_allocation(16, 12);
    let hit_only_contract = match hit_only_clip {
        Some(inset) => ComponentHitTestContract::allocation_bounds_clipped_by_inset(
            ComponentHitTestOrder::front_to_back(1),
            hit_only_allocation,
            inset,
        ),
        None => ComponentHitTestContract::allocation_bounds(
            ComponentHitTestOrder::front_to_back(1),
            hit_only_allocation,
        ),
    };
    let hit_only = match portal_child_owner {
        Some(_) => text_component(VISUAL_HIT_ONLY_COMPONENT),
        None => component(VISUAL_HIT_ONLY_COMPONENT),
    }
    .with_hit_test(hit_only_contract);
    let hit_only = match portal_child_owner {
        Some(owner) => hit_only
            .with_allocation_measurement_contract(hit_only_allocation)
            .with_surface_paint_order(4)
            .with_semantic_text(ComponentSemanticTextContract::body_default(
                token_id(VISUAL_PAINT_AND_HIT_TOKEN),
                5,
            ))
            .with_portal_child(ComponentPortalChildContract::new(
                ComponentId::new(owner).expect("valid portal child owner component id"),
            )),
        None => hit_only,
    };
    let paint_and_hit = component(VISUAL_PAINT_AND_HIT_COMPONENT)
        .with_allocation_measurement_contract(paint_and_hit_allocation)
        .with_surface_paint_order(3)
        .with_hit_test(ComponentHitTestContract::allocation_bounds(
            paint_and_hit_order,
            paint_and_hit_allocation,
        ));
    let paint_and_hit = if paint_and_hit_semantic_text {
        paint_and_hit.with_semantic_text(ComponentSemanticTextContract::body_default(
            token_id(VISUAL_PAINT_AND_HIT_TOKEN),
            4,
        ))
    } else {
        paint_and_hit
    };
    let paint_and_hit = if paint_and_hit_focusable {
        paint_and_hit.with_focus(ComponentFocusSupport::focusable())
    } else {
        paint_and_hit
    };
    let builder = WorthUi::app()
        .with_change_profile(profile)
        .register_component(
            component(VISUAL_PAINT_ONLY_COMPONENT)
                .with_allocation_measurement_contract(
                    ComponentAllocationMeasurementContract::fill_viewport(),
                )
                .with_surface_paint_order(paint_only_order),
        )
        .register_component(hit_only)
        .register_component(paint_and_hit)
        .register_component(component(VISUAL_NEITHER_COMPONENT))
        .register_surface(SurfaceDescriptor::new(
            SurfaceId::new(VISUAL_IDENTITY_SURFACE).expect("valid visual identity surface id"),
            SurfaceKind::primary_content(),
            ComponentId::new(VISUAL_PAINT_ONLY_COMPONENT)
                .expect("valid visual identity root component id"),
            SurfacePlacementClass::primary_region(),
            SurfaceStateClass::ephemeral(),
        ))
        .register_theme_token(color_token(VISUAL_RED_TOKEN, "#cf222e"))
        .register_theme_token(color_token(VISUAL_PURPLE_TOKEN, "#8250df"))
        .register_theme_token(alias_token(VISUAL_PAINT_ONLY_TOKEN, VISUAL_RED_TOKEN))
        .register_theme_token(alias_token(VISUAL_PAINT_AND_HIT_TOKEN, VISUAL_PURPLE_TOKEN));
    FixedCertificationApplicationBuilder::new(builder, host)
}

fn component(id: &str) -> ComponentDescriptor {
    ComponentDescriptor::new(
        ComponentId::new(id).expect("valid visual identity component id"),
        ComponentPropSchema::named(format!("{id}.props")),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
}

fn text_component(id: &str) -> ComponentDescriptor {
    ComponentDescriptor::new(
        ComponentId::new(id).expect("valid visual identity text component id"),
        ComponentPropSchema::named(format!("{id}.props")),
        ComponentChildPolicy::text_children(),
        ComponentStateOwnership::runtime_owned(),
    )
}

fn inset_allocation(horizontal: u16, vertical: u16) -> ComponentAllocationMeasurementContract {
    ComponentAllocationMeasurementContract::viewport_inset(ComponentViewportInset::symmetric(
        horizontal, vertical,
    ))
}

fn color_token(id: &str, color: &str) -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::define(
        token_id(id),
        ThemeTokenFamily::surface(),
        ThemeTokenSource::application(),
        ThemeTokenValue::color(
            UiThemeColor::parse(color).expect("valid visual identity theme color"),
        ),
    )
}

fn alias_token(id: &str, target: &str) -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::alias(
        token_id(id),
        ThemeTokenFamily::surface(),
        ThemeTokenSource::application(),
        ThemeTokenAlias::to(token_id(target)),
    )
}

fn token_id(id: &str) -> ThemeTokenId {
    ThemeTokenId::new(id).expect("valid visual identity theme token id")
}
