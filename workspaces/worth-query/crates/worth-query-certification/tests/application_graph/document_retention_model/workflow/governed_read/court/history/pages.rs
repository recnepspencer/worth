//! Affinity, fresh revocation and canonical envelope dependency marks on a real workflow.
use super::*;
#[test]
fn governed_history_pages_and_envelope_marks_preserve_scope_revocation_and_custody() {
    let application = install();
    let instance = start(&application, 97_000);
    propose(&application, instance.clone(), 97_010);
    let _required = collect_to_approval(&application, &instance, 97_020);
    let runtime = application.runtime();
    let scope = request_scope();
    let operator = authenticate_operator(runtime.installed_schema(), &scope);
    let reviewer = authenticate_reviewer(runtime.installed_schema(), &scope);
    let basis = runtime
        .request(&operator, &scope)
        .retain_current_read()
        .unwrap();
    let target = target(&application, &basis);
    let before = runtime.workflow_instance_progress_counters();
    let request = runtime.request(&operator, &scope);
    let first = request
        .at(&basis)
        .read_workflow_history::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
            application.vocabulary(),
            &target,
            input(),
            input(),
            1,
            WorthQueryWorkflowIndexRead::PublishedOnly,
            None,
        )
        .unwrap();
    assert_eq!(first.entries().len(), 1);
    assert_eq!(first.entries()[0].occurrence(), 0);
    assert!(first.continuation().is_some());
    assert!(matches!(
        request
            .at(&basis)
            .read_workflow_history::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
                application.vocabulary(),
                &target,
                input(),
                input(),
                2,
                WorthQueryWorkflowIndexRead::PublishedOnly,
                first.continuation()
            ),
        Err(WorthQueryWorkflowObservationDenial::Read(
            WorthQueryWorkflowReadDenial::ContinuationMismatch
        ))
    ));
    let second = request
        .at(&basis)
        .read_workflow_history::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
            application.vocabulary(),
            &target,
            input(),
            input(),
            1,
            WorthQueryWorkflowIndexRead::PublishedOnly,
            first.continuation(),
        )
        .unwrap();
    assert_eq!(second.entries().len(), 1);
    assert!(second.entries()[0].occurrence() > first.entries()[0].occurrence());
    let mut subscription = request
        .at(&basis)
        .subscribe_workflow_history::<GovernedDocumentWorkflow, SubjectRead>(
            application.vocabulary(),
            &target,
            input(),
        )
        .unwrap();
    let old_retention =
        crate::document_retention_model::readback::read_retention(runtime, instance.branch());
    assert!(matches!(
        crate::document_retention_model::presented_request::set_retention(
            application.program_runtime(),
            instance.branch(),
            old_retention + 1,
            97_100
        )
        .unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(
        crate::document_retention_model::readback::read_retention(runtime, instance.branch()),
        old_retention + 1,
        "genuine unrelated domain effect"
    );
    assert_eq!(
        request
            .at(&basis)
            .poll_workflow_history::<GovernedDocumentWorkflow, SubjectRead>(
                application.vocabulary(),
                &mut subscription,
                input()
            )
            .unwrap(),
        WorthQueryWorkflowHistoryMark::Pending,
        "unrelated domain commit does not touch native workflow records or absent edges"
    );
    let new_basis = request.retain_current_read().unwrap();
    assert!(matches!(
        request
            .at(&new_basis)
            .read_workflow_history::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
                application.vocabulary(),
                &target,
                input(),
                input(),
                1,
                WorthQueryWorkflowIndexRead::PublishedOnly,
                first.continuation()
            ),
        Err(WorthQueryWorkflowObservationDenial::Read(
            WorthQueryWorkflowReadDenial::PolicyMismatch
        ))
    ));
    revoke(&application, "workflow-audit-read-grant", 97_101);
    let withheld = page(&application, &reviewer, &scope, &basis, &target);
    assert!(withheld
        .entries()
        .iter()
        .all(|r| matches!(r.audit(), WorthQueryWorkflowHistoryAudit::Withheld)));
    assert!(
        withheld.entries()[0].proposal().is_some(),
        "audit revocation cannot erase an existing proposal from base lifecycle truth"
    );
    assert_eq!(
        runtime.workflow_instance_progress_counters(),
        before,
        "read-only pages, audit and marks do not spend workflow steps or warm progress"
    );
    // A genuine bounded native envelope overflow; no reduced test-only capacity or forged cause.
    for offset in 0..65u64 {
        assert!(matches!(
            crate::document_retention_model::presented_request::set_retention(
                application.program_runtime(),
                instance.branch(),
                old_retention + 2 + offset,
                97_200 + offset
            )
            .unwrap(),
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
    }
    assert_eq!(
        request
            .at(&basis)
            .poll_workflow_history::<GovernedDocumentWorkflow, SubjectRead>(
                application.vocabulary(),
                &mut subscription,
                input()
            )
            .unwrap(),
        WorthQueryWorkflowHistoryMark::Overflow
    );
    revoke(&application, "workflow-subject-read-grant", 97_300);
    assert!(matches!(
        request
            .at(&basis)
            .read_workflow_history::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
                application.vocabulary(),
                &target,
                input(),
                input(),
                1,
                WorthQueryWorkflowIndexRead::PublishedOnly,
                first.continuation()
            ),
        Err(WorthQueryWorkflowObservationDenial::Read(
            WorthQueryWorkflowReadDenial::Authorization(_)
        ))
    ));
    assert!(
        matches!(
            request
                .at(&basis)
                .poll_workflow_history::<GovernedDocumentWorkflow, SubjectRead>(
                    application.vocabulary(),
                    &mut subscription,
                    input()
                ),
            Err(WorthQueryWorkflowObservationDenial::Read(
                WorthQueryWorkflowReadDenial::Authorization(_)
            ))
        ),
        "even an overflow is disclosed only after fresh permission; refusal retains the owner"
    );
    let head = request.retain_current_read().unwrap();
    subscription.close();
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        head.selected_commit()
    );
}
