use super::{
    denial, ApplicationWorkflowValidationDenial as Denial,
    ApplicationWorkflowValidationDenialKind as Kind,
};
use crate::application_program::workflow::ApplicationWorkflowGlobalAction;

pub(super) fn validate(actions: &[ApplicationWorkflowGlobalAction]) -> Result<(), Denial> {
    let mut bindings = std::collections::BTreeSet::new();
    for action in actions {
        let operation = action.operation();
        let binding = operation.binding();
        if !binding.is_some_and(|binding| binding.2)
            || operation
                .workflow_execution_requirements()
                .approval()
                .is_some()
        {
            return Err(denial(Kind::InvalidGlobalAction, operation.identifier()));
        }
        if !bindings.insert(binding.expect("guarded binding checked").0) {
            return Err(denial(Kind::DuplicateGlobalAction, operation.identifier()));
        }
    }
    Ok(())
}
