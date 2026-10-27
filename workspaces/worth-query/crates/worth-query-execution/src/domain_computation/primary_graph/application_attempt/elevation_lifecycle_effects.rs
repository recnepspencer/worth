use super::effect_program::WorthQueryApplicationRealizedEffect;

pub(super) fn lifecycle_effects_are_exact(
    actual: &[WorthQueryApplicationRealizedEffect],
    expected: &[WorthQueryApplicationRealizedEffect],
) -> bool {
    actual.len() == expected.len()
        && actual
            .iter()
            .zip(expected)
            .all(|(actual, expected)| match (actual, expected) {
                (
                    WorthQueryApplicationRealizedEffect::UpdateEntity {
                        entity,
                        entity_id,
                        fields,
                    },
                    WorthQueryApplicationRealizedEffect::UpdateEntity {
                        entity: expected_entity,
                        entity_id: expected_id,
                        fields: expected_fields,
                    },
                ) => {
                    entity == expected_entity
                        && entity_id == expected_id
                        && fields == expected_fields
                }
                (
                    WorthQueryApplicationRealizedEffect::CreateRelation {
                        fields,
                        kind,
                        key,
                        from,
                        to,
                    },
                    WorthQueryApplicationRealizedEffect::CreateRelation {
                        fields: expected_fields,
                        kind: expected_kind,
                        key: expected_key,
                        from: expected_from,
                        to: expected_to,
                    },
                ) => {
                    kind == expected_kind
                        && key == expected_key
                        && from == expected_from
                        && to == expected_to
                        && fields == expected_fields
                }
                _ => false,
            })
}
