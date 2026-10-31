//! Validate metadata meaning against actual compiled bindings before installation.
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
use worth_query_declaration::facade::application_query::ErasedApplicationQueryInspectionDefinition;
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaMember,
};

mod meaning;
#[cfg(test)]
mod tests;
pub(crate) use meaning::CompiledInspectionMeaning;

/// Validate all definitions before constructing any optional installed leaf.
pub(super) fn validate<'a, Schema: ApplicationSchema>(
    schema: &'a WorthQueryInstalledApplicationSchema<Schema>,
    bindings: &BTreeMap<String, Arc<WorthQueryCompiledApplicationQueryBinding>>,
) -> Result<Vec<&'a ErasedApplicationQueryInspectionDefinition>, Denial> {
    validate_all(schema.installed_declaration().members(), bindings)
}

fn validate_all<'a>(
    members: &'a [ApplicationSchemaMember],
    bindings: &BTreeMap<String, Arc<WorthQueryCompiledApplicationQueryBinding>>,
) -> Result<Vec<&'a ErasedApplicationQueryInspectionDefinition>, Denial> {
    let mut seen = BTreeSet::new();
    let mut validated = Vec::new();
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
        validated.push(definition);
    }
    Ok(validated)
}

#[cfg(test)]
fn validate_definitions(
    members: &[ApplicationSchemaMember],
    bindings: &BTreeMap<String, Arc<WorthQueryCompiledApplicationQueryBinding>>,
) -> Result<(), Denial> {
    validate_all(members, bindings).map(|_| ())
}
