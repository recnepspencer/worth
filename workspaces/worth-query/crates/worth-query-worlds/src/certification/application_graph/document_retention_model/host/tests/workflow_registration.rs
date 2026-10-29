//! A refused signing owner must not license program adoption through stray vocabulary coverage.
pub use super::super::super::workflow::{
    install_certification_authentication_with_clock, install_workflow_spec,
    prepare_second_program_adoption, publish_definition, terminal_definition, workflow_resources,
};
pub use super::super::*;
pub use worth_query_host::facade::{
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorthQueryApplicationProgramAdoptionPreparationDenial,
    },
    application_installation::{
        WorthQuerySigningOwnerInstallationDenial, WorthQueryWorkflowRuntimeBindingDenial,
    },
    primary_graph::{WorthQueryBranchAdoptionPreparationDenial, WorthQueryWorkflowDispositions},
};

pub fn assert_missing_coverage(application: &DocumentWorkflowRuntime) {
    assert!(
        matches!(
            prepare_second_program_adoption(
                application,
                application.current_world(),
                Some(WorthQueryWorkflowDispositions::carry_compatible())
            ),
            Err(
                WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(
                    WorthQueryBranchAdoptionPreparationDenial::WorkflowDispositionRejected
                )
            )
        ),
        "a real source definition cannot carry into a target without installed vocabulary"
    );
}
