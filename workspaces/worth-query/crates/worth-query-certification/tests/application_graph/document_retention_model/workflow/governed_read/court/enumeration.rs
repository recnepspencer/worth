//! Real scoped prefix pages, exact continuation and fresh disclosure.
use super::super::super::super::retention_entry::RELATED_DOCUMENT_IDENTITY;
use super::*;
use worth_query_host::facade::declaration::application_program::{
    ApplicationWorkflowDefinitionIdentity, ApplicationWorkflowDefinitionReadTarget,
};

#[test]
fn definition_pages_without_start_tokens_preserve_scope_basis_and_fresh_disclosure() {
    let application = install();
    let definition = publish(&application);
    let scope = request_scope();
    let runtime = application.runtime();
    let operator = authenticate_operator(runtime.installed_schema(), &scope);
    let reviewer = authenticate_reviewer(runtime.installed_schema(), &scope);
    let mut expected = std::collections::BTreeSet::new();
    for (document, key) in [
        (DOCUMENT_IDENTITY, 91_800_u64),
        (DOCUMENT_IDENTITY, 91_801),
        (RELATED_DOCUMENT_IDENTITY, 91_802),
    ] {
        let start = runtime
            .request(&operator, &scope)
            .mutate(WorkflowInstanceStartIntent {
                input: WorkflowInstanceStartInput {
                    document_identity: document.to_owned(),
                },
            })
            .without_source()
            .idempotency(&key)
            .prepare_workflow_instance_start(application.vocabulary(), definition.clone())
            .unwrap()
            .execute();
        match start {
            WorkflowInstanceStartOutcome::Started(done) => {
                expected.insert(done.instance().entity_id());
            }
            other => panic!("genuine independently authorized starts required: {other:?}"),
        };
    }
    // No discovery_intent is captured anywhere in this court.
    let basis = runtime
        .request(&operator, &scope)
        .retain_current_read()
        .unwrap();
    let before = runtime.workflow_instance_progress_counters();
    let target = ApplicationWorkflowDefinitionReadTarget::<GovernedDocumentWorkflow>::new(
        ApplicationWorkflowDefinitionIdentity::new("governed-original-intent").unwrap(),
    );
    let request = runtime.request(&operator, &scope);
    let first = request
        .at(&basis)
        .read_workflow_instances::<GovernedDocumentWorkflow, DefinitionRead>(
            application.vocabulary(),
            &target,
            input(),
            WorthQueryWorkflowReadExtent::Definition,
            1,
            WorthQueryWorkflowIndexRead::PublishedOnly,
            None,
        )
        .unwrap();
    assert_eq!(first.entries().len(), 1);
    assert!(first.continuation().is_some());
    let alternate = request
        .at(&basis)
        .read_workflow_instances::<GovernedDocumentWorkflow, DefinitionRead>(
            application.vocabulary(),
            &target,
            WorkflowReadInput {
                document_identity: RELATED_DOCUMENT_IDENTITY.to_owned(),
            },
            WorthQueryWorkflowReadExtent::Definition,
            1,
            WorthQueryWorkflowIndexRead::PublishedOnly,
            None,
        )
        .unwrap();
    assert_eq!(
        alternate.entries().len(),
        1,
        "the alternate definition resource scope genuinely admits without a cursor"
    );
    assert!(
        matches!(
            request
                .at(&basis)
                .read_workflow_instances::<GovernedDocumentWorkflow, DefinitionRead>(
                    application.vocabulary(),
                    &target,
                    WorkflowReadInput {
                        document_identity: RELATED_DOCUMENT_IDENTITY.to_owned()
                    },
                    WorthQueryWorkflowReadExtent::Definition,
                    1,
                    WorthQueryWorkflowIndexRead::PublishedOnly,
                    Some(&first)
                ),
            Err(WorthQueryWorkflowObservationDenial::Read(
                WorthQueryWorkflowReadDenial::ContinuationMismatch
            ))
        ),
        "freshly authorized different resource scope cannot reuse the old opaque continuation"
    );
    let mut found = std::collections::BTreeSet::new();
    let mut page = first;
    loop {
        assert_eq!(page.basis().selected_commit(), basis.selected_commit());
        for item in page.entries() {
            match item {
                WorthQueryWorkflowInstanceRead::Live {
                    instance,
                    observation,
                } => {
                    assert!(found.insert(instance.entity_id()));
                    assert!(matches!(
                        observation.audit(),
                        WorthQueryWorkflowAudit::Withheld
                    ));
                }
                _ => panic!("authentic current untouched instances must be live"),
            }
        }
        let Some(cursor) = page.continuation() else {
            break;
        };
        let next = request
            .at(&basis)
            .read_workflow_instances::<GovernedDocumentWorkflow, DefinitionRead>(
                application.vocabulary(),
                &target,
                input(),
                WorthQueryWorkflowReadExtent::Definition,
                1,
                WorthQueryWorkflowIndexRead::PublishedOnly,
                Some(cursor),
            )
            .unwrap();
        page = next;
    }
    assert_eq!(found, expected);
    let subject = request
        .at(&basis)
        .read_workflow_instances::<GovernedDocumentWorkflow, SubjectRead>(
            application.vocabulary(),
            &target,
            input(),
            WorthQueryWorkflowReadExtent::Subject,
            8,
            WorthQueryWorkflowIndexRead::PublishedOnly,
            None,
        )
        .unwrap();
    assert_eq!(
        subject.entries().len(),
        2,
        "subject prefix excludes the real other subject, rather than scanning/filtering its rows"
    );
    let empty = ApplicationWorkflowDefinitionReadTarget::<GovernedDocumentWorkflow>::new(
        ApplicationWorkflowDefinitionIdentity::new("unpublished-page-target").unwrap(),
    );
    for name in [&target, &empty] {
        assert!(matches!(
            runtime
                .request(&reviewer, &scope)
                .at(&basis)
                .read_workflow_instances::<GovernedDocumentWorkflow, DefinitionRead>(
                    application.vocabulary(),
                    name,
                    input(),
                    WorthQueryWorkflowReadExtent::Definition,
                    1,
                    WorthQueryWorkflowIndexRead::PublishedOnly,
                    None
                ),
            Err(WorthQueryWorkflowObservationDenial::Read(
                WorthQueryWorkflowReadDenial::Authorization(_)
            ))
        ));
    }
    let cursor = request
        .at(&basis)
        .read_workflow_instances::<GovernedDocumentWorkflow, DefinitionRead>(
            application.vocabulary(),
            &target,
            input(),
            WorthQueryWorkflowReadExtent::Definition,
            1,
            WorthQueryWorkflowIndexRead::PublishedOnly,
            None,
        )
        .unwrap();
    for (name, width) in [(&empty, 1), (&target, 2)] {
        assert!(matches!(
            request
                .at(&basis)
                .read_workflow_instances::<GovernedDocumentWorkflow, DefinitionRead>(
                    application.vocabulary(),
                    name,
                    input(),
                    WorthQueryWorkflowReadExtent::Definition,
                    width,
                    WorthQueryWorkflowIndexRead::PublishedOnly,
                    cursor.continuation()
                ),
            Err(WorthQueryWorkflowObservationDenial::Read(
                WorthQueryWorkflowReadDenial::ContinuationMismatch
            ))
        ));
    }
    assert_eq!(runtime.workflow_instance_progress_counters(), before);
    revoke(&application, "workflow-audit-read-grant", 91_803); // Genuine unrelated grant change advances head.
    let latest = request.retain_current_read().unwrap();
    assert_ne!(latest.selected_commit(), basis.selected_commit());
    assert!(matches!(
        request
            .at(&latest)
            .read_workflow_instances::<GovernedDocumentWorkflow, DefinitionRead>(
                application.vocabulary(),
                &target,
                input(),
                WorthQueryWorkflowReadExtent::Definition,
                1,
                WorthQueryWorkflowIndexRead::PublishedOnly,
                cursor.continuation()
            ),
        Err(WorthQueryWorkflowObservationDenial::Read(
            WorthQueryWorkflowReadDenial::ContinuationMismatch
        ))
    ));
    let old = request
        .at(&basis)
        .read_workflow_instances::<GovernedDocumentWorkflow, DefinitionRead>(
            application.vocabulary(),
            &target,
            input(),
            WorthQueryWorkflowReadExtent::Definition,
            1,
            WorthQueryWorkflowIndexRead::PublishedOnly,
            cursor.continuation(),
        )
        .unwrap();
    assert!(matches!(
        &old.entries()[0],
        WorthQueryWorkflowInstanceRead::Historical { .. }
    ));
    revoke(&application, "workflow-definition-read-grant", 91_804);
    assert!(matches!(
        request
            .at(&basis)
            .read_workflow_instances::<GovernedDocumentWorkflow, DefinitionRead>(
                application.vocabulary(),
                &target,
                input(),
                WorthQueryWorkflowReadExtent::Definition,
                1,
                WorthQueryWorkflowIndexRead::PublishedOnly,
                cursor.continuation()
            ),
        Err(WorthQueryWorkflowObservationDenial::Read(
            WorthQueryWorkflowReadDenial::Authorization(_)
        ))
    ));
    assert_eq!(
        runtime.workflow_instance_progress_counters(),
        before,
        "old-basis continuation cannot install/evict head progress"
    );
}
