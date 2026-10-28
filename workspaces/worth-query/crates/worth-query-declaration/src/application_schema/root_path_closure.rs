//! Schema dependency closure of a structurally valid path, without caller policy.
use super::schema_read_closure::{entity_exists, field_matches, relation_exists};
use super::ApplicationSchemaMember;
use crate::application_query::ApplicationQueryRootPathMeaning;

pub(super) fn is_closed(
    path: &ApplicationQueryRootPathMeaning,
    members: &[ApplicationSchemaMember],
) -> bool {
    path.is_well_formed()
        && entity_exists(members, path.start_entity())
        && entity_exists(members, path.terminal_entity())
        && path
            .steps()
            .iter()
            .all(|step| relation_exists(members, step.relation(), step.from(), step.to()))
        && path.guards().iter().all(|guard| {
            field_matches(
                members,
                guard.entity(),
                guard.aspect(),
                guard.field(),
                guard.scalar_family(),
                Some(guard.value_type()),
                true,
            )
        })
}
