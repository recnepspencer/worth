//! Typed construction of the installed ordinary retention action on its actual branch.
use super::super::{
    authentication::Caller,
    host::*,
    retention,
    schema::{data::RetentionEdit, Schema},
};
use worth_query_decl::facade::application_program::ApplicationSourceCompletion;
use worth_query_host::facade::{application_entry::*, product::WorthQueryProductBranch};

pub type Operation = WorthQuerySourceOnlyOperation<
    Schema,
    RetentionEdit,
    Program,
    RetentionAction,
    ApplicationSourceCompletion,
>;
pub type Completed = WorthQueryCompletedSourceOperation<
    Schema,
    RetentionEdit,
    Program,
    RetentionAction,
    ApplicationSourceCompletion,
>;
pub type Terminal = WorthQueryTerminalSourceOperation<
    Schema,
    RetentionEdit,
    Program,
    RetentionAction,
    ApplicationSourceCompletion,
>;
pub type Completion = WorthQuerySourceCompletion<retention::Changed>;

#[derive(Debug)]
pub enum ConstructionFailure {
    Capture(WorthQueryOperationInputCaptureDenial),
    Action(WorthQueryOperationConstructionDenial),
}
pub fn construct(
    host: &Host,
    branch: WorthQueryProductBranch,
    caller: &Caller<'_>,
    input: &RetentionEdit,
    key: retention::StepKey,
) -> Result<Operation, ConstructionFailure> {
    host.request(&caller.principal, caller.scope)
        .on_branch(branch)
        .mutate(input.clone())
        .without_source()
        .idempotency_owned(key)
        .map_err(|(_, _, cause)| ConstructionFailure::Capture(cause))?
        .retain_operation::<Program, RetentionAction>(host)
        .map_err(|(_, cause)| ConstructionFailure::Action(cause))
}
