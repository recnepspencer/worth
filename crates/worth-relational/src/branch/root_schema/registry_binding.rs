use super::*;

/// Re-admit an older basis only when the available registry carries exactly
/// the same executable kind meaning. A changed schema id/version may be
/// rebound from the envelope; changed names, plans, integrity, or policies
/// require their own historical registry and are denied here.
pub(super) fn registry_for_snapshot(
    available: &RelationalSchemaRegistry,
    expected: &SchemaAuthoritySnapshot,
) -> Option<RelationalSchemaRegistry> {
    // Descriptor rebinding of physical kinds cannot substitute a different
    // executable recipe, even when all physical registrations still match.
    if available.derived_parts() != expected.derived_relations
        || available.projected_domain_parts() != expected.projected_member_domains
    {
        return None;
    }
    if available.authority_snapshot() == *expected {
        return Some(available.clone());
    }
    let observed = available.authority_snapshot();
    let entity_meaning_matches = observed.entity_kinds.len() == expected.entity_kinds.len()
        && observed
            .entity_kinds
            .iter()
            .zip(&expected.entity_kinds)
            .all(|(observed, expected)| {
                observed.kind_id == expected.kind_id
                    && observed.kind_name == expected.kind_name
                    && observed.aspect_plan_revision == expected.aspect_plan_revision
            });
    let relation_meaning_matches = observed.relation_kinds.len() == expected.relation_kinds.len()
        && observed
            .relation_kinds
            .iter()
            .zip(&expected.relation_kinds)
            .all(|(observed, expected)| {
                observed.kind_id == expected.kind_id
                    && observed.kind_name == expected.kind_name
                    && observed.aspect_plan_revision == expected.aspect_plan_revision
                    && observed.relation_integrity_plan_revision
                        == expected.relation_integrity_plan_revision
                    && observed.cross_context_policy == expected.cross_context_policy
                    && observed.cascade_delete_policy == expected.cascade_delete_policy
            });
    if !entity_meaning_matches || !relation_meaning_matches {
        return None;
    }

    let mut rebound = available.clone();
    for expected_kind in &expected.entity_kinds {
        let registration = rebound.entity_kinds.get_mut(&expected_kind.kind_id)?;
        registration.schema_id = expected_kind.schema_id.clone();
        registration.schema_version_id = expected_kind.schema_version_id;
    }
    for expected_kind in &expected.relation_kinds {
        let registration = rebound.relation_kinds.get_mut(&expected_kind.kind_id)?;
        registration.schema_id = expected_kind.schema_id.clone();
        registration.schema_version_id = expected_kind.schema_version_id;
    }
    (rebound.authority_snapshot() == *expected).then_some(rebound)
}
