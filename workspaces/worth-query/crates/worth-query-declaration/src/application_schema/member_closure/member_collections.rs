use std::collections::BTreeSet;

use super::super::ApplicationSchemaMember;

pub(super) fn collect_entities(members: &[ApplicationSchemaMember]) -> BTreeSet<&str> {
    members
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::Entity { entity } => Some(entity.as_str()),
            _ => None,
        })
        .collect()
}

pub(super) fn collect_aspects(members: &[ApplicationSchemaMember]) -> BTreeSet<(&str, &str)> {
    members
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::Aspect { entity, aspect, .. } => {
                Some((entity.as_str(), aspect.as_str()))
            }
            _ => None,
        })
        .collect()
}

pub(super) fn collect_units(members: &[ApplicationSchemaMember]) -> BTreeSet<&str> {
    members
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::Unit { unit } => Some(unit.as_str()),
            _ => None,
        })
        .collect()
}

pub(super) fn collect_operations(members: &[ApplicationSchemaMember]) -> BTreeSet<&str> {
    members
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::Operation { operation, .. } => Some(operation.as_str()),
            _ => None,
        })
        .collect()
}

pub(super) fn collect_abilities(members: &[ApplicationSchemaMember]) -> BTreeSet<(&str, &str)> {
    members
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::Ability {
                ability,
                scope_entity,
            } => Some((ability.as_str(), scope_entity.as_str())),
            _ => None,
        })
        .collect()
}

pub(super) fn collect_policies(members: &[ApplicationSchemaMember]) -> BTreeSet<&str> {
    members
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::Policy { policy } => Some(policy.as_str()),
            _ => None,
        })
        .collect()
}

pub(super) fn collect_principal_entities(members: &[ApplicationSchemaMember]) -> BTreeSet<&str> {
    members
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::PrincipalBinding {
                principal_entity, ..
            } => Some(principal_entity.as_str()),
            _ => None,
        })
        .collect()
}

pub(super) fn collect_fields(members: &[ApplicationSchemaMember]) -> BTreeSet<(&str, &str, &str)> {
    members
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::Field {
                entity,
                aspect,
                field,
                ..
            } => Some((entity.as_str(), aspect.as_str(), field.as_str())),
            _ => None,
        })
        .collect()
}

pub(super) fn collect_relations(
    members: &[ApplicationSchemaMember],
) -> BTreeSet<(&str, &str, &str)> {
    members
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::Relation {
                relation, from, to, ..
            } => Some((relation.as_str(), from.as_str(), to.as_str())),
            _ => None,
        })
        .collect()
}

pub(super) fn collect_effects(members: &[ApplicationSchemaMember]) -> BTreeSet<&str> {
    members
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::Effect { effect, .. } => Some(effect.as_str()),
            _ => None,
        })
        .collect()
}
