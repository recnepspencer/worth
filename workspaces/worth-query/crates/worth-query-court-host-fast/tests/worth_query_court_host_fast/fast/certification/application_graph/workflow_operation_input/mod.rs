pub(crate) use worth_query_worlds::facade::certification::application_graph::workflow_operation_input::*;

#[test]
fn ordinary_publication_refuses_zero_and_two_inputs_before_publication() {
    let application = publish_workflow_on_first_program();
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    for inputs in [0, 2] {
        runtime
            .with_host_advancement(|phase| {
                let refusal = runtime
                    .request(&principal, &scope)
                    .mutate(authoring_intent())
                    .workflow(&phase, &application, draft(inputs));
                match refusal {
                    Err(WorthQueryOrdinaryWorkflowPublicationDenial::InvalidDefinition(denial)) => {
                        assert_eq!(
                            denial.kind(),
                            ApplicationWorkflowValidationDenialKind::MissingOperationInput
                        );
                        assert_eq!(denial.subject(), "perform");
                    }
                    _ => panic!("ordinary publication must refuse the input contract"),
                }
            })
            .unwrap();
    }
    assert_eq!(
        super::document_retention_model::workflow::discover_definition_metadata(
            &application,
            runtime.current_world(),
            &ApplicationWorkflowDefinitionIdentity::new("input-cardinality").unwrap()
        ),
        Err(())
    );
    let published = runtime
        .with_host_advancement(|phase| {
            runtime
                .request(&principal, &scope)
                .mutate(authoring_intent())
                .workflow(&phase, &application, draft(1))
                .unwrap()
                .publish(WorkflowDefinitionExpectedPredecessor::Absent)
                .idempotency(&94101_u64)
        })
        .unwrap()
        .execute()
        .unwrap();
    assert!(matches!(
        published,
        WorkflowDefinitionPublicationOutcome::Published(_)
    ));
}

#[test]
fn authentic_predecessor_draft_cannot_import_executable_meaning() {
    let _application = publish_workflow_on_first_program();
    // Captured with the original production validator and writer, before the
    // guarded-input requirement was enforced. No bytes are converted on import.
    let bytes = include_bytes!(
        "../../../../../../../worth-query-worlds/src/certification/application_graph/workflow_definition_readmission/missing_input.wqwd"
    );
    let original = bytes.to_vec();
    let denial = decode_workflow_definition_draft(bytes, WorthQueryPackageArchiveLimits::DEFAULT)
        .expect_err("displaced authored format is refused before executable interpretation");
    assert_eq!(denial.kind(), worth_query_package_archive::facade::WorthQueryPackageArchiveDenialKind::UnsupportedWorkflowDefinitionDraftVersion);
    assert_eq!(bytes.as_slice(), original);
}
