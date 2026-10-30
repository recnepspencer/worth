mod aspect_removal;
use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;

use crate::application_schema::WorthQueryInstalledApplicationSchemaContractCatalog;
use crate::domain_operation::{
    WorthQueryOperationEntityTouchScope, WorthQueryOperationFieldTouchScope,
    WorthQueryOperationRelationFieldInitializationScope, WorthQueryOperationRelationTouchScope,
    WorthQueryOperationTouchContract, WorthQueryOperationTouchScope,
};
use crate::package::WorthQueryPortableOperationTouchScope;

pub(crate) fn install_portable_graph_touches(
    binding: &ApplicationSchemaBindingIdentity,
    catalog: &WorthQueryInstalledApplicationSchemaContractCatalog,
    portable: &[WorthQueryPortableOperationTouchScope],
    reads: &crate::domain_operation::WorthQueryOperationGraphReadContract,
) -> Result<(WorthQueryOperationTouchContract, usize), ()> {
    let scopes = portable
        .iter()
        .map(|scope| match scope {
            WorthQueryPortableOperationTouchScope::RemoveAspect {
                entity, contract, ..
            } => aspect_removal::install(binding, catalog, entity, contract, reads),
            WorthQueryPortableOperationTouchScope::InitializeRelationField {
                relation,
                from,
                to,
                contract,
                field_path,
                ..
            } => {
                let installed = catalog
                    .relation_aspect(relation, contract.key().as_str())
                    .ok_or(())?;
                if installed.contract() != contract
                    || field_path.fields().len() != 1
                    || !installed.contains_field(&field_path.fields()[0])
                {
                    return Err(());
                }
                Ok(WorthQueryOperationTouchScope::InitializeRelationField(
                    WorthQueryOperationRelationFieldInitializationScope::new(
                        WorthQueryOperationRelationTouchScope::new(
                            binding.clone(),
                            relation.clone(),
                            from.clone(),
                            to.clone(),
                        ),
                        installed,
                        field_path.clone(),
                    ),
                ))
            }
            WorthQueryPortableOperationTouchScope::CreateEntity { entity, .. } => {
                Ok(WorthQueryOperationTouchScope::CreateEntity(
                    WorthQueryOperationEntityTouchScope::new(binding.clone(), entity.clone()),
                ))
            }
            WorthQueryPortableOperationTouchScope::DeleteEntity { entity, .. } => {
                Ok(WorthQueryOperationTouchScope::DeleteEntity(
                    WorthQueryOperationEntityTouchScope::new(binding.clone(), entity.clone()),
                ))
            }
            WorthQueryPortableOperationTouchScope::WriteField {
                entity,
                contract,
                field_path,
                ..
            } => {
                let installed = catalog.aspect(entity, contract.key().as_str()).ok_or(())?;
                if installed.contract() != contract {
                    return Err(());
                }
                Ok(WorthQueryOperationTouchScope::WriteField(
                    WorthQueryOperationFieldTouchScope::new(
                        binding.clone(),
                        entity.clone(),
                        installed,
                        field_path.clone(),
                    ),
                ))
            }
            WorthQueryPortableOperationTouchScope::LinkRelation {
                relation, from, to, ..
            } => Ok(WorthQueryOperationTouchScope::LinkRelation(
                WorthQueryOperationRelationTouchScope::new(
                    binding.clone(),
                    relation.clone(),
                    from.clone(),
                    to.clone(),
                ),
            )),
            WorthQueryPortableOperationTouchScope::UnlinkRelation {
                relation, from, to, ..
            } => Ok(WorthQueryOperationTouchScope::UnlinkRelation(
                WorthQueryOperationRelationTouchScope::new(
                    binding.clone(),
                    relation.clone(),
                    from.clone(),
                    to.clone(),
                ),
            )),
        })
        .collect::<Result<Vec<_>, ()>>()?;
    let count = scopes.len();
    let contract = if scopes.is_empty() {
        WorthQueryOperationTouchContract::NotRequired
    } else {
        WorthQueryOperationTouchContract::Declared {
            graph_roles: vec!["primary".to_owned()],
            scopes,
        }
    };
    Ok((contract, count))
}

#[cfg(test)]
mod tests;
