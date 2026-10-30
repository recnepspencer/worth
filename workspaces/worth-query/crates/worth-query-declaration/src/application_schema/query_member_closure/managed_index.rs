//! Resolve descriptive index sources without issuing provider or source authority.
use super::super::{ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember};
use crate::application_query::{
    ApplicationManagedIndexDefinitionDenial as Cause, ApplicationQueryAuthorizationRequirement,
    ApplicationQueryDisclosurePosture, ErasedApplicationManagedIndexDefinition,
    ErasedApplicationQueryDefinition,
};
use std::collections::BTreeMap;

pub(super) fn validate(members: &[ApplicationSchemaMember]) -> Result<(), Denial> {
    let queries: BTreeMap<_, _> = members
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::ApplicationQuery { definition } => {
                Some((definition.name(), definition))
            }
            _ => None,
        })
        .collect();
    let mut indexes = BTreeMap::new();
    for definition in members.iter().filter_map(|member| match member {
        ApplicationSchemaMember::ManagedIndex { definition } => Some(definition),
        _ => None,
    }) {
        let name = &definition.parts().name;
        definition.validate().map_err(|cause| denied(name, cause))?;
        if indexes.insert(name.as_str(), definition).is_some() {
            return Err(denied(name, Cause::DuplicateIndex));
        }
    }
    validate_source_cycles(&indexes, &queries)?;
    for definition in indexes.values() {
        let name = &definition.parts().name;
        let source = queries
            .get(
                definition
                    .parts()
                    .source
                    .binding
                    .parts()
                    .query_name
                    .as_str(),
            )
            .ok_or_else(|| denied(name, Cause::MissingSource))?;
        validate_source(definition, source).map_err(|cause| denied(name, cause))?;
    }
    for parent in queries.values() {
        let Some(selection) = parent.managed_index() else {
            continue;
        };
        let definition = indexes
            .get(selection.parts().index.as_str())
            .ok_or_else(|| denied(&selection.parts().index, Cause::MissingSource))?;
        validate_parent(definition, parent)
            .map_err(|cause| denied(&definition.parts().name, cause))?;
    }
    Ok(())
}

fn validate_source_cycles(
    indexes: &BTreeMap<&str, &ErasedApplicationManagedIndexDefinition>,
    queries: &BTreeMap<&str, &ErasedApplicationQueryDefinition>,
) -> Result<(), Denial> {
    let mut complete = std::collections::BTreeSet::new();
    for name in indexes.keys().copied() {
        let mut path = std::collections::BTreeSet::new();
        let mut current = name;
        while !complete.contains(current) {
            if !path.insert(current) {
                return Err(denied(name, Cause::SourceDependencyCycle));
            }
            let index = indexes
                .get(current)
                .ok_or_else(|| denied(name, Cause::MissingSource))?;
            let source = queries
                .get(index.parts().source.binding.parts().query_name.as_str())
                .ok_or_else(|| denied(name, Cause::MissingSource))?;
            let Some(selection) = source.managed_index() else {
                break;
            };
            current = selection.parts().index.as_str();
        }
        complete.extend(path);
    }
    Ok(())
}

fn validate_source(
    index: &ErasedApplicationManagedIndexDefinition,
    source: &ErasedApplicationQueryDefinition,
) -> Result<(), Cause> {
    let parts = index.parts();
    let binding = parts.source.binding.parts();
    if binding.query.as_str() != source.query_type()
        || binding.result.as_str() != source.result_type()
        || binding.parameters.as_str() != source.parameter_type()
        || binding.scope.as_str() != source.scope_type()
        || binding.scope_entity != source.scope_entity()
        || parts.entity != source.root_entity()
    {
        return Err(Cause::ForeignSourceBinding);
    }
    let disclosure = source.disclosure();
    if source.authorization() != &ApplicationQueryAuthorizationRequirement::Public
        || disclosure.posture() != ApplicationQueryDisclosurePosture::Public
        || disclosure.capability_name().is_some()
        || disclosure.capability_type().is_some()
        || !disclosure.rules().is_empty()
    {
        return Err(Cause::UnsupportedSourcePosture);
    }
    if source.managed_index().is_some() {
        return Err(Cause::UnsupportedSourceShape);
    }
    if !source.parameters().is_empty()
        || source.continuation().is_some()
        || source.live_cause().is_some()
        || source.orbit().is_some()
        || source.component_selection().is_some()
    {
        return Err(Cause::UnsupportedSourceShape);
    }
    Ok(())
}

fn validate_parent(
    index: &ErasedApplicationManagedIndexDefinition,
    parent: &ErasedApplicationQueryDefinition,
) -> Result<(), Cause> {
    let parts = index.parts();
    let selection = parent.managed_index().expect("selected parent");
    let source = parts.source.binding.parts();
    if parent.scope_entity() != source.scope_entity
        || parent.scope_type() != source.scope.as_str()
        || parent.root_entity() != parts.entity
    {
        return Err(Cause::ForeignScope);
    }
    if parent.orbit().is_some()
        || parent.component_selection().is_some()
        || !parent.root_paths().is_empty()
        || parent.continuation().is_some()
        || parent.live_cause().is_some()
    {
        return Err(Cause::ConflictingRootSelection);
    }
    let parameter = parent
        .parameters()
        .iter()
        .find(|parameter| parameter.name() == selection.parts().probe_parameter)
        .ok_or(Cause::ForeignProbe)?;
    let probe = parts.probe.parts();
    if parameter.value_type() != probe.binding.as_str()
        || parameter.scalar_family() != probe.scalar_family
        || parameter.unit() != probe.unit.as_deref()
        || parameter.frame() != probe.frame.as_deref()
    {
        return Err(Cause::ForeignProbe);
    }
    Ok(())
}

fn denied(index: &str, cause: Cause) -> Denial {
    Denial::InvalidManagedIndex {
        index: index.to_owned(),
        cause,
    }
}
