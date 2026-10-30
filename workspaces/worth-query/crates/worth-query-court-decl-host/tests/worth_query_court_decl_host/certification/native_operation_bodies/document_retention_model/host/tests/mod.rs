pub(crate) use worth_query_testkit_document_retention::facade::document_retention_model::host::tests::*;

#[test]
fn guarded_binding_cannot_install_without_a_program_owner() {
    let result =
        worth_query_host::facade::application_installation::in_memory::<DocumentRetentionSchema>(
            DocumentRetentionSchema::declaration().expect("the fixture schema is valid"),
            ((),),
            ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
            host_limits(),
            |_, _, _| Ok(()),
        );
    assert!(matches!(
        result,
        Err(WorthQueryInMemoryApplicationDenial::WorkflowAuthorityRequiresProgram)
    ));
}

#[test]
fn a_workflow_control_binding_refuses_a_handler() {
    let result = in_memory_rostered_program(
        validated_first_program(),
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
        DocumentRetentionSchema::declaration().expect("the document-retention schema is valid"),
        ((),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(),
        |phase, graph, installed| {
            seed_host(phase, graph, installed)?;
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
        .advanced()
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&0x9176_d000_u64)
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
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

pub(crate) mod workflow_registration;
