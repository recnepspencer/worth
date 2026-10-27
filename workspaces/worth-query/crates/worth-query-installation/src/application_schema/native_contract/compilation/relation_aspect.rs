use super::{
    denial, validate_aspect_identity, ApplicationFieldBindingOwnerKind, ApplicationSchemaMember,
    AspectIdentity, BTreeMap, DeclaredAspectSeed, DeclaredAspects, DenialKind,
    ErasedApplicationSchemaDeclaration, FieldKey, WorthQueryApplicationSchemaContractCatalogDenial,
};

pub(super) fn collect_relation_aspects(
    schema: &ErasedApplicationSchemaDeclaration,
    aspects: &mut DeclaredAspects,
    identities: &mut BTreeMap<AspectIdentity, (String, String)>,
) -> Result<(), WorthQueryApplicationSchemaContractCatalogDenial> {
    for definition in schema.members().iter().filter_map(|member| match member {
        ApplicationSchemaMember::RelationAspect { definition } => Some(definition),
        _ => None,
    }) {
        let aspect = definition.parts();
        validate_aspect_identity(
            identities,
            &aspect.relation,
            &aspect.aspect,
            aspect.identity,
            aspect.revision,
        )?;
        let mut fields = BTreeMap::new();
        for field in &aspect.fields {
            let key = FieldKey::new(&field.field)
                .ok_or_else(|| denial(DenialKind::InvalidFieldKey, &field.field))?;
            if fields
                .insert(
                    key,
                    (
                        field.scalar_family,
                        field.presence,
                        field.codec_digest(&aspect.relation, &aspect.aspect),
                    ),
                )
                .is_some()
            {
                return Err(denial(DenialKind::DuplicateFieldLocus, &field.field));
            }
        }
        let seed = DeclaredAspectSeed {
            identity: aspect.identity,
            revision: aspect.revision,
            absence: aspect.absence,
            fields,
        };
        if aspects
            .insert(
                (
                    ApplicationFieldBindingOwnerKind::Relation,
                    aspect.relation.clone(),
                    aspect.aspect.clone(),
                ),
                seed,
            )
            .is_some()
        {
            return Err(denial(DenialKind::DuplicateAspectLocus, &aspect.aspect));
        }
    }
    Ok(())
}
