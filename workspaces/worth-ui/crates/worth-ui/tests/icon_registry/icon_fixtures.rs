use worth_ui::facade::declaration::{
    CommandDescriptor, CommandId, ComponentChildPolicy, ComponentDescriptor, ComponentId,
    ComponentPropSchema, ComponentStateOwnership, IconDescriptor, IconFamily, IconId,
    IconSourceDescriptor, RuntimeOutcomeDenialPosture, RuntimeOutcomeFamily,
    RuntimeOutcomePresentation, RuntimeOutcomeProjectionDescriptor, RuntimeOutcomeProjectionId,
    RuntimeOutcomeSourceReference, SurfaceDescriptor, SurfaceId, SurfaceKind,
    SurfacePlacementClass, SurfaceStateClass, ThemeTokenDescriptor, ThemeTokenFamily, ThemeTokenId,
    ThemeTokenSource, ThemeTokenValue, UiThemeColor,
};

pub(crate) fn command_icon(id: &str) -> IconDescriptor {
    IconDescriptor::new(
        icon_id(id),
        IconFamily::command(),
        IconSourceDescriptor::symbol(id),
    )
}

pub(crate) fn surface_icon(id: &str) -> IconDescriptor {
    IconDescriptor::new(
        icon_id(id),
        IconFamily::surface(),
        IconSourceDescriptor::symbol(id),
    )
}

pub(crate) fn runtime_outcome_icon(id: &str) -> IconDescriptor {
    IconDescriptor::new(
        icon_id(id),
        IconFamily::runtime_outcome(),
        IconSourceDescriptor::symbol(id),
    )
}

pub(crate) fn icon_id(raw_text: &str) -> IconId {
    IconId::new(raw_text).expect("valid icon id")
}

pub(crate) fn command_descriptor(id: &str, label: &str) -> CommandDescriptor {
    CommandDescriptor::new(command_id(id), label)
}

pub(crate) fn command_id(raw_text: &str) -> CommandId {
    CommandId::new(raw_text).expect("valid command id")
}

pub(crate) fn component_descriptor(id: &str) -> ComponentDescriptor {
    ComponentDescriptor::new(
        component_id(id),
        ComponentPropSchema::named(format!("{id}.props")),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
}

pub(crate) fn component_id(raw_text: &str) -> ComponentId {
    ComponentId::new(raw_text).expect("valid component id")
}

pub(crate) fn surface_descriptor(id: &str, component_id: &str) -> SurfaceDescriptor {
    SurfaceDescriptor::new(
        surface_id(id),
        SurfaceKind::primary_content(),
        self::component_id(component_id),
        SurfacePlacementClass::primary_region(),
        SurfaceStateClass::restorable(),
    )
}

pub(crate) fn surface_id(raw_text: &str) -> SurfaceId {
    SurfaceId::new(raw_text).expect("valid surface id")
}

pub(crate) fn color_theme_token(id: &str, hex: &str) -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::define(
        theme_token_id(id),
        ThemeTokenFamily::text(),
        ThemeTokenSource::application(),
        ThemeTokenValue::color(UiThemeColor::parse(hex).expect("valid theme color value")),
    )
}

pub(crate) fn theme_token_id(raw_text: &str) -> ThemeTokenId {
    ThemeTokenId::new(raw_text).expect("valid theme token id")
}

pub(crate) fn denied_projection_with_icon(
    projection_id: &str,
    icon: &str,
) -> RuntimeOutcomeProjectionDescriptor {
    RuntimeOutcomeProjectionDescriptor::new(
        RuntimeOutcomeProjectionId::new(projection_id)
            .expect("valid runtime outcome projection id"),
        RuntimeOutcomeFamily::denied(),
        RuntimeOutcomeSourceReference::new(RuntimeOutcomeFamily::denied()),
    )
    .with_presentation(RuntimeOutcomePresentation::new().with_icon(icon_id(icon)))
    .with_denial_posture(RuntimeOutcomeDenialPosture::structured_status())
}
