use std::collections::BTreeMap;

use worth_query_declaration::facade::application_schema::{
    ApplicationFieldBindingLocus, ApplicationFieldBindingRecipe, ApplicationFieldValueMeaning,
    ApplicationSchemaBindingIdentity, ApplicationSchemaMember, ApplicationSchemaMemberProvenance,
    ErasedApplicationSchemaDeclaration,
};

use super::denial::{
    WorthQueryApplicationValueBindingInstallationDenial,
    WorthQueryApplicationValueBindingInstallationDenialKind as DenialKind,
};
use super::{
    WorthQueryInstalledApplicationValueBinding, WorthQueryInstalledApplicationValueBindingCatalog,
};
use crate::application_schema::WorthQueryInstalledApplicationSchemaContractCatalog;

#[cfg(test)]
#[path = "compilation_tests.rs"]
mod tests;

pub(crate) fn compile_value_binding_catalog(
    schema_binding: &ApplicationSchemaBindingIdentity,
    declaration: &ErasedApplicationSchemaDeclaration,
    provenance: &ApplicationSchemaMemberProvenance,
    native_contracts: &WorthQueryInstalledApplicationSchemaContractCatalog,
) -> Result<
    WorthQueryInstalledApplicationValueBindingCatalog,
    WorthQueryApplicationValueBindingInstallationDenial,
> {
    let declared = collect_declared_fields(declaration)?;
    let recipes = provenance
        .field_bindings()
        .iter()
        .map(|recipe| (recipe.locus(), recipe))
        .collect::<BTreeMap<_, _>>();

    for locus in declared.keys() {
        let recipe = recipes
            .get(locus)
            .ok_or_else(|| denial(DenialKind::MissingBinding, locus))?;
        validate_contract(locus, declared[locus], recipe, native_contracts)?;
    }
    for locus in recipes.keys() {
        if !declared.contains_key(*locus) {
            return Err(denial(DenialKind::UnexpectedBinding, locus));
        }
    }

    let bindings = recipes
        .into_iter()
        .map(|(locus, recipe)| {
            (
                locus.clone(),
                WorthQueryInstalledApplicationValueBinding::new(
                    schema_binding.clone(),
                    recipe.clone(),
                ),
            )
        })
        .collect();
    Ok(WorthQueryInstalledApplicationValueBindingCatalog::new(
        bindings,
    ))
}

type DeclaredFieldContract<'a> = (
    worth_foundational::facade::ScalarAspectType,
    &'a str,
    Option<&'a str>,
    Option<&'a str>,
    Option<
        &'a worth_query_declaration::facade::application_schema::ApplicationCanonicalValueContract,
    >,
    Option<worth_foundational::facade::CanonicalDigestId>,
    worth_query_declaration::facade::application_schema::ApplicationFieldPresence,
);

fn collect_declared_fields(
    declaration: &ErasedApplicationSchemaDeclaration,
) -> Result<
    BTreeMap<ApplicationFieldBindingLocus, DeclaredFieldContract<'_>>,
    WorthQueryApplicationValueBindingInstallationDenial,
> {
    let mut declared = BTreeMap::new();
    for member in declaration.members() {
        let ApplicationSchemaMember::Field {
            entity,
            aspect,
            field,
            scalar_family,
            value_type,
            unit,
            frame,
            canonical,
            presence,
            ..
        } = member
        else {
            continue;
        };
        let locus = ApplicationFieldBindingLocus::new(entity, aspect, field);
        let codec = if canonical.is_some() {
            Some(
                ApplicationFieldValueMeaning::from_member(member)
                    .map_err(|_| denial(DenialKind::ContractMismatch, &locus))?
                    .and_then(|meaning| meaning.codec_digest())
                    .ok_or_else(|| denial(DenialKind::ContractMismatch, &locus))?,
            )
        } else {
            None
        };
        declared.insert(
            locus,
            (
                *scalar_family,
                value_type.as_str(),
                unit.as_deref(),
                frame.as_deref(),
                canonical.as_ref(),
                codec,
                *presence,
            ),
        );
    }
    Ok(declared)
}

fn validate_contract(
    locus: &ApplicationFieldBindingLocus,
    declared: DeclaredFieldContract<'_>,
    recipe: &ApplicationFieldBindingRecipe,
    native_contracts: &WorthQueryInstalledApplicationSchemaContractCatalog,
) -> Result<(), WorthQueryApplicationValueBindingInstallationDenial> {
    let (scalar_family, identity, unit, frame, canonical, codec, presence) = declared;
    if recipe.scalar_family() != scalar_family
        || recipe.canonical_contract() != canonical
        || (recipe.canonical_contract().is_some()
            && (scalar_family != worth_foundational::facade::ScalarAspectType::String
                || !recipe.supports_admitted_decode()))
        || recipe.binding_identity().as_str() != identity
        || recipe.unit().map(|unit| unit.as_str()) != unit
        || recipe.frame().map(|frame| frame.as_str()) != frame
    {
        return Err(denial(DenialKind::ContractMismatch, locus));
    }
    let native = native_contracts
        .aspect(locus.entity(), locus.aspect())
        .ok_or_else(|| denial(DenialKind::NativeContractMissing, locus))?;
    let contains_field = native.fields().any(|field| field.as_str() == locus.field());
    if !contains_field {
        return Err(denial(DenialKind::NativeContractMissing, locus));
    }
    let worth_foundational::facade::AspectShape::Struct(shape) = native.contract().shape() else {
        return Err(denial(DenialKind::ContractMismatch, locus));
    };
    let native_field = shape
        .fields()
        .iter()
        .find(|field| field.key().as_str() == locus.field())
        .ok_or_else(|| denial(DenialKind::NativeContractMissing, locus))?;
    let requirement = match presence {
        worth_query_declaration::facade::application_schema::ApplicationFieldPresence::Required => {
            worth_foundational::facade::FieldRequirement::Required
        }
        worth_query_declaration::facade::application_schema::ApplicationFieldPresence::Optional => {
            worth_foundational::facade::FieldRequirement::Optional
        }
    };
    if native_field.codec() != codec
        || native_field.value_type() != scalar_family
        || native_field.requirement() != requirement
    {
        return Err(denial(DenialKind::ContractMismatch, locus));
    }
    Ok(())
}

fn denial(
    kind: DenialKind,
    locus: &ApplicationFieldBindingLocus,
) -> WorthQueryApplicationValueBindingInstallationDenial {
    WorthQueryApplicationValueBindingInstallationDenial::new(
        kind,
        format!("{}:{}:{}", locus.entity(), locus.aspect(), locus.field()),
    )
}
