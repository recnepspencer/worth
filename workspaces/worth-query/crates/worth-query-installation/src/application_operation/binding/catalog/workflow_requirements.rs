use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBindingDescriptor, ApplicationWorkflowExecutionRequirements,
};

use super::{denial, DenialKind, WorthQueryApplicationOperationInstallationDenial};

/// Rejects evidence declarations on bindings that cannot consume native
/// workflow authority, and malformed descriptive contract values.
pub(super) fn validate(
    descriptor: &ApplicationMutationBindingDescriptor,
) -> Result<(), WorthQueryApplicationOperationInstallationDenial> {
    let Some(requirements) = descriptor.workflow_execution_requirements() else {
        return Ok(());
    };
    if !descriptor.requires_workflow_authority()
        || descriptor.workflow_control()
        || ApplicationWorkflowExecutionRequirements::from_persistence_identity(
            &requirements.persistence_identity(),
        )
        .as_ref()
            != Some(requirements)
    {
        return Err(denial(
            DenialKind::MutationBindingMeaningChanged,
            descriptor.identity(),
        ));
    }
    Ok(())
}
