use std::collections::BTreeSet;

use crate::application_query::{
    ApplicationHistoryTarget, ApplicationQueryResultShape, ErasedApplicationHistoryQueryDefinition,
};
use crate::application_schema::{
    ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember as Member,
};

pub(super) fn validate(members: &[Member]) -> Result<(), Denial> {
    let mut names = BTreeSet::new();
    for definition in members.iter().filter_map(|member| match member {
        Member::ApplicationHistoryQuery { definition } => Some(definition),
        _ => None,
    }) {
        let parts = definition.parts();
        if !names.insert(parts.name.as_str()) {
            return Err(Denial::DuplicateApplicationHistoryQuery);
        }
        definition
            .validate()
            .map_err(|_| Denial::InvalidApplicationHistoryQuery)?;
        validate_subject_query(definition, members)?;
        validate_aspects(definition, members)?;
        let disclosure = &parts.metadata_disclosure;
        if let (Some(name), Some(identity)) =
            (disclosure.capability_name(), disclosure.capability_type())
            && !members.iter().any(|member| {
                matches!(member, Member::ApplicationCapability { contract }
                if contract.name() == name && contract.capability_type() == identity)
            })
        {
            return Err(Denial::MissingApplicationHistoryDependency);
        }
    }
    Ok(())
}

fn validate_subject_query(
    definition: &ErasedApplicationHistoryQueryDefinition,
    members: &[Member],
) -> Result<(), Denial> {
    let parts = definition.parts();
    let reference = parts.subject_read.parts();
    let query = members
        .iter()
        .find_map(|member| match member {
            Member::ApplicationQuery { definition }
                if definition.name() == reference.query_name =>
            {
                Some(definition)
            }
            _ => None,
        })
        .ok_or(Denial::MissingApplicationHistoryDependency)?;
    if query.query_type() != reference.query.as_str()
        || query.parameter_type() != reference.parameters.as_str()
        || query.result_type() != reference.result.as_str()
        || query.scope_entity() != reference.scope_entity
        || query.scope_type() != reference.scope.as_str()
    {
        return Err(Denial::MissingApplicationHistoryDependency);
    }
    match &parts.target {
        ApplicationHistoryTarget::Entity { entity } if query.root_entity() == entity => Ok(()),
        ApplicationHistoryTarget::Relation { .. }
            if relation_slot_exists(query.result_shape(), &parts.target) =>
        {
            Ok(())
        }
        _ => Err(Denial::MissingApplicationHistoryDependency),
    }
}

fn relation_slot_exists(
    shape: &ApplicationQueryResultShape,
    target: &ApplicationHistoryTarget,
) -> bool {
    shape.relations().iter().any(|relation| {
        target
            .relation_slot_key()
            .is_some_and(|slot| relation.slot_key() == slot)
            || relation_slot_exists(relation.nested_shape(), target)
    })
}

fn validate_aspects(
    definition: &ErasedApplicationHistoryQueryDefinition,
    members: &[Member],
) -> Result<(), Denial> {
    let parts = definition.parts();
    for selected in &parts.aspects {
        let found = members.iter().any(|member| match (member, &parts.target) {
            (
                Member::Aspect {
                    entity,
                    aspect,
                    identity,
                    revision,
                    ..
                },
                ApplicationHistoryTarget::Entity { entity: target },
            ) => {
                entity == target
                    && aspect == &selected.aspect
                    && identity == &selected.identity
                    && revision == &selected.revision
            }
            (
                Member::RelationAspect { definition },
                ApplicationHistoryTarget::Relation {
                    relation, from, to, ..
                },
            ) => {
                let aspect = definition.parts();
                &aspect.relation == relation
                    && &aspect.from == from
                    && &aspect.to == to
                    && aspect.aspect == selected.aspect
                    && aspect.identity == selected.identity
                    && aspect.revision == selected.revision
            }
            _ => false,
        });
        if !found {
            return Err(Denial::MissingApplicationHistoryDependency);
        }
    }
    Ok(())
}
