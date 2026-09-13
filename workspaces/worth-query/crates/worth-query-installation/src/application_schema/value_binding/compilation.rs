use std::collections::BTreeMap;

use worth_query_declaration::facade::application_schema::{
    ApplicationFieldBindingLocus, ApplicationFieldBindingRecipe, ApplicationSchemaBindingIdentity,
    ApplicationSchemaMember, ApplicationSchemaMemberProvenance, ErasedApplicationSchemaDeclaration,
};

use super::denial::{
    WorthQueryApplicationValueBindingInstallationDenial,
    WorthQueryApplicationValueBindingInstallationDenialKind as DenialKind,
};
use super::{
    WorthQueryInstalledApplicationValueBinding, WorthQueryInstalledApplicationValueBindingCatalog,
};
use crate::application_schema::WorthQueryInstalledApplicationSchemaContractCatalog;

pub(crate) fn compile_value_binding_catalog(
    schema_binding: &ApplicationSchemaBindingIdentity,
    declaration: &ErasedApplicationSchemaDeclaration,
    provenance: &ApplicationSchemaMemberProvenance,
    native_contracts: &WorthQueryInstalledApplicationSchemaContractCatalog,
) -> Result<
    WorthQueryInstalledApplicationValueBindingCatalog,
    WorthQueryApplicationValueBindingInstallationDenial,
> {
    let declared = collect_declared_fields(declaration);
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
);

fn collect_declared_fields(
    declaration: &ErasedApplicationSchemaDeclaration,
) -> BTreeMap<ApplicationFieldBindingLocus, DeclaredFieldContract<'_>> {
    declaration
        .members()
        .iter()
        .filter_map(|member| {
            let ApplicationSchemaMember::Field {
                entity,
                aspect,
                field,
                scalar_family,
                value_type,
                unit,
                frame,
                ..
            } = member
            else {
                return None;
            };
            Some((
                ApplicationFieldBindingLocus::new(entity, aspect, field),
                (
                    *scalar_family,
                    value_type.as_str(),
                    unit.as_deref(),
                    frame.as_deref(),
                ),
            ))
        })
        .collect()
}

fn validate_contract(
    locus: &ApplicationFieldBindingLocus,
    declared: DeclaredFieldContract<'_>,
    recipe: &ApplicationFieldBindingRecipe,
    native_contracts: &WorthQueryInstalledApplicationSchemaContractCatalog,
) -> Result<(), WorthQueryApplicationValueBindingInstallationDenial> {
    let (scalar_family, identity, unit, frame) = declared;
    if recipe.scalar_family() != scalar_family
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
