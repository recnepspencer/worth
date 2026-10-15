//! A target cannot silently strand a live assessment obligation at activation.
//! The owner inventories it, and only the lawful disposition is offered.

use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorthQueryApplicationProgramAdoptionPreparationDenial, WorthQueryApplicationRequestExt,
    WorthQueryWorkflowProposalPreparationDenialKind,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
use worth_query_host::facade::primary_graph::{
    WorthQueryBranchAdoptionPreparationDenial,
    WorthQueryWorkflowDefinitionDisposition as DefinitionDisposition,
    WorthQueryWorkflowInstanceDisposition as InstanceDisposition,
};

use crate::document_retention_model::{
    host::publish_workflow_on_first_program,
    operator_identity::{authenticate_operator, request_scope},
    presented_request::set_retention,
    programs::RemovedAssessmentSupplierProgram,
    settled_verdict::{settle, RetentionVerdict},
    workflow::{
        advance_instance, propose_instance, propose_instance_on_branch, publish_definition,
        reviewed_document_definition, start_instance,
    },
};

#[test]
fn supplier_removal_requires_owner_disposition_for_a_waiting_instance() {
    let application = publish_workflow_on_first_program();
    let main = application.current_world();
    let sibling = application
        .with_host_advancement(|host_phase| {
            application
                .runtime()
                .branches()
                .fork(main)
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&host_phase)
                .expect("P0 sibling publishes before the instance starts")
        })
        .expect("installed host call opens");
    let definition = match publish_definition(
        &application,
        reviewed_document_definition("completed"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        83_000,
    )
    .expect("P0 definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("P0 definition did not publish: {other:?}"),
    };
    let instance = match start_instance(&application, definition.clone(), 83_001)
        .expect("P0 instance prepares")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("P0 instance did not start: {other:?}"),
    };
    propose_instance(&application, instance.clone(), 83_002).expect("proposal publishes");
    assert!(matches!(
        advance_instance(&application, instance.clone(), 83_003),
        Ok(WorkflowProgressOutcome::AwaitingAssessment(_)),
    ));
    let historical_fork = application
        .with_host_advancement(|host_phase| {
            application
                .runtime()
                .branches()
                .fork(main)
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&host_phase)
                .expect("fork retains instance history without executable custody")
        })
        .expect("installed host call opens");

    let target = *application
        .program_runtime()
        .supported_program::<RemovedAssessmentSupplierProgram>()
        .expect("supplier-removal target is rostered")
        .owned_revision();
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let programs = runtime
        .request(&principal, &scope)
        .on_branch(main)
        .programs();
    let requirements = programs
        .compare(&target)
        .expect("owner compares supplier removal");
    assert!(requirements.semantic_diff().changes().iter().any(|change| {
        change.family()
            == worth_query_host::facade::declaration::application_program::ApplicationSemanticFamily::Operations
            && change.kind()
                == worth_query_host::facade::declaration::application_program::ApplicationSemanticChangeKind::Removed
    }));
    let denied = match runtime
        .request(&principal, &scope)
        .on_branch(main)
        .programs()
        .adopt(&requirements)
        .expect("declared adoption capability admits")
        .prepare(1_024)
    {
        Err(denial) => denial,
        Ok(_) => panic!("a waiting instance requires a lawful owner disposition"),
    };
    assert!(matches!(
        denied,
        WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(
            WorthQueryBranchAdoptionPreparationDenial::WorkflowDispositionRequired
        )
    ));
    let carry = runtime.request(&principal, &scope).on_branch(main).programs()
        .adopt(&requirements).unwrap().workflow(worth_query_host::facade::primary_graph::WorthQueryWorkflowDispositions::carry_compatible()).prepare(1_024);
    assert!(
        matches!(
            carry,
            Err(
                WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(
                    WorthQueryBranchAdoptionPreparationDenial::WorkflowDispositionRejected
                )
            )
        ),
        "native safety refuses incompatible Carry without exposing private occurrences"
    );
    let fork_programs = runtime
        .request(&principal, &scope)
        .on_branch(historical_fork)
        .programs();
    let fork_requirements = fork_programs
        .compare(&target)
        .expect("historical fork compares its own P0 program");
    // The fork holds the copied definition as its own current definition, but
    // the copied instance is historical: only the definition is decided.
    let fork_adoption = runtime
        .request(&principal, &scope)
        .on_branch(historical_fork)
        .programs()
        .adopt(&fork_requirements)
        .expect("declared adoption capability admits");
    let retire = worth_query_host::facade::primary_graph::WorthQueryWorkflowDispositions::new()
        .definition(
            &definition.held_on(historical_fork),
            DefinitionDisposition::Retire,
        );
    match fork_adoption.workflow(retire).prepare(1_024) {
        Ok(_) => {}
        Err(denial) => panic!("copied instance history must not require live custody: {denial:?}"),
    }
    let copied_denial =
        propose_instance_on_branch(&application, historical_fork, instance.clone(), 83_006)
            .expect_err("copied instance history is not executable on the fork");
    assert_eq!(
        copied_denial.kind(),
        WorthQueryWorkflowProposalPreparationDenialKind::InstanceBranchMismatch,
    );
    assert_eq!(
        settle(set_retention(
            application.program_runtime(),
            main,
            8,
            83_004
        )),
        RetentionVerdict::Performed(8),
        "denied adoption leaves P0 active on main",
    );
    assert_eq!(
        settle(set_retention(
            application.program_runtime(),
            sibling,
            8,
            83_005
        )),
        RetentionVerdict::Performed(8),
        "the P0 sibling remains independently callable",
    );
}
