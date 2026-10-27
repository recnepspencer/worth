//! Validate optional meaning against actual compiled bindings before runtime refusal.
use super::{
    binding::WorthQueryCompiledApplicationQueryBinding,
    WorthQueryApplicationQueryInstallationDenial as Denial,
    WorthQueryApplicationQueryInstallationDenialKind as Kind,
};
use crate::application_schema::WorthQueryInstalledApplicationSchema;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaMember,
};

#[cfg(test)]
mod tests;

pub(super) fn validate<Schema: ApplicationSchema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    bindings: &BTreeMap<String, Arc<WorthQueryCompiledApplicationQueryBinding>>,
) -> Result<(), Denial> {
    validate_definitions(schema.installed_declaration().members(), bindings)
}

pub(super) fn validate_definitions(
    members: &[ApplicationSchemaMember],
    bindings: &BTreeMap<String, Arc<WorthQueryCompiledApplicationQueryBinding>>,
) -> Result<(), Denial> {
    let mut seen = BTreeSet::new();
    let mut first = None;
    for member in members {
        let ApplicationSchemaMember::ApplicationQueryInspection { definition } = member else {
            continue;
        };
        let reference = &definition.parts().binding;
        let name = reference.parts().binding.as_str();
        definition
            .validate()
            .map_err(|_| Denial::new(Kind::InvalidInspectionDefinition, name))?;
        if !seen.insert(name) {
            return Err(Denial::new(Kind::InvalidInspectionDefinition, name));
        }
        // This catalog has already checked query/result, scope, principal and limits.
        if !bindings
            .get(name)
            .is_some_and(|compiled| reference.matches_descriptor(compiled.descriptor()))
        {
            return Err(Denial::new(Kind::InspectionBindingMeaningChanged, name));
        }
        first.get_or_insert(name);
    }
    // Never report inert meaning as an installed runtime capability. Validate every
    // definition first so a valid unavailable definition cannot hide a foreign one.
    if let Some(name) = first {
        return Err(Denial::new(Kind::InspectionUnavailable, name));
    }
    Ok(())
}
