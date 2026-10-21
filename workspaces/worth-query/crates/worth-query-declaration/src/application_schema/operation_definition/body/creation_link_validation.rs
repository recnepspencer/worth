//! Fresh descriptions must retain exact relation endpoints and Link membership.
use crate::application_schema::{
    ApplicationOperationBodyCreation as Creation, ApplicationOperationBodyCreationLink as Link,
    ApplicationOperationBodyCreationLinkDirection as Direction,
    ApplicationOperationBodyCreationLinkPosture as Posture, ApplicationOperationBodyProgram,
    ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember,
};
pub(super) fn validate_links(
    operation: &str,
    body: &ApplicationOperationBodyProgram,
    members: &[ApplicationSchemaMember],
) -> Result<(), Denial> {
    let Some(creation) = body.creation() else {
        return Ok(());
    };
    let mut keys = std::collections::BTreeSet::new();
    for link in creation.links() {
        if link.scope_entity() != body.scope_entity().unwrap_or_default()
            || link.created_entity() != creation.entity()
            || link.key.is_empty()
            || link.key.trim() != link.key
        {
            return Err(Denial::program_body_dependency(operation));
        }
        if !keys.insert((&link.relation, &link.key)) {
            return Err(Denial::DuplicateMember);
        }
        if link.posture == Posture::ReplaceSingle
            && !replacement_is_declared(operation, creation, link, members)
        {
            return Err(Denial::program_body_dependency(operation));
        }
        if !members.iter().any(|member| matches!(member, ApplicationSchemaMember::Relation { relation, from, to, .. }
            if relation == &link.relation && from == &link.from && to == &link.to))
            || !members.iter().any(|member| matches!(member, ApplicationSchemaMember::OperationProgram { operation: candidate, target }
                if candidate == operation && target == &link.program_target())) {
            return Err(Denial::program_body_dependency(operation));
        }
    }
    Ok(())
}

fn replacement_is_declared(
    operation: &str,
    creation: &Creation,
    link: &Link,
    members: &[ApplicationSchemaMember],
) -> bool {
    let Some(unlink) = link.replacement_target() else {
        return false;
    };
    link.direction == Direction::FromScope
        && creation.links().iter().filter(|other| other.relation == link.relation).count() == 1
        && members.iter().any(|member| matches!(member, ApplicationSchemaMember::Relation { relation, integrity, .. }
            if relation == &link.relation && integrity.cardinality.source_max == Some(1)))
        && members.iter().any(|member| matches!(member, ApplicationSchemaMember::OperationProgram { operation: declared, target }
            if declared == operation && target == &unlink))
        && link.replacement_reads().iter().all(|read| members.iter().any(|member| matches!(member,
            ApplicationSchemaMember::OperationDecisionRead { operation: declared, target } if declared == operation && target == read)))
}
