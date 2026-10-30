use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;

use crate::application_schema::WorthQueryInstalledApplicationSchemaContractCatalog;
use crate::domain_operation::{
    WorthQueryOperationApplicationProjectionScope, WorthQueryOperationEntityReadScope,
    WorthQueryOperationGraphAccess, WorthQueryOperationGraphParticipation,
    WorthQueryOperationGraphReadContract, WorthQueryOperationGraphReadRole,
    WorthQueryOperationGraphReadScope, WorthQueryOperationNativeProjectionContract,
    WorthQueryOperationRelationReadScope,
};
use crate::package::WorthQueryPortableOperationGraphReadScope;

pub(crate) fn install_portable_graph_reads(
    binding: &ApplicationSchemaBindingIdentity,
    catalog: &WorthQueryInstalledApplicationSchemaContractCatalog,
    reads: &[WorthQueryPortableOperationGraphReadScope],
) -> Result<WorthQueryOperationGraphReadContract, ()> {
    let mut scopes = Vec::with_capacity(reads.len());
    for read in reads {
        let scope = match read {
            WorthQueryPortableOperationGraphReadScope::DerivedSource { relation, .. } => {
                WorthQueryOperationGraphReadScope::DerivedSource(
                    crate::domain_operation::WorthQueryOperationDerivedSourceReadScope::new(
                        binding.clone(),
                        relation.clone(),
                    ),
                )
            }
            WorthQueryPortableOperationGraphReadScope::Components { definition, .. } => {
                WorthQueryOperationGraphReadScope::Components(
                    crate::domain_operation::WorthQueryOperationComponentReadScope::new(
                        binding.clone(),
                        definition.clone(),
                    ),
                )
            }
            WorthQueryPortableOperationGraphReadScope::Entity { entity, .. } => {
                WorthQueryOperationGraphReadScope::Entity(WorthQueryOperationEntityReadScope::new(
                    binding.clone(),
                    entity.clone(),
                ))
            }
            WorthQueryPortableOperationGraphReadScope::NativeProjection {
                entity,
                aspect,
                contract,
                mask,
                ..
            } => {
                let installed = catalog.aspect(entity, aspect.as_str()).ok_or(())?;
                if installed.contract() != contract {
                    return Err(());
                }
                let projection = WorthQueryOperationNativeProjectionContract::from_installed(
                    installed,
                    mask.clone(),
                )
                .map_err(|_| ())?;
                WorthQueryOperationGraphReadScope::NativeProjection(
                    WorthQueryOperationApplicationProjectionScope::new(
                        binding.clone(),
                        entity.clone(),
                        aspect.clone(),
                        projection,
                        installed.binding().clone(),
                    ),
                )
            }
            WorthQueryPortableOperationGraphReadScope::Relation {
                relation, from, to, ..
            } => WorthQueryOperationGraphReadScope::Relation(
                WorthQueryOperationRelationReadScope::new(
                    binding.clone(),
                    relation.clone(),
                    from.clone(),
                    to.clone(),
                ),
            ),
        };
        scopes.push(scope);
    }
    Ok(WorthQueryOperationGraphReadContract::Declared {
        roles: vec![WorthQueryOperationGraphReadRole::new(
            "primary".to_owned(),
            WorthQueryOperationGraphParticipation::PrimaryLogicalGraph,
            WorthQueryOperationGraphAccess::Project,
            scopes,
        )],
    })
}
