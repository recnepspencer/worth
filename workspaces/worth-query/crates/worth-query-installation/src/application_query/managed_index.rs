//! Bind provider meaning to actual compiled source, scope and probe owners.
use super::{
    binding::WorthQueryCompiledApplicationQueryBinding as Binding,
    WorthQueryApplicationQueryInstallationDenial as Denial,
    WorthQueryApplicationQueryInstallationDenialKind as Kind,
};
use crate::application_schema::WorthQueryInstalledApplicationSchema;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationManagedIndexBindingDescriptor, ApplicationManagedIndexSelectionRecipe,
        ApplicationQueryAuthorizationRequirement, ApplicationQueryDisclosurePosture,
        ErasedApplicationManagedIndexDefinition,
    },
    application_schema::{ApplicationSchema, ApplicationSchemaMember},
};
mod access;
mod compiled;
pub use access::{
    WorthQueryInstalledManagedIndexAccessContract, WorthQueryInstalledManagedIndexReadMeaning,
};
mod installed;
pub(crate) use compiled::CompiledManagedIndex;
pub use installed::{WorthQueryInstalledManagedIndex, WorthQueryInstalledManagedIndexParent};

#[cfg(test)]
mod tests;

pub(super) fn compile<S: ApplicationSchema>(
    schema: &WorthQueryInstalledApplicationSchema<S>,
    bindings: &BTreeMap<String, Arc<Binding>>,
) -> Result<BTreeMap<String, Arc<CompiledManagedIndex>>, Denial> {
    let definitions = validated_definitions(
        schema.installed_declaration().members(),
        schema.member_provenance.managed_index_bindings(),
        schema.member_provenance.managed_index_selections(),
        bindings,
    )?;
    definitions
        .into_iter()
        .map(|(name, definition)| {
            Ok((
                name,
                Arc::new(definition.install(schema.binding_identity())?),
            ))
        })
        .collect()
}

#[cfg(test)]
fn validate_definitions(
    members: &[ApplicationSchemaMember],
    recipes: &[ApplicationManagedIndexBindingDescriptor],
    selections: &[ApplicationManagedIndexSelectionRecipe],
    bindings: &BTreeMap<String, Arc<Binding>>,
) -> Result<(), Denial> {
    validated_definitions(members, recipes, selections, bindings).map(|_| ())
}

fn validated_definitions(
    members: &[ApplicationSchemaMember],
    recipes: &[ApplicationManagedIndexBindingDescriptor],
    selections: &[ApplicationManagedIndexSelectionRecipe],
    bindings: &BTreeMap<String, Arc<Binding>>,
) -> Result<BTreeMap<String, compiled::ValidatedManagedIndex>, Denial> {
    let mut names = BTreeSet::new();
    let mut definitions = BTreeMap::new();
    for member in members {
        let ApplicationSchemaMember::ManagedIndex { definition } = member else {
            continue;
        };
        let name = definition.parts().name.as_str();
        definition
            .validate()
            .map_err(|_| denied(Kind::InvalidManagedIndexDefinition, name))?;
        if !names.insert(name) {
            return Err(denied(Kind::InvalidManagedIndexDefinition, name));
        }
        let mut actual = recipes
            .iter()
            .filter(|recipe| recipe.meaning().parts().name == name);
        let recipe = actual
            .next()
            .filter(|recipe| recipe.meaning() == definition)
            .ok_or_else(|| denied(Kind::ManagedIndexBindingMeaningChanged, name))?;
        if actual.next().is_some() {
            return Err(denied(Kind::ManagedIndexBindingMeaningChanged, name));
        }
        let source = bindings
            .get(recipe.source().identity())
            .filter(|source| {
                source.descriptor() == recipe.source()
                    && definition
                        .parts()
                        .source
                        .binding
                        .matches_descriptor(source.descriptor())
            })
            .ok_or_else(|| denied(Kind::ManagedIndexBindingMeaningChanged, name))?;
        validate_source(definition, source)?;
        let mut parents = BTreeMap::new();
        for parent in bindings.values().filter(|parent| {
            parent
                .query()
                .definition()
                .managed_index()
                .is_some_and(|selection| selection.parts().index == name)
        }) {
            validate_parent(definition, recipe, selections, source, parent)?;
            parents.insert(
                parent.descriptor().identity().to_owned(),
                Arc::clone(parent),
            );
        }
        definitions.insert(
            name.to_owned(),
            compiled::ValidatedManagedIndex {
                recipe: recipe.clone(),
                source: Arc::clone(source),
                parents,
            },
        );
    }
    Ok(definitions)
}

