//! Authority-free global action metadata readmits only the exact installed binding.
use crate::{
    binary_input::BinaryInput,
    binary_output::BinaryOutput,
    denial::{
        WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
    },
};
use worth_query_declaration::facade::{
    application_operation::ApplicationWorkflowExecutionRequirements,
    application_program::{
        ApplicationWorkflowAuthoringCommand, ApplicationWorkflowGlobalAction,
        ApplicationWorkflowSpec,
    },
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DraftGlobalAction {
    operation: String,
    input_type: String,
    binding: String,
    requirements: ApplicationWorkflowExecutionRequirements,
}

pub(super) fn encode(
    output: &mut BinaryOutput,
    action: &ApplicationWorkflowGlobalAction,
) -> Result<(), Denial> {
    match action {
        ApplicationWorkflowGlobalAction::Terminate(operation) => {
            output.raw_bytes(&[1]);
            for value in [
                operation.identifier(),
                operation.input_type().as_str(),
                operation
                    .binding()
                    .ok_or_else(|| Denial::new(Kind::InvalidRecordShape))?
                    .0,
                &operation
                    .workflow_execution_requirements()
                    .persistence_identity(),
            ] {
                super::records::text(output, value)?;
            }
        }
    }
    Ok(())
}

pub(super) fn decode(input: &mut BinaryInput<'_>) -> Result<DraftGlobalAction, Denial> {
    if input.u8()? != 1 {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    Ok(DraftGlobalAction {
        operation: super::records::nonempty_text(input)?,
        input_type: super::records::nonempty_text(input)?,
        binding: super::records::nonempty_text(input)?,
        requirements: ApplicationWorkflowExecutionRequirements::from_persistence_identity(
            input.text()?,
        )
        .filter(|requirements| requirements.approval().is_none())
        .ok_or_else(|| Denial::new(Kind::InvalidRecordShape))?,
    })
}

pub(super) fn command<Schema: ApplicationSchema, Spec: ApplicationWorkflowSpec<Schema = Schema>>(
    action: &DraftGlobalAction,
    installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
) -> Result<
    ApplicationWorkflowAuthoringCommand,
    super::compatibility::WorthQueryWorkflowDefinitionDraftDenial,
> {
    use super::compatibility::{denial, WorthQueryWorkflowDefinitionDraftDenialKind as Refused};
    let operation = installed
        .draft_operation(&action.operation, Some(&action.binding))
        .ok_or_else(|| denial(Refused::UnknownMember, &action.operation))?;
    if operation.input_type().as_str() != action.input_type
        || operation.workflow_execution_requirements() != &action.requirements
    {
        return Err(denial(Refused::ChangedMember, &action.operation));
    }
    Ok(ApplicationWorkflowAuthoringCommand::termination_action(
        operation,
    ))
}
