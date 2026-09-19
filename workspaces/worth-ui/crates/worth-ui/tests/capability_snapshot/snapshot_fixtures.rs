use worth_ui::facade::declaration::{
    CommandDescriptor, CommandId, ComponentChildPolicy, ComponentDescriptor, ComponentId,
    ComponentPropSchema, ComponentStateOwnership, IconDescriptor, IconFamily, IconId,
    IconSourceDescriptor, NativeCapabilityDescriptor, NativeCapabilityFamily, NativeCapabilityId,
    NativePlatformPosture, PluginCapabilityPermission, PluginContributionFamily,
    PluginSlotContributionReference, PluginSlotDescriptor, PluginSlotDiagnostics, PluginSlotId,
    PluginSlotOrdering, PluginSlotSupportPosture, ThemeTokenDescriptor, ThemeTokenFamily,
    ThemeTokenId, ThemeTokenSource, ThemeTokenValue, UiThemeColor,
};

pub(crate) fn command(id: &str, label: &str) -> CommandDescriptor {
    CommandDescriptor::new(command_id(id), label)
}

pub(crate) fn command_with_icon(id: &str, icon_id: &str) -> CommandDescriptor {
    command(id, "Save").with_icon(self::icon_id(icon_id))
}

pub(crate) fn command_id(raw_text: &str) -> CommandId {
    CommandId::new(raw_text).expect("valid command id")
}

pub(crate) fn component(id: &str) -> ComponentDescriptor {
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

pub(crate) fn command_icon(id: &str) -> IconDescriptor {
    IconDescriptor::new(
        icon_id(id),
        IconFamily::command(),
        IconSourceDescriptor::symbol(id),
    )
}

pub(crate) fn icon_id(raw_text: &str) -> IconId {
    IconId::new(raw_text).expect("valid icon id")
}

pub(crate) fn theme_token(id: &str) -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::define(
        theme_token_id(id),
        ThemeTokenFamily::text(),
        ThemeTokenSource::application(),
        ThemeTokenValue::color(UiThemeColor::parse("#3366ff").expect("valid color")),
    )
}

pub(crate) fn theme_token_id(raw_text: &str) -> ThemeTokenId {
    ThemeTokenId::new(raw_text).expect("valid theme token id")
}

pub(crate) fn deferred_native_capability(id: &str) -> NativeCapabilityDescriptor {
    NativeCapabilityDescriptor::new(native_capability_id(id))
        .with_family(NativeCapabilityFamily::clipboard())
        .with_platform_posture(NativePlatformPosture::deferred())
}

pub(crate) fn native_capability_id(raw_text: &str) -> NativeCapabilityId {
    NativeCapabilityId::new(raw_text).expect("valid native capability id")
}

pub(crate) fn plugin_slot(id: &str) -> PluginSlotDescriptor {
    PluginSlotDescriptor::new(plugin_slot_id(id))
        .allow_family(PluginContributionFamily::command())
        .with_permission(PluginCapabilityPermission::host_granted())
        .with_ordering(PluginSlotOrdering::stable_by_plugin_then_declaration())
        .with_diagnostics(PluginSlotDiagnostics::explain_contributions())
        .with_support(PluginSlotSupportPosture::supported())
}

pub(crate) fn deferred_plugin_slot(id: &str) -> PluginSlotDescriptor {
    plugin_slot(id).with_support(PluginSlotSupportPosture::deferred())
}

pub(crate) fn plugin_slot_referencing(id: &str, target_slot_id: &str) -> PluginSlotDescriptor {
    plugin_slot(id).with_contribution_reference(PluginSlotContributionReference::slot(
        plugin_slot_id(target_slot_id),
    ))
}

pub(crate) fn plugin_slot_id(raw_text: &str) -> PluginSlotId {
    PluginSlotId::new(raw_text).expect("valid plugin slot id")
}
