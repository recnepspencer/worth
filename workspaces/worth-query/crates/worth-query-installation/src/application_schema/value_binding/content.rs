//! Typed content recipes belong to the same installed field catalogue.
use super::denial::{
    WorthQueryApplicationValueBindingInstallationDenial as Denial,
    WorthQueryApplicationValueBindingInstallationDenialKind as Kind,
};
use std::{collections::BTreeMap, sync::Arc};
use worth_foundational::facade::CanonicalDigestId;
use worth_query_declaration::facade::application_schema::{
    ApplicationContentBindingRecipe, ApplicationFieldBindingLocus, ApplicationFieldValueMeaning,
    ApplicationSchemaMember as Member, ApplicationSchemaMemberProvenance,
};

#[derive(Clone, Debug)]
pub(super) struct InstalledContentField {
    pub(super) format: Arc<ApplicationContentBindingRecipe>,
    pub(super) codec: CanonicalDigestId,
}
pub(super) type Formats = BTreeMap<[u8; 32], Arc<ApplicationContentBindingRecipe>>;
pub(super) type Fields = BTreeMap<ApplicationFieldBindingLocus, InstalledContentField>;
pub(super) fn compile(
    members: &[Member],
    provenance: &ApplicationSchemaMemberProvenance,
) -> Result<(Formats, Fields), Denial> {
    let mut declared = BTreeMap::new();
    for member in members {
        if let Member::ContentFormat { declaration } = member
            && declared.insert(declaration.identity, declaration).is_some()
        {
            return Err(Denial::new(Kind::DuplicateContentFormat, &declaration.name));
        }
    }
    let mut formats = Formats::new();
    for recipe in provenance.content_bindings() {
        let declaration = recipe.declaration();
        if formats
            .insert(declaration.identity, Arc::new(recipe.clone()))
            .is_some()
        {
            return Err(Denial::new(Kind::DuplicateContentFormat, &declaration.name));
        }
        if declared.get(&declaration.identity).copied() != Some(declaration) {
            return Err(Denial::new(Kind::ContentFormatMismatch, &declaration.name));
        }
    }
    for declaration in declared.values() {
        if !formats.contains_key(&declaration.identity) {
            return Err(Denial::new(Kind::MissingContentBinding, &declaration.name));
        }
    }
    let mut fields = Fields::new();
    let mut field_recipes = BTreeMap::new();
    for (locus, recipe) in provenance.content_field_bindings() {
        if field_recipes.insert(locus, recipe).is_some() {
            return Err(Denial::new(Kind::ContentFormatMismatch, locus.field()));
        }
    }
    for member in members {
        let Member::Field {
            entity,
            aspect,
            field,
            content_format: Some(identity),
            ..
        } = member
        else {
            continue;
        };
        let locus = ApplicationFieldBindingLocus::new(entity, aspect, field);
        let format = formats
            .get(identity)
            .ok_or_else(|| Denial::new(Kind::MissingContentBinding, field))?;
        let field_recipe = field_recipes
            .remove(&locus)
            .ok_or_else(|| Denial::new(Kind::MissingContentBinding, field))?;
        // Full descriptor AND actual binding/value types, never callback addresses.
        if field_recipe != format.as_ref() {
            return Err(Denial::new(Kind::ContentFormatMismatch, field));
        }
        let primitive = provenance
            .field_binding(&locus)
            .ok_or_else(|| Denial::new(Kind::MissingBinding, field))?;
        if !is_content_reference(primitive) {
            return Err(Denial::new(Kind::ContentFormatMismatch, field));
        }
        let codec = ApplicationFieldValueMeaning::from_member(member)
            .map_err(|_| Denial::new(Kind::ContractMismatch, field))?
            .and_then(|meaning| meaning.codec_digest())
            .ok_or_else(|| Denial::new(Kind::ContractMismatch, field))?;
        fields.insert(
            locus,
            InstalledContentField {
                format: format.clone(),
                codec,
            },
        );
    }
    if let Some((locus, _)) = field_recipes.first_key_value() {
        return Err(Denial::new(Kind::ContentFormatMismatch, locus.field()));
    }
    Ok((formats, fields))
}

fn is_content_reference(
    recipe: &worth_query_declaration::facade::application_schema::ApplicationFieldBindingRecipe,
) -> bool {
    recipe.scalar_family() == worth_foundational::facade::ScalarAspectType::ContentRef
        && recipe.canonical_contract().is_none()
}
