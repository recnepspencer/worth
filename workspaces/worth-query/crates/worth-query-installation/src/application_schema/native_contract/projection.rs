//! Pure declared projection shared with the actual installed contract compiler.
use super::{
    compilation, WorthQueryApplicationSchemaContractCatalogDenial as D,
    WorthQueryApplicationSchemaContractCatalogDenialKind as K,
};
use worth_foundational::facade::{AspectContract, AspectKey};
use worth_query_declaration::facade::application_schema::ErasedApplicationSchemaDeclaration;

/// Resolve the complete aspect, including absence, siblings and codec digests.
/// This describes declaration meaning; it grants no installation authority.
pub fn declared_application_aspect_contract(
    declaration: &ErasedApplicationSchemaDeclaration,
    entity: &str,
    aspect: &str,
) -> Result<AspectContract, D> {
    let mut aspects = compilation::collect_declared_aspects(declaration)?;
    compilation::collect_declared_fields(declaration, &mut aspects)?;
    let seed = aspects
        .get(&(entity.to_owned(), aspect.to_owned()))
        .ok_or_else(|| D::new(K::FieldWithoutAspect, format!("{entity}:{aspect}")))?;
    let key = AspectKey::new(aspect).ok_or_else(|| D::new(K::InvalidAspectKey, aspect))?;
    compilation::build_foundational_contract(&key, aspect, seed)
}
