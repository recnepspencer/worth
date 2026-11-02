//! Schema dependency closure of a structurally valid path, without caller policy.
use super::schema_read_closure::{entity_exists, field_matches, relation_exists};
use super::ApplicationSchemaMember;
use crate::application_query::{
    ApplicationQueryRootPathMeaning, ApplicationQueryRootPathSourceKind as SourceKind,
};

pub(super) fn is_closed(
    path: &ApplicationQueryRootPathMeaning,
    members: &[ApplicationSchemaMember],
) -> bool {
    path.is_well_formed()
        && entity_exists(members, path.start_entity())
        && entity_exists(members, path.terminal_entity())
        && path.steps().iter().all(|step| match step.source_kind() {
            SourceKind::Stored => relation_exists(members, step.relation(), step.from(), step.to()),
            SourceKind::Derived => {
                members.iter().any(|member| {
                    matches!(member, ApplicationSchemaMember::DerivedRelation { definition }
                        if definition.parts().relation == step.relation()
                            && &definition.parts().from == step.from_member()
                            && &definition.parts().to == step.to_member())
                }) && super::application_derived_relation_source_closure(members, step.relation())
                    .is_ok_and(|closure| closure.projected_domains.is_empty())
            }
        })
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

#[cfg(test)]
mod tests;
