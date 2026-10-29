pub(crate) use worth_query_worlds::facade::certification::application_graph::workflow_publication_scale::*;

#[test]
fn cold_compilation_of_large_definition_uses_bounded_start_facts() {
    const NODES: u16 = 512;
    const CONNECTIONS: u16 = NODES - 1;
    let resources = WorthQueryApplicationWorkflowResourceCeiling::new(
        NODES,
        CONNECTIONS,
        CONNECTIONS,
        COMPONENTS,
        super::document_retention_model::host::WORKFLOW_DEFINITION_BYTES,
        2,
        u32::from(NODES),
        256 * 1024,
    )
    .expect("finite workflow resources");
    let application =
        retain_workflow_with_resources(publish_on_first_program_for_workflow_scale(), resources);
    let runtime = application.runtime();
    let cancellation = worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    let scope =
        worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
            Some(std::time::Instant::now() + std::time::Duration::from_secs(300)),
            cancellation.token(),
        );
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let publication = runtime
        .with_host_advancement(|host_phase| {
            runtime
                .request(&principal, &scope)
                .mutate(WorkflowDefinitionAuthoringIntent {
                    input: WorkflowDefinitionAuthoringInput {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                        retention_days: 8,
                    },
                })
                .workflow(&host_phase, &application, sparse_definition(NODES))
                .expect("large definition validates")
                .publish(WorkflowDefinitionExpectedPredecessor::Absent)
                .idempotency(&918_401)
        })
        .expect("installed host call opens")
        .execute()
        .expect("large definition publishes");
    let WorkflowDefinitionPublicationOutcome::Published(published) = publication else {
        panic!("expected publication, got {publication:?}");
    };
    let before = runtime.workflow_compilation_reuse_counters();
    assert!(matches!(
        start_instance(&application, published.definition().clone(), 918_402)
            .expect("cold start must fit its admission budget"),
        WorkflowInstanceStartOutcome::Started(_)
    ));
    let cold = runtime.workflow_compilation_reuse_counters();
    assert_eq!(cold.cold_misses(), before.cold_misses() + 1);
    assert!(matches!(
        start_instance(&application, published.definition().clone(), 918_403)
            .expect("warm start must reuse compiled meaning"),
        WorkflowInstanceStartOutcome::Started(_)
    ));
    let warm = runtime.workflow_compilation_reuse_counters();
    assert_eq!(warm.warm_hits(), cold.warm_hits() + 1);
    assert_eq!(warm.cold_misses(), cold.cold_misses());
}
