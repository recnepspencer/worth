pub(crate) use worth_query_worlds::facade::certification::application_graph::workflow_publication_scale::*;

#[test]
#[ignore = "scheduled public 10k-node publication qualification"]
fn public_ten_thousand_node_definition_publishes() {
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
    .expect("finite workflow-scale workflow resources");
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
    let cost_scope = application
        .with_host_advancement(|host_phase| {
            runtime
                .capture_certification_cost_scope(&host_phase, application.current_world())
                .expect("document branch remains measurable")
        })
        .expect("installed host call opens");
    let started_at = std::time::Instant::now();
    let authored = sparse_definition(NODES);
    let authored_ms = started_at.elapsed().as_millis();
    let draft = runtime
        .with_host_advancement(|host_phase| {
            runtime
                .request(&principal, &scope)
                .mutate(WorkflowDefinitionAuthoringIntent {
                    input: WorkflowDefinitionAuthoringInput {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                        retention_days: 8,
                    },
                })
                .workflow(&host_phase, &application, authored)
                .expect("the 10k definition validates against installed vocabulary")
        })
        .expect("installed host call opens");
    let validated_ms = started_at.elapsed().as_millis();
    let publication = draft
        .publish(WorkflowDefinitionExpectedPredecessor::Absent)
        .idempotency(&918_400)
        .execute()
        .expect("the 10k definition prepares for publication");
    let cost = application
        .with_host_advancement(|host_phase| {
            runtime
                .observe_certification_cost(&host_phase, &cost_scope)
                .expect("document publication cost remains measurable")
        })
        .expect("installed host call opens");
    let sharing = cost.relational().sharing_cost_delta();
    assert_eq!(sharing.copied_truth_bytes, 0);
    assert!(sharing.publication_new_authoritative_bytes > 0);
    assert!(
        sharing.publication_new_authoritative_bytes <= u64::from(NODES) * 32 * 1024,
        "10k document publication must retain proportional authoritative bytes"
    );
    assert!(
        sharing.publication_content_values_hashed <= u64::from(NODES) * 32,
        "10k document publication must hash proportional content"
    );
    println!(
        "document publication authored_ms={authored_ms} validated_ms={validated_ms} elapsed_ms={} application_work={:?} copied_truth_bytes={} new_authoritative_bytes={} content_values_hashed={} touched_regions={} reused_regions={}",
        started_at.elapsed().as_millis(),
        cost.application_work(),
        sharing.copied_truth_bytes,
        sharing.publication_new_authoritative_bytes,
        sharing.publication_content_values_hashed,
        sharing.publication_touched_region_count,
        sharing.publication_reused_region_count,
    );
    let WorkflowDefinitionPublicationOutcome::Published(published) = publication else {
        panic!("the 10k definition must publish: {publication:?}");
    };
    let compilation = runtime.workflow_compilation_reuse_counters();
    let cold_started_at = std::time::Instant::now();
    assert!(matches!(
        start_instance(&application, published.definition().clone(), 918_402)
            .expect("the 10k cold start must fit its own operation budget"),
        WorkflowInstanceStartOutcome::Started(_)
    ));
    let cold = runtime.workflow_compilation_reuse_counters();
    assert_eq!(cold.cold_misses(), compilation.cold_misses() + 1);
    println!(
        "document cold start elapsed_ms={}",
        cold_started_at.elapsed().as_millis()
    );
    assert!(matches!(
        start_instance(&application, published.definition().clone(), 918_403)
            .expect("the 10k warm start must reuse compiled meaning"),
        WorkflowInstanceStartOutcome::Started(_)
    ));
    let warm = runtime.workflow_compilation_reuse_counters();
    assert_eq!(warm.warm_hits(), cold.warm_hits() + 1);
    assert_eq!(warm.cold_misses(), cold.cold_misses());
}
