pub(crate) use worth_query_worlds::facade::certification::application_graph::workflow_ordinary_component::*;

#[test]
fn public_component_ports_validate_and_compile_into_ordinary_progression() {
    let validated = reviewed_component_draft()
        .validate()
        .expect("the public component definition validates");
    let expansions = validated.component_expansions();
    assert_eq!(expansions.len(), 1);
    let review = &expansions[0];
    assert_eq!(review.component().as_str(), "required-document-review");
    assert_eq!(review.occurrence_path(), "checks");
    assert_eq!(review.ports().len(), 3);
    for (port, node) in [
        ("structural-subject", "checks/structural"),
        ("reviewed-evidence", "checks/evidence"),
    ] {
        let binding = review
            .ports()
            .iter()
            .find(|binding| binding.identity() == port)
            .expect("the public port remains in expansion provenance");
        assert_eq!(binding.expanded_node().as_str(), node);
    }
    assert_eq!(review.nodes().len(), 3);
    assert_eq!(review.nodes()[0].authored().as_str(), "structural");
    assert_eq!(review.nodes()[0].expanded().as_str(), "checks/structural");

    let application = publish_workflow_on_first_program();
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let published = runtime
        .with_host_advancement(|host_phase| {
            runtime
                .request(&principal, &scope)
                .mutate(WorkflowDefinitionAuthoringIntent {
                    input: WorkflowDefinitionAuthoringInput {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                        retention_days: 8,
                    },
                })
                .workflow(&host_phase, &application, reviewed_component_draft())
                .expect("the component binds to installed vocabulary")
                .publish(WorkflowDefinitionExpectedPredecessor::Absent)
                .idempotency(&918_300)
        })
        .expect("installed host call opens")
        .execute()
        .expect("the ordinary publication prepares");
    let definition = match published {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("the definition must publish: {other:?}"),
    };
    let instance = match start_instance(&application, definition, 918_301)
        .expect("the published definition starts")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("the instance must start: {other:?}"),
    };
    propose_instance(&application, instance.clone(), 918_302).expect("the proposal prepares");
    let progress = advance_instance(&application, instance, 918_303)
        .expect("the expanded definition advances");
    match progress {
        WorkflowProgressOutcome::AwaitingAssessment(required) => {
            assert_eq!(required.node_path(), "checks/structural");
        }
        other => panic!("the published component must demand structural review: {other:?}"),
    }
}