fn validate_source(
    index: &ErasedApplicationManagedIndexDefinition,
    source: &Binding,
) -> Result<(), Denial> {
    let name = &index.parts().name;
    let query = source.query();
    let definition = query.definition();
    let disclosure = definition.disclosure();
    if definition.authorization() != &ApplicationQueryAuthorizationRequirement::Public
        || disclosure.posture() != ApplicationQueryDisclosurePosture::Public
        || disclosure.capability_name().is_some()
        || disclosure.capability_type().is_some()
        || !disclosure.rules().is_empty()
        || !definition.parameters().is_empty()
        || definition.managed_index().is_some()
        || definition.continuation().is_some()
        || definition.live_cause().is_some()
        || definition.orbit().is_some()
        || definition.component_selection().is_some()
    {
        return Err(denied(Kind::ManagedIndexSourceUnsupported, name));
    }
    if definition.root_entity() != index.parts().entity {
        return Err(denied(Kind::ManagedIndexBindingMeaningChanged, name));
    }
    Ok(())
}

fn validate_parent(
    index: &ErasedApplicationManagedIndexDefinition,
    recipe: &ApplicationManagedIndexBindingDescriptor,
    selections: &[ApplicationManagedIndexSelectionRecipe],
    source: &Binding,
    parent: &Binding,
) -> Result<(), Denial> {
    let name = &index.parts().name;
    let source = source.descriptor();
    let descriptor = parent.descriptor();
    if descriptor.scope_identity() != source.scope_identity()
        || descriptor.scope() != source.scope()
        || descriptor.principal() != source.principal()
    {
        return Err(denied(Kind::ManagedIndexBindingMeaningChanged, name));
    }
    let query = parent.query();
    let definition = query.definition();
    let selection = definition
        .managed_index()
        .expect("selected installed parent");
    let mut exact = selections.iter().filter(|selection| {
        selection.matches_parent(
            descriptor,
            definition.managed_index().expect("selected parent"),
        )
    });
    if !exact
        .next()
        .is_some_and(|selection| recipe.admits_selection(selection))
        || exact.next().is_some()
    {
        return Err(denied(Kind::ManagedIndexBindingMeaningChanged, name));
    }
    let probe = index.parts().probe.parts();
    if definition.root_entity() != index.parts().entity
        || !definition.parameters().iter().any(|parameter| {
            parameter.name() == selection.parts().probe_parameter
                && parameter.value_type() == probe.binding.as_str()
                && parameter.scalar_family() == probe.scalar_family
                && parameter.unit() == probe.unit.as_deref()
                && parameter.frame() == probe.frame.as_deref()
        })
    {
        return Err(denied(Kind::ManagedIndexBindingMeaningChanged, name));
    }
    let disclosure = definition.disclosure();
    if definition.authorization() != &ApplicationQueryAuthorizationRequirement::Public
        || disclosure.posture() != ApplicationQueryDisclosurePosture::Public
        || disclosure.capability_name().is_some()
        || disclosure.capability_type().is_some()
        || !disclosure.rules().is_empty()
    {
        return Err(denied(Kind::ManagedIndexParentUnsupported, name));
    }
    Ok(())
}

fn denied(kind: Kind, name: &str) -> Denial {
    Denial::new(kind, name)
}
