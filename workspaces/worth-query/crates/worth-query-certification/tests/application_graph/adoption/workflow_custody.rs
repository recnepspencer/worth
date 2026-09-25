//! A target cannot silently strand a live assessment obligation at activation.

use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorthQueryApplicationProgramAdoptionPreparationDenial, WorthQueryApplicationRequestExt,
    WorthQueryWorkflowProposalPreparationDenialKind,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
use worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial;

use crate::bounded_dimension_model::{
    host::publish_workflow_on_first_program,
    operator_identity::{authenticate_operator, request_scope},
    presented_request::set_dimension,
    programs::RemovedAssessmentSupplierProgram,
    settled_verdict::{settle, DimensionVerdict},
    workflow::{
        advance_instance, propose_instance, propose_instance_on_branch, publish_definition,
        reviewed_geometry_definition, start_instance,
    },
};

#[test]
fn supplier_removal_requires_owner_disposition_for_a_waiting_instance() {
    let application = publish_workflow_on_first_program();
    let main = application.current_world();
    let sibling = application
        .runtime()
        .branches()
        .fork(main)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("P0 sibling publishes before the instance starts");
    let definition = match publish_definition(
        &application,
        reviewed_geometry_definition("completed"),
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
    let instance =
        match start_instance(&application, definition, 83_001).expect("P0 instance prepares") {
            WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
            other => panic!("P0 instance did not start: {other:?}"),
        };
    propose_instance(&application, instance.clone(), 83_002).expect("proposal publishes");
    assert!(matches!(
        advance_instance(&application, instance.clone(), 83_003),
        Ok(WorkflowProgressOutcome::AwaitingAssessment(_)),
    ));
    let historical_fork = application
        .runtime()
        .branches()
        .fork(main)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("fork retains instance history without executable custody");

    let target = application
        .program_runtime()
        .supported_program::<RemovedAssessmentSupplierProgram>()
        .expect("supplier-removal target is rostered")
        .owned_revision()
        .clone();
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
        .adopt(&target)
        .requirements(&requirements)
        .prepare(1_024)
    {
        Err(denial) => denial,
        Ok(_) => panic!("a waiting instance requires a lawful owner disposition"),
    };
    match denied {
        WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(
            WorthQueryBranchAdoptionPreparationDenial::WorkflowDispositionRequired { instances },
        ) => assert_eq!(instances.as_ref(), &[instance.entity_id()]),
        other => panic!("adoption did not name the exact waiting instance: {other:?}"),
    }
    let fork_programs = runtime
        .request(&principal, &scope)
        .on_branch(historical_fork)
        .programs();
    let fork_requirements = fork_programs
        .compare(&target)
        .expect("historical fork compares its own P0 program");
    match runtime
        .request(&principal, &scope)
        .on_branch(historical_fork)
        .programs()
        .adopt(&target)
        .requirements(&fork_requirements)
        .prepare(1_024)
    {
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
        settle(set_dimension(
            application.program_runtime(),
            main,
            8,
            83_004
        )),
        DimensionVerdict::Performed(8),
        "denied adoption leaves P0 active on main",
    );
    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            sibling,
            8,
            83_005
        )),
        DimensionVerdict::Performed(8),
        "the P0 sibling remains independently callable",
    );
}
