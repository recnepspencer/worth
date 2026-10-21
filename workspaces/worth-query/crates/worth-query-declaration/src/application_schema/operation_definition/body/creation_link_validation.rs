//! Fresh descriptions must retain exact relation endpoints and Link membership.
use crate::application_schema::{
    ApplicationOperationBodyCreation as Creation, ApplicationOperationBodyCreationLink as Link,
    ApplicationOperationBodyCreationLinkDirection as Direction, ApplicationOperationBodyProgram,
    ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember,
};
pub(super) fn validate_links(
    operation: &str,
    body: &ApplicationOperationBodyProgram,
    members: &[ApplicationSchemaMember],
) -> Result<(), Denial> {
    if !body.has_supported_retiring_replacement() {
        return Err(Denial::program_body_dependency(operation));
    }
    for creation in body.creation().into_iter().flat_map(|c| c.creation_roots()) {
        let mut keys = std::collections::BTreeSet::new();
        for link in creation.links() {
            if !source_is_declared(operation, body, link, members)
                || link.created_entity() != creation.entity()
                || link.key.is_empty()
                || link.key.trim() != link.key
            {
                return Err(Denial::program_body_dependency(operation));
            }
            if !keys.insert((&link.relation, &link.key)) {
                return Err(Denial::DuplicateMember);
            }
            if link.posture.replaces_selection()
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
    }
    Ok(())
}

fn source_is_declared(
    operation: &str,
    body: &ApplicationOperationBodyProgram,
    link: &Link,
    members: &[ApplicationSchemaMember],
) -> bool {
    use crate::application_schema::ApplicationOperationBodyCreationLinkSource as Source;
    match &link.source {
        Source::Scope => link.scope_entity() == body.scope_entity().unwrap_or_default(),
        Source::Principal {mutation_binding, principal_binding, identity} => {
            link.direction == Direction::ToScope && !link.posture.replaces_selection()
                && link.scope_entity() == identity.entity()
                && members.iter().any(|m| matches!(m, ApplicationSchemaMember::ApplicationMutation {description}
                    if description.operation()==operation && description.binding_identity().as_str()==mutation_binding
                    && Some(description.scope().entity.as_str())==body.scope_entity()))
                && members.iter().any(|m| matches!(m, ApplicationSchemaMember::PrincipalBinding {
                    binding, principal_entity, principal_identity_aspect, principal_identity_field, ..}
                    if binding==principal_binding && principal_entity==identity.entity()
                    && principal_identity_aspect==identity.aspect() && principal_identity_field==identity.field()))
                && link.source_reads().iter().all(|read| members.iter().any(|m| matches!(m,
                    ApplicationSchemaMember::OperationDecisionRead {operation:op,target} if op==operation && target==read)))
        }
    }
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
        && (!link.posture.retires_predecessor() || (members.iter().any(|member| matches!(member,ApplicationSchemaMember::Relation {relation,integrity,..} if relation==&link.relation && integrity.cardinality.target_max==Some(1)))
            && members.iter().any(|member|matches!(member,ApplicationSchemaMember::OperationProgram {operation:declared,target} if declared==operation && Some(target)==link.retirement_target().as_ref()))))
        && creation.links().iter().filter(|other| other.relation == link.relation).count() == 1
        && members.iter().any(|member| matches!(member, ApplicationSchemaMember::Relation { relation, integrity, .. }
            if relation == &link.relation && integrity.cardinality.source_max == Some(1)))
        && members.iter().any(|member| matches!(member, ApplicationSchemaMember::OperationProgram { operation: declared, target }
            if declared == operation && target == &unlink))
        && link.replacement_reads().iter().all(|read| members.iter().any(|member| matches!(member,
            ApplicationSchemaMember::OperationDecisionRead { operation: declared, target } if declared == operation && target == read)))
}
