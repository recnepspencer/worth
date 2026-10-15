//! Fresh descriptions must retain exact relation endpoints and Link membership.
use crate::application_schema::{
    ApplicationOperationBodyProgram, ApplicationSchemaDeclarationDenial as Denial,
    ApplicationSchemaMember,
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
            return Err(Denial::MissingOperationProgramDependency);
        }
        if !keys.insert((&link.relation, &link.key)) {
            return Err(Denial::DuplicateMember);
        }
        if !members.iter().any(|member| matches!(member, ApplicationSchemaMember::Relation { relation, from, to, .. }
            if relation == &link.relation && from == &link.from && to == &link.to))
            || !members.iter().any(|member| matches!(member, ApplicationSchemaMember::OperationProgram { operation: candidate, target }
                if candidate == operation && target == &link.program_target())) {
            return Err(Denial::MissingOperationProgramDependency);
        }
    }
    Ok(())
}
