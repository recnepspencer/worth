//! Pure declared projection shared with the actual installed contract compiler.
use super::{
    compilation, WorthQueryApplicationSchemaContractCatalogDenial as D,
    WorthQueryApplicationSchemaContractCatalogDenialKind as K,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationFieldBindingOwnerKind, ErasedApplicationSchemaDeclaration,
};

/// Resolve the Native compiler's complete contract and physical binding together.
/// This is declaration meaning, not an installed projection or runtime grant.
pub fn declared_application_aspect(
    declaration: &ErasedApplicationSchemaDeclaration,
    entity: &str,
    aspect: &str,
) -> Result<crate::package::WorthQueryPortableNativeAspectContractRecord, D> {
    let mut aspects = compilation::collect_declared_aspects(declaration)?;
    compilation::collect_declared_fields(declaration, &mut aspects)?;
    let seed = aspects
        .remove(&(
            ApplicationFieldBindingOwnerKind::Entity,
            entity.to_owned(),
            aspect.to_owned(),
        ))
        .ok_or_else(|| D::new(K::FieldWithoutAspect, format!("{entity}:{aspect}")))?;
    compilation::compile_portable_aspect_record(
        declaration.name(),
        ApplicationFieldBindingOwnerKind::Entity,
        entity.to_owned(),
        aspect.to_owned(),
        seed,
    )
}
