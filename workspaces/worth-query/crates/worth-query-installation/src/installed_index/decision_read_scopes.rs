//! Compile each content-addressed read scope once at schema installation.
use crate::application_schema::WorthQueryInstalledApplicationSchemaContractCatalog as Catalog;
use crate::package::WorthQueryPortableNativeAspectContractRecord;
use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaBindingIdentity, ApplicationSchemaMember as Member,
};

pub(super) fn compile(
    binding: &ApplicationSchemaBindingIdentity,
    catalog: &mut Catalog,
    schema: &str,
    members: &[Member],
    native: &[WorthQueryPortableNativeAspectContractRecord],
) -> Result<(), ()> {
    for member in members {
        let Member::DecisionReadScope { definition } = member else {
            continue;
        };
        let reads = crate::application_operation::compile_shared_read_scopes(
            schema, native, members, definition,
        )
        .map_err(|_| ())?;
        let compiled =
            crate::application_operation::install_shared_read_scopes(binding, catalog, &reads)?;
        let scopes = compiled
            .roles()
            .iter()
            .flat_map(|role| role.read_scopes())
            .cloned()
            .collect::<Vec<_>>();
        let parents = definition.targets().iter().filter_map(|target| match target {
            worth_query_declaration::facade::application_schema::ApplicationOperationDecisionReadTarget::ManagedIndex { parent } => Some(parent.clone()),
            _ => None,
        }).collect::<Vec<_>>();
        let role = crate::domain_operation::WorthQueryOperationGraphReadRole::shared(
            definition.identity(),
            scopes.into(),
            parents.into(),
        );
        if catalog
            .shared_decision_reads
            .insert(definition.identity().to_owned(), role)
            .is_some()
        {
            return Err(());
        }
    }
    Ok(())
}
