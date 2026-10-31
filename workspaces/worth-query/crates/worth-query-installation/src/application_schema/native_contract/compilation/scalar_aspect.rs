//! Explicit scalar storage consumes the declared sole field, never its position.
use super::*;
#[cfg(test)]
mod tests;

pub(super) fn collect(
    schema: &ErasedApplicationSchemaDeclaration,
    aspects: &mut DeclaredAspects,
) -> Result<(), WorthQueryApplicationSchemaContractCatalogDenial> {
    for member in schema.members() {
        let ApplicationSchemaMember::ScalarAspectField {
            entity,
            aspect,
            field,
        } = member
        else {
            continue;
        };
        let seed = aspects
            .get_mut(&(
                ApplicationFieldBindingOwnerKind::Entity,
                entity.clone(),
                aspect.clone(),
            ))
            .ok_or_else(|| denial(DenialKind::FieldWithoutAspect, field))?;
        let key = FieldKey::new(field.as_str())
            .ok_or_else(|| denial(DenialKind::InvalidFieldKey, field))?;
        if seed.scalar_field.replace(key.clone()).is_some()
            || seed.fields.len() != 1
            || !matches!(
                seed.fields.get(&key),
                Some((_, ApplicationFieldPresence::Required, None))
            )
        {
            return Err(denial(DenialKind::InvalidAspectShape, aspect));
        }
    }
    Ok(())
}

pub(super) fn contract(
    aspect: &AspectKey,
    field: &FieldKey,
    seed: &DeclaredAspectSeed,
) -> Result<AspectContract, WorthQueryApplicationSchemaContractCatalogDenial> {
    let Some((family, ApplicationFieldPresence::Required, None)) = seed.fields.get(field) else {
        return Err(denial(DenialKind::InvalidAspectShape, aspect.as_str()));
    };
    Ok(aspects()
        .contract()
        .for_key(aspect.clone())
        .identified_by(seed.identity)
        .at_revision(seed.revision)
        .scalar_with(
            *family,
            AspectMaskContract::scalar(),
            seed.absence,
            AspectEquivalenceBasis::ExactCanonicalValue,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        ))
}
