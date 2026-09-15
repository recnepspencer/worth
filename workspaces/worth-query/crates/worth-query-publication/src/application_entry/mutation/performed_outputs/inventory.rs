use std::collections::BTreeSet;

use worth_query_declaration::facade::application_program::{
    ApplicationProgramDefinition, ApplicationProgramInventoryIdentity,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;

pub(super) fn inventory_closure<Schema, Program, Inventory>(
    application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
) -> Result<
    (BTreeSet<&'static str>, BTreeSet<&'static str>),
    crate::application_entry::WorthQueryRequiredOutputPreparationDenial,
>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
{
    let inventory = application
        .installed_program()
        .inventory::<Inventory>()
        .ok_or(
            crate::application_entry::WorthQueryRequiredOutputPreparationDenial::MissingInventory,
        )?;
    let inventory_feature_types = inventory
        .outputs()
        .iter()
        .map(|output| output.feature_type())
        .collect::<BTreeSet<_>>();
    let required_types = application
        .installed_program()
        .inventory_feature_closure::<Inventory>()
        .expect("the inventory was resolved above");
    let identities = application
        .installed_program()
        .features()
        .iter()
        .filter(|feature| required_types.contains(&feature.type_id()))
        .map(|feature| feature.identity())
        .collect();
    let inventory_features = application
        .installed_program()
        .features()
        .iter()
        .filter(|feature| inventory_feature_types.contains(&feature.type_id()))
        .map(|feature| feature.identity())
        .collect();
    Ok((identities, inventory_features))
}
