use super::*;
use declaration::application_operation::ApplicationMutationBinding;

use super::super::retention_entry::DOCUMENT_IDENTITY;
use super::super::schema::{Document, DocumentRetentionSchema};
use super::super::workflow::{WorkflowAdvanceBinding, WorkflowAdvanceInput, WorkflowAdvanceIntent};

type WorkflowAdvanceAccepted =
    <WorkflowAdvanceBinding as ApplicationMutationBinding<DocumentRetentionSchema>>::Result;
type WorkflowAdvanceDenial =
    <WorkflowAdvanceBinding as ApplicationMutationBinding<DocumentRetentionSchema>>::Denial;

#[test]
fn guarded_binding_cannot_install_without_a_program_owner() {
    let result =
        worth_query_host::facade::application_installation::in_memory::<DocumentRetentionSchema>(
            DocumentRetentionSchema::declaration().expect("the fixture schema is valid"),
            ((),),
            host_limits(),
            |_, _| Ok(()),
        );
    assert!(matches!(
        result,
        Err(WorthQueryInMemoryApplicationDenial::WorkflowAuthorityRequiresProgram)
    ));
}

/// A handler an author might wrongly write for a kernel-recorded control step.
struct StrayAdvanceHandler;

impl primary_graph::OperationHandler<DocumentRetentionSchema, WorkflowAdvanceBinding>
    for StrayAdvanceHandler
{
    fn decide(
        &self,
        _: &WorkflowAdvanceInput,
        _: &mut primary_graph::DecisionReader<
            '_,
            '_,
            '_,
            DocumentRetentionSchema,
            WorkflowAdvanceBinding,
        >,
    ) -> primary_graph::HandlerResult<
        primary_graph::WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>,
        WorkflowAdvanceDenial,
    > {
        primary_graph::HandlerResult::Cancelled
    }

    fn candidate_requirements(
        &self,
        _: &WorkflowAdvanceInput,
        _: &primary_graph::WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>,
    ) -> declaration::application_operation::ApplicationCandidateRequirements {
        WorkflowAdvanceBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        _: &WorkflowAdvanceInput,
        _: primary_graph::WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>,
        _: &mut primary_graph::CandidateWriter<'_, DocumentRetentionSchema, WorkflowAdvanceBinding>,
    ) -> primary_graph::HandlerResult<WorkflowAdvanceAccepted, WorkflowAdvanceDenial> {
        primary_graph::HandlerResult::Cancelled
    }
}

#[test]
fn a_workflow_control_binding_refuses_a_handler() {
    let result = in_memory_rostered_program(
        validated_first_program(),
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
        DocumentRetentionSchema::declaration().expect("the document-retention schema is valid"),
        ((),),
        host_limits(),
        |graph, installed| {
            seed_host(graph, installed)?;
            let binding = installed
                .installed_mutation_binding::<WorkflowAdvanceBinding>()
                .expect("the advance binding is installed");
            graph.install_handler(&binding, StrayAdvanceHandler)
        },
    );
    match result {
        Err(WorthQueryInMemoryApplicationDenial::InitialState(denial)) => assert_eq!(
            denial.kind(),
            primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::WorkflowControlHandler
        ),
        Err(other) => panic!("the stray handler was refused for the wrong reason: {other:?}"),
        Ok(_) => panic!("a handler for a kernel-recorded control binding must be refused"),
    }
}

#[test]
fn the_ordinary_lane_refuses_a_workflow_control_binding() {
    use worth_query_host::facade::application_entry::{
        WorthQueryApplicationRequestExt, WorthQueryApplicationRequestMutationDenial,
    };

    let host = publish_on_first_program();
    let runtime = host.runtime();
    let scope = super::super::operator_identity::request_scope();
    let principal =
        super::super::operator_identity::authenticate_operator(runtime.installed_schema(), &scope);
    let settlement = runtime
        .request(&principal, &scope)
        .on_branch(host.current_world())
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&0x9176_d000_u64)
        .execute_in_program(&host);
    assert!(
        matches!(
            settlement,
            Err(WorthQueryApplicationRequestMutationDenial::Handler(
                primary_graph::MutationHandlerExecutionDenial::WorkflowControl
            ))
        ),
        "the kernel records control steps, so the ordinary lane names that: {settlement:?}"
    );
}
