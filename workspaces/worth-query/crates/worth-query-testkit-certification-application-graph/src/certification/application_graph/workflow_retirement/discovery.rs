//! A selected branch names the definition it holds current under a workflow
//! identity. A discovered reference grants nothing: a start that names a
//! definition its branch has since superseded or retired is refused with a
//! typed outcome before any effect, and a retry of a recorded start still
//! replays.

pub use worth_query_host::facade::domain::WorthQueryApplicationWorkflowInstallationDenialKind;
pub use worth_query_host::facade::{
    application_discovery::WorthQueryWorkflowDefinitionDiscovery,
    declaration::application_program::ApplicationWorkflowDefinitionIdentity,
    product::WorthQueryProductBranch,
};

pub use super::fork_definitions::fork_of;
pub use super::*;

pub fn discover(
    application: &DocumentWorkflowRuntime,
    branch: WorthQueryProductBranch,
    identity: &ApplicationWorkflowDefinitionIdentity,
) -> Result<
    WorthQueryWorkflowDefinitionDiscovery,
    worth_query_host::facade::domain::WorthQueryApplicationWorkflowInstallationDenialKind,
> {
    super::super::document_retention_model::workflow::discover_definition_metadata(
        application,
        branch,
        identity,
    )
}
