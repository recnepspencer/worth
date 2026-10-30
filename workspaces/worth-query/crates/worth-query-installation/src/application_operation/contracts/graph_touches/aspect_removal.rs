//! Complete same-binding Native field closure is prerequisite to parent removal.
use super::*;
use crate::domain_operation::{
    WorthQueryOperationAspectRemovalScope, WorthQueryOperationGraphReadContract,
    WorthQueryOperationGraphReadScope,
};
use worth_foundational::facade::{AbsenceLaw, AspectContract, AspectShape, CanonicalFieldPath};
pub(super) fn install(
    binding: &ApplicationSchemaBindingIdentity,
    catalog: &WorthQueryInstalledApplicationSchemaContractCatalog,
    entity: &str,
    contract: &AspectContract,
    reads: &WorthQueryOperationGraphReadContract,
) -> Result<WorthQueryOperationTouchScope, ()> {
    let installed = catalog.aspect(entity, contract.key().as_str()).ok_or(())?;
    if installed.contract() != contract || contract.absence() != AbsenceLaw::Optional {
        return Err(());
    }
    let AspectShape::Struct(shape) = contract.shape() else {
        return Err(());
    };
    if shape.fields().is_empty()
        || !shape.fields().iter().all(|field| {
            let path = CanonicalFieldPath::single(field.key().clone());
            reads
                .roles()
                .iter()
                .flat_map(|role| role.read_scopes())
                .any(|read| match read {
                    WorthQueryOperationGraphReadScope::NativeProjection(read) => {
                        read.schema() == binding
                            && read.entity().semantic_key() == entity
                            && read.projection().contract() == contract
                            && (read.projection().mask().is_whole_aspect()
                                || read.projection().mask().paths().contains(&path))
                    }
                    _ => false,
                })
        })
    {
        return Err(());
    }
    Ok(WorthQueryOperationTouchScope::RemoveAspect(
        WorthQueryOperationAspectRemovalScope::new(binding.clone(), entity.into(), installed),
    ))
}
