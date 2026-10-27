//! Match read dependencies against actual schema member meaning.
use super::ApplicationSchemaMember;
use worth_foundational::facade::ScalarAspectType;

pub(super) fn entity_exists(members: &[ApplicationSchemaMember], expected: &str) -> bool {
    members.iter().any(
        |member| matches!(member, ApplicationSchemaMember::Entity { entity } if entity == expected),
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn field_matches(
    members: &[ApplicationSchemaMember],
    expected_entity: &str,
    expected_aspect: &str,
    expected_field: &str,
    expected_scalar: ScalarAspectType,
    expected_value_type: Option<&str>,
    equality_required: bool,
) -> bool {
    members.iter().any(|member| {
        matches!(
            member,
            ApplicationSchemaMember::Field {
                entity,
                aspect,
                field,
                scalar_family,
                value_type,
                equality_queryable,
                ..
            } if entity == expected_entity
                && aspect == expected_aspect
                && field == expected_field
                && *scalar_family == expected_scalar
                && expected_value_type.is_none_or(|expected| value_type == expected)
                && (!equality_required || *equality_queryable)
        )
    })
}

pub(super) fn relation_exists(
    members: &[ApplicationSchemaMember],
    expected_relation: &str,
    expected_from: &str,
    expected_to: &str,
) -> bool {
    members.iter().any(|member| {
        matches!(
            member,
            ApplicationSchemaMember::Relation { relation, from, to, .. }
                if relation == expected_relation
                    && from == expected_from
                    && to == expected_to
        )
    })
}
