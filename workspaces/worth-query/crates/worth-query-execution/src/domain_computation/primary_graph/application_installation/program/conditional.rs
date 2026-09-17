use std::any::TypeId;
use std::collections::BTreeSet;

use worth_query_declaration::facade::application_program::ApplicationActionDeclaration;

use super::WorthQueryInMemoryApplicationDenial;

pub(super) fn validate_conditional_actions(
    actions: &[ApplicationActionDeclaration],
    installed_operations: impl Iterator<Item = TypeId>,
) -> Result<(), WorthQueryInMemoryApplicationDenial> {
    let declared = actions
        .iter()
        .filter(|action| action.conditional_only())
        .map(|action| action.operation_type())
        .collect::<BTreeSet<_>>();
    let installed = installed_operations.collect::<BTreeSet<_>>();
    let conflicting_client_action = actions
        .iter()
        .any(|action| !action.conditional_only() && installed.contains(&action.operation_type()));
    if declared == installed && !conflicting_client_action {
        Ok(())
    } else {
        Err(WorthQueryInMemoryApplicationDenial::ConditionalProgramMismatch)
    }
}
