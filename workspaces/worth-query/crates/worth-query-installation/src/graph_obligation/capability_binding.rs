//! Resolve the declared disclosure audience to real installed capability contracts.
use super::WorthQueryInstalledGraphCapabilityRequirement;
use crate::application_schema::WorthQueryInstalledApplicationSchema;
use worth_query_declaration::facade::{
    application_query::ApplicationQueryDisclosureContract, application_schema::ApplicationSchema,
};

pub(crate) fn install_disclosure_capabilities<Schema: ApplicationSchema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    disclosure: &ApplicationQueryDisclosureContract,
) -> Vec<WorthQueryInstalledGraphCapabilityRequirement> {
    let (Some(name), Some(capability_type)) =
        (disclosure.capability_name(), disclosure.capability_type())
    else {
        return Vec::new();
    };
    schema
        .capability_registry
        .values()
        .filter(|compiled| {
            compiled.contract().name() == name
                && compiled.contract().capability_type() == capability_type
        })
        .map(|compiled| {
            WorthQueryInstalledGraphCapabilityRequirement::new(
                compiled.identity().clone(),
                compiled.contract().clone(),
            )
        })
        .collect()
}
