//! Real metadata disclosure: independent scope, retained basis and fresh revocation.
use super::*;
use worth_query_host::facade::declaration::application_program::{
    ApplicationWorkflowDefinitionIdentity, ApplicationWorkflowDefinitionReadTarget,
};

#[test]
fn definition_metadata_is_independent_principal_first_and_fresh_at_retained_data() {
    let application = install();
    let published = publish(&application);
    let scope = request_scope();
    let runtime = application.runtime();
    let operator = authenticate_operator(runtime.installed_schema(), &scope);
    let reviewer = authenticate_reviewer(runtime.installed_schema(), &scope);
    let retained = runtime
        .request(&operator, &scope)
        .retain_current_read()
        .unwrap();
    let target = ApplicationWorkflowDefinitionReadTarget::<GovernedDocumentWorkflow>::new(
        ApplicationWorkflowDefinitionIdentity::new("governed-original-intent").unwrap(),
    );
    let absent = ApplicationWorkflowDefinitionReadTarget::<GovernedDocumentWorkflow>::new(
        ApplicationWorkflowDefinitionIdentity::new("never-published-metadata").unwrap(),
    );
    let request = runtime.request(&operator, &scope);
    let found = request
        .at(&retained)
        .discover_workflow_definition::<GovernedDocumentWorkflow, MetadataRead>(
            application.vocabulary(),
            &target,
            input(),
        )
        .unwrap();
    assert_eq!(found.basis().selected_commit(), retained.selected_commit());
    assert_eq!(
        found.definition(),
        &WorthQueryWorkflowDefinitionDiscovery::Current(published.clone())
    );
    let missing = request
        .at(&retained)
        .discover_workflow_definition::<GovernedDocumentWorkflow, MetadataRead>(
            application.vocabulary(),
            &absent,
            input(),
        )
        .unwrap();
    assert_eq!(
        missing.definition(),
        &WorthQueryWorkflowDefinitionDiscovery::Unpublished
    );
    assert_eq!(
        missing.basis().selected_commit(),
        retained.selected_commit()
    );
    for name in [&target, &absent] {
        // A subject reader cannot substitute its installed concrete binding in the metadata slot.
        assert!(matches!(
            request
                .at(&retained)
                .discover_workflow_definition::<GovernedDocumentWorkflow, SubjectRead>(
                    application.vocabulary(),
                    name,
                    input()
                ),
            Err(WorthQueryWorkflowObservationDenial::Read(
                WorthQueryWorkflowReadDenial::Installation(_)
            ))
        ));
    }
    let denied = runtime.request(&reviewer, &scope);
    let present = denied
        .at(&retained)
        .discover_workflow_definition::<GovernedDocumentWorkflow, MetadataRead>(
            application.vocabulary(),
            &target,
            input(),
        )
        .err()
        .unwrap();
    let absent_denial = denied
        .at(&retained)
        .discover_workflow_definition::<GovernedDocumentWorkflow, MetadataRead>(
            application.vocabulary(),
            &absent,
            input(),
        )
        .err()
        .unwrap();
    match (present, absent_denial) {
        (
            WorthQueryWorkflowObservationDenial::Read(WorthQueryWorkflowReadDenial::Authorization(
                a,
            )),
            WorthQueryWorkflowObservationDenial::Read(WorthQueryWorkflowReadDenial::Authorization(
                b,
            )),
        ) => assert_eq!(a.causes(), b.causes()),
        other => panic!("unauthorized metadata presence must be undisclosed: {other:?}"),
    }
    let retired = runtime
        .request(&operator, &scope)
        .mutate(super::super::super::authoring_intent())
        .without_source()
        .idempotency(&91_699_u64)
        .prepare_workflow_definition_retirement(application.vocabulary(), published.clone())
        .unwrap()
        .execute();
    assert!(matches!(
        retired,
        worth_query_host::facade::application_entry::WorkflowDefinitionRetirementOutcome::Retired(
            _
        )
    ));
    let latest = runtime
        .request(&operator, &scope)
        .retain_current_read()
        .unwrap();
    let now = runtime
        .request(&operator, &scope)
        .at(&latest)
        .discover_workflow_definition::<GovernedDocumentWorkflow, MetadataRead>(
            application.vocabulary(),
            &target,
            input(),
        )
        .unwrap();
    assert_eq!(
        now.definition(),
        &WorthQueryWorkflowDefinitionDiscovery::Retired
    );
    let never = runtime
        .request(&operator, &scope)
        .at(&latest)
        .discover_workflow_definition::<GovernedDocumentWorkflow, MetadataRead>(
            application.vocabulary(),
            &absent,
            input(),
        )
        .unwrap();
    assert_eq!(
        never.definition(),
        &WorthQueryWorkflowDefinitionDiscovery::Unpublished
    );
    let old = runtime
        .request(&operator, &scope)
        .at(&retained)
        .discover_workflow_definition::<GovernedDocumentWorkflow, MetadataRead>(
            application.vocabulary(),
            &target,
            input(),
        )
        .unwrap();
    assert_eq!(
        old.definition(),
        &WorthQueryWorkflowDefinitionDiscovery::Current(published)
    );
    assert_eq!(old.basis().selected_commit(), retained.selected_commit());
    revoke(&application, "workflow-metadata-read-grant", 91_700);
    assert!(
        matches!(
            runtime
                .request(&operator, &scope)
                .at(&retained)
                .discover_workflow_definition::<GovernedDocumentWorkflow, MetadataRead>(
                    application.vocabulary(),
                    &target,
                    input()
                ),
            Err(WorthQueryWorkflowObservationDenial::Read(
                WorthQueryWorkflowReadDenial::Authorization(_)
            ))
        ),
        "old data never revives a revoked current metadata grant"
    );
    let cancellation=worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    let cancelled = WorthQueryRequestScope::new(None, cancellation.token());
    cancellation.cancel();
    assert!(
        matches!(
            runtime
                .request(&operator, &cancelled)
                .at(&retained)
                .discover_workflow_definition::<GovernedDocumentWorkflow, MetadataRead>(
                    application.vocabulary(),
                    &target,
                    input()
                ),
            Err(WorthQueryWorkflowObservationDenial::ExecutionRequest(_))
        ),
        "technical stops are not metadata absence"
    );
    let expired=WorthQueryRequestScope::new(Some(std::time::Instant::now()),
        worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new().token());
    assert!(
        matches!(
            runtime
                .request(&operator, &expired)
                .at(&retained)
                .discover_workflow_definition::<GovernedDocumentWorkflow, MetadataRead>(
                    application.vocabulary(),
                    &target,
                    input()
                ),
            Err(WorthQueryWorkflowObservationDenial::ExecutionRequest(_))
        ),
        "deadline expiry is not metadata absence"
    );
}
