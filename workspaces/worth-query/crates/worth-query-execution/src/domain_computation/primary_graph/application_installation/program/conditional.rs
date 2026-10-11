use crate::equality_type_id::EqualityTypeId as TypeId;
use std::collections::HashSet;

use worth_query_declaration::facade::application_program::ApplicationActionDeclaration;

use super::super::WorthQueryInMemoryApplicationDenial;

pub(super) fn validate_conditional_actions(
    actions: &[ApplicationActionDeclaration],
    installed_operations: impl Iterator<Item = TypeId>,
) -> Result<(), WorthQueryInMemoryApplicationDenial> {
    let declared = actions
        .iter()
        .filter(|action| action.conditional_only())
        .map(|action| TypeId::from(action.operation_type()))
        .collect::<HashSet<_>>();
    let installed = installed_operations.collect::<HashSet<_>>();
    let conflicting_client_action = actions.iter().any(|action| {
        !action.conditional_only() && installed.contains(&TypeId::from(action.operation_type()))
    });
    let complete = declared == installed;
    if complete && !conflicting_client_action {
        Ok(())
    } else {
        Err(WorthQueryInMemoryApplicationDenial::ConditionalProgramMismatch)
    }
}
