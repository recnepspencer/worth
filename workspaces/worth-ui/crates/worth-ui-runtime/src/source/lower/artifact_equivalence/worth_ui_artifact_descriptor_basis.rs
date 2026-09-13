use crate::capability::{
    CommandDescriptor, ComponentDescriptor, IconDescriptor, MosaicPlacementPolicyDescriptor,
    MosaicRegionKindDescriptor, MosaicSizingContractDescriptor, MosaicStateSlotDescriptor,
    SurfaceDescriptor, ThemeTokenDescriptor,
};

pub(super) fn component_descriptor_basis(descriptor: &ComponentDescriptor) -> String {
    let theme_token_dependencies = descriptor
        .theme_token_dependencies()
        .iter()
        .map(|token_id| token_id.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let command_binding_slots = descriptor
        .command_binding_slots()
        .iter()
        .map(|command_id| command_id.as_str())
        .collect::<Vec<_>>()
        .join(",");
    [
        descriptor.id().as_str().to_owned(),
        option_digest_basis(descriptor.prop_schema().map(|schema| schema.digest_basis())),
        descriptor.child_policy().as_str().to_owned(),
        option_text_basis(
            descriptor
                .state_ownership()
                .map(|ownership| ownership.as_str()),
        ),
        descriptor.accessibility().as_str().to_owned(),
        descriptor.focus().as_str().to_owned(),
        format!("theme_token_dependencies:[{theme_token_dependencies}]"),
        format!("command_binding_slots:[{command_binding_slots}]"),
        descriptor.execution_lane().as_str().to_owned(),
        option_digest_basis(
            descriptor
                .allocation_measurement_contract()
                .map(|contract| contract.digest_basis()),
        ),
    ]
    .join("|")
}

pub(super) fn command_descriptor_basis(descriptor: &CommandDescriptor) -> String {
    [
        descriptor.id().as_str().to_owned(),
        descriptor.label().to_owned(),
        option_text_basis(descriptor.description()),
        option_id_basis(descriptor.icon().map(|icon| icon.as_str())),
        option_digest_basis(
            descriptor
                .default_shortcut()
                .map(|shortcut| shortcut.digest_basis()),
        ),
        option_digest_basis(descriptor.route().map(|route| route.digest_basis())),
        descriptor.category().as_str().to_owned(),
        option_id_basis(
            descriptor
                .projection_eligibility()
                .map(|projection| projection.as_str()),
        ),
    ]
    .join("|")
}

pub(super) fn surface_descriptor_basis(descriptor: &SurfaceDescriptor) -> String {
    let command_slots = descriptor
        .command_slots()
        .iter()
        .map(|command| command.as_str())
        .collect::<Vec<_>>()
        .join(",");
    [
        descriptor.id().as_str().to_owned(),
        descriptor.kind().digest_basis(),
        descriptor.component_id().as_str().to_owned(),
        descriptor.placement_class().digest_basis(),
        descriptor.state_class().digest_basis(),
        format!("command_slots:[{command_slots}]"),
        option_text_basis(descriptor.label()),
        option_id_basis(descriptor.icon().map(|icon| icon.as_str())),
        option_id_basis(
            descriptor
                .view_binding()
                .map(|view_binding| view_binding.as_str()),
        ),
    ]
    .join("|")
}

pub(super) fn icon_descriptor_basis(descriptor: &IconDescriptor) -> String {
    [
        descriptor.id().as_str().to_owned(),
        descriptor.family().digest_basis(),
        descriptor
            .source()
            .map(|source| source.digest_basis())
            .unwrap_or_else(|| "source:none".to_owned()),
        descriptor.theme_posture().digest_basis().to_owned(),
        descriptor.accessibility_posture().digest_basis().to_owned(),
    ]
    .join("|")
}

pub(super) fn theme_token_descriptor_basis(descriptor: &ThemeTokenDescriptor) -> String {
    [
        descriptor.id().as_str().to_owned(),
        descriptor.family().digest_basis(),
        descriptor.source().digest_basis().to_owned(),
        option_digest_basis(descriptor.value().map(|value| value.digest_basis())),
        option_digest_basis(
            descriptor
                .alias_definition()
                .map(|alias| alias.digest_basis()),
        ),
    ]
    .join("|")
}

pub(super) fn mosaic_region_descriptor_basis(descriptor: &MosaicRegionKindDescriptor) -> String {
    let surface_classes = descriptor
        .allowed_surface_classes()
        .iter()
        .map(|surface_class| surface_class.digest_basis())
        .collect::<Vec<_>>()
        .join(",");
    [
        descriptor.id().as_str().to_owned(),
        descriptor.role().digest_basis().to_owned(),
        option_digest_basis(
            descriptor
                .sizing_behavior()
                .map(|value| value.digest_basis()),
        ),
        option_digest_basis(
            descriptor
                .scroll_ownership()
                .map(|value| value.digest_basis()),
        ),
        option_digest_basis(descriptor.focus_scope().map(|value| value.digest_basis())),
        option_digest_basis(descriptor.child_rule().map(|value| value.digest_basis())),
        format!("allowed_surface_classes:[{surface_classes}]"),
        option_digest_basis(descriptor.persistence().map(|value| value.digest_basis())),
        option_digest_basis(descriptor.clipping().map(|value| value.digest_basis())),
        option_digest_basis(descriptor.hit_test().map(|value| value.digest_basis())),
        option_text_basis(descriptor.label()),
    ]
    .join("|")
}

pub(super) fn mosaic_placement_descriptor_basis(
    descriptor: &MosaicPlacementPolicyDescriptor,
) -> String {
    [
        descriptor.id().as_str().to_owned(),
        descriptor.action().digest_basis().to_owned(),
        option_digest_basis(descriptor.source().map(|value| value.digest_basis())),
        option_digest_basis(descriptor.target().map(|value| value.digest_basis())),
        option_digest_basis(descriptor.persistence().map(|value| value.digest_basis())),
        option_digest_basis(
            descriptor
                .stable_identity_behavior()
                .map(|value| value.digest_basis()),
        ),
        option_digest_basis(
            descriptor
                .conflict_behavior()
                .map(|value| value.digest_basis()),
        ),
        option_digest_basis(
            descriptor
                .reload_reconciliation()
                .map(|value| value.digest_basis()),
        ),
        option_digest_basis(descriptor.support().map(|value| value.digest_basis())),
        option_text_basis(descriptor.label()),
    ]
    .join("|")
}

pub(super) fn mosaic_sizing_descriptor_basis(
    descriptor: &MosaicSizingContractDescriptor,
) -> String {
    let diagnostics = descriptor
        .raw_measurements_for_diagnostics()
        .iter()
        .map(|measurement| measurement.digest_basis())
        .collect::<Vec<_>>()
        .join(",");
    [
        descriptor.id().as_str().to_owned(),
        descriptor.kind().digest_basis().to_owned(),
        descriptor
            .named_measurement()
            .map(|measurement| measurement.digest_basis())
            .unwrap_or_else(|| "measurement:none".to_owned()),
        option_digest_basis(
            descriptor
                .measurement_authority()
                .map(|value| value.digest_basis()),
        ),
        option_digest_basis(
            descriptor
                .resize_permission()
                .map(|value| value.digest_basis()),
        ),
        option_digest_basis(descriptor.persistence().map(|value| value.digest_basis())),
        option_digest_basis(
            descriptor
                .overflow_behavior()
                .map(|value| value.digest_basis()),
        ),
        option_digest_basis(
            descriptor
                .parent_growth_behavior()
                .map(|value| value.digest_basis()),
        ),
        option_digest_basis(
            descriptor
                .viewport_constraint()
                .map(|value| value.digest_basis()),
        ),
        format!("raw_measurements:[{diagnostics}]"),
        option_text_basis(descriptor.label()),
    ]
    .join("|")
}

pub(super) fn mosaic_state_descriptor_basis(descriptor: &MosaicStateSlotDescriptor) -> String {
    [
        descriptor.id().as_str().to_owned(),
        descriptor.kind().digest_basis().to_owned(),
        option_digest_basis(
            descriptor
                .owner_identity()
                .map(|identity| identity.digest_basis()),
        ),
        option_digest_basis(
            descriptor
                .persistence_policy()
                .map(|policy| policy.digest_basis()),
        ),
        option_digest_basis(
            descriptor
                .replacement_rule()
                .map(|rule| rule.digest_basis()),
        ),
        option_digest_basis(
            descriptor
                .truth_posture()
                .map(|posture| posture.digest_basis()),
        ),
        option_text_basis(descriptor.label()),
    ]
    .join("|")
}

fn option_id_basis(value: Option<&str>) -> String {
    option_text_basis(value)
}

fn option_text_basis(value: Option<&str>) -> String {
    value
        .map(|value| format!("some:{value}"))
        .unwrap_or_else(|| "none".to_owned())
}

fn option_digest_basis(value: Option<impl Into<String>>) -> String {
    value
        .map(|value| format!("some:{}", value.into()))
        .unwrap_or_else(|| "none".to_owned())
}
