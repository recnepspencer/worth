//! Qualify a typed field witness before touching historical output material.
use super::WorthQueryCorrectionOutputProjectionDenial as Denial;
use std::any::TypeId;
use worth_query_declaration::facade::application_aftermath::{
    ApplicationCorrectionOutputField, DeclaredCorrectionOutputField,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog;

pub(in crate::domain_computation::primary_graph) fn validate_witness<S>(
    source: TypeId,
    manifest: &[DeclaredCorrectionOutputField],
    bindings: &WorthQueryInstalledApplicationValueBindingCatalog,
    field: &ApplicationCorrectionOutputField<S>,
) -> Result<(), Denial> {
    if field.source_binding_type() != source {
        return Err(Denial::ForeignWitness);
    }
    if !manifest.contains(field.declaration()) {
        return Err(Denial::UndeclaredField);
    }
    let parts = field.declaration().parts();
    let installed = bindings
        .field(&parts.entity, &parts.aspect, &parts.field)
        .ok_or(Denial::FieldMeaningMismatch)?;
    let recipe = field.binding_recipe();
    if installed.binding_type() != recipe.binding_type()
        || installed.value_type() != recipe.value_type()
        || installed.identity() != recipe.binding_identity()
        || installed.scalar_family() != recipe.scalar_family()
        || installed.unit() != recipe.unit()
        || installed.frame() != recipe.frame()
        || installed.canonical_contract() != recipe.canonical_contract()
    {
        return Err(Denial::FieldMeaningMismatch);
    }
    Ok(())
}
