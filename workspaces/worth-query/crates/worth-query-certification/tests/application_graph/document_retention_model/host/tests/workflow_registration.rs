//! A refused signing owner must not license program adoption through stray vocabulary coverage.
use super::super::*;
use crate::document_retention_model::workflow::{
    install_certification_authentication_with_clock, install_workflow_spec,
    prepare_second_program_adoption, publish_definition, terminal_definition, workflow_resources,
};
use worth_query_host::facade::{
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorthQueryApplicationProgramAdoptionPreparationDenial,
    },
    application_installation::{
        WorthQuerySigningOwnerInstallationDenial, WorthQueryWorkflowRuntimeBindingDenial,
    },
    primary_graph::{WorthQueryBranchAdoptionPreparationDenial, WorthQueryWorkflowDispositions},
};

#[test]
fn signing_owner_registration_and_workflow_coverage_install_together() {
    for conflicting_owner in [false, true] {
        let mut program = publish_on_first_program();
        let target = *program
            .supported_program::<RetentionProgramP1>()
            .unwrap()
            .installed_program()
            .revision();
        let (authentication, authentication_clock) =
            install_certification_authentication_with_clock(program.installed_schema());
        let (other, _) =
            install_certification_authentication_with_clock(program.installed_schema());
        program
            .install_signing_owner(
                target,
                if conflicting_owner {
                    other.signing_owner()
                } else {
                    authentication.signing_owner()
                },
            )
            .unwrap();
        let source = install_workflow_spec(
            program.installed_schema(),
            program.installed_program(),
            workflow_resources(),
        );
        let native = program
            .retain_workflow_spec(source, authentication.signing_owner())
            .unwrap();
        let mut application = DocumentWorkflowRuntime {
            workflow: native,
            authentication,
            authentication_clock,
        };
        let published = publish_definition(
            &application,
            terminal_definition("done"),
            WorkflowDefinitionExpectedPredecessor::Absent,
            92_000,
        )
        .unwrap();
        assert!(matches!(
            published,
            WorkflowDefinitionPublicationOutcome::Published(_)
        ));
        assert_missing_coverage(&application);
        let target_spec = install_workflow_spec(
            application.installed_schema(),
            application
                .supported_program::<RetentionProgramP1>()
                .unwrap()
                .installed_program(),
            workflow_resources(),
        );
        let registered = application
            .workflow_mut()
            .support_workflow_spec(target_spec);
        if conflicting_owner {
            let (_, denial) = registered.unwrap_err();
            assert_eq!(
                denial,
                WorthQueryWorkflowRuntimeBindingDenial::SigningOwner(
                    WorthQuerySigningOwnerInstallationDenial::AlreadyInstalled
                )
            );
            assert_missing_coverage(&application);
        } else {
            registered.unwrap_or_else(|(_, denial)| panic!("identical signing owner: {denial:?}"));
            assert!(
                prepare_second_program_adoption(
                    &application,
                    application.current_world(),
                    Some(WorthQueryWorkflowDispositions::carry_compatible())
                )
                .is_ok(),
                "same-owner registration installs actual executable target coverage"
            );
        }
    }
}

fn assert_missing_coverage(application: &DocumentWorkflowRuntime) {
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
