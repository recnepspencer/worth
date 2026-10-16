//! Real original-start lookup, independent disclosure grants and retained-data revocation.
use super::super::super::{
    operator_identity::{
        authenticate_operator, authenticate_reviewer, authenticate_ungranted_reviewer,
        request_scope,
    },
    retention_entry::DOCUMENT_IDENTITY,
    schema::{Document, DocumentRetentionField},
};
use super::super::{
    WorkflowGrantStatusInput, WorkflowGrantStatusIntent, WorkflowInstanceStartInput,
    WorkflowInstanceStartIntent, WorkflowInstanceStartOperation,
};
use super::*;
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
    },
    application_discovery::*,
    application_entry::*,
};
#[path = "court/definition.rs"]
mod definition;
#[path = "court/enumeration.rs"]
mod enumeration;
#[path = "court/history.rs"]
mod history;
#[path = "court/history_stimuli.rs"]
mod history_stimuli;
#[path = "court/provenance.rs"]
mod provenance;
#[path = "court/publication.rs"]
mod publication;
#[path = "court/reacquisition.rs"]
mod reacquisition;
use publication::{prepare, publish, revoke};

#[test]
fn governed_original_intent_read_uses_fresh_subject_and_independent_audit_at_retained_data() {
    let application = install();
    let definition = publish(&application);
    let prepared = prepare(&application, definition.clone(), 91_501);
    let original = prepared.discovery_intent();
    let abandoned = prepare(&application, definition, 91_502);
    let missing = abandoned.discovery_intent();
    drop(abandoned); // Existing native prepared-attempt drop releases its unperformed attempt.
    let performed = match prepared.execute() {
        WorkflowInstanceStartOutcome::Started(performed) => performed,
        other => panic!("authentic start must publish: {other:?}"),
    };
    let instance = performed.instance().clone();
    let original_actor = performed.receipt().principal_scope().principal();
    let runtime = application.runtime();
    let scope = request_scope();
    let operator = authenticate_operator(runtime.installed_schema(), &scope);
    let reviewer = authenticate_reviewer(runtime.installed_schema(), &scope);
    let unauthorized = authenticate_ungranted_reviewer(runtime.installed_schema(), &scope);
    let observation = runtime
        .request(&operator, &scope)
        .retain_current_read()
        .unwrap();
    let before = runtime.workflow_instance_progress_counters();
    application
        .with_host_advancement(|phase| {
            assert!(
                runtime
                    .release_equality_index_generations_for_test(
                        &phase,
                        DocumentRetentionField::reference()
                    )
                    .unwrap()
                    > 0
            );
            assert!(!runtime
                .has_equality_index_generation_for_test(
                    &phase,
                    runtime.current_world(),
                    DocumentRetentionField::reference()
                )
                .unwrap());
        })
        .unwrap();
    let base = read(&application, &operator, &scope, &observation, &original).unwrap();
    match base {
        WorthQueryWorkflowInstanceRead::Live {
            instance: found,
            observation: view,
        } => {
            assert_eq!(found, instance);
            assert_eq!(view.state(), WorthQueryWorkflowObservedState::Ready);
            assert_eq!(view.selected_commit(), observation.selected_commit());
            assert_eq!(view.transition_count(), 0);
            assert!(matches!(view.audit(), WorthQueryWorkflowAudit::Withheld));
        }
        _ => panic!("the current source must be live"),
    }
    let metadata_target=worth_query_host::facade::declaration::application_program::ApplicationWorkflowDefinitionReadTarget::<GovernedDocumentWorkflow>::new(
        worth_query_host::facade::declaration::application_program::ApplicationWorkflowDefinitionIdentity::new("governed-original-intent").unwrap());
    let metadata = runtime
        .request(&operator, &scope)
        .at(&observation)
        .discover_workflow_definition::<GovernedDocumentWorkflow, MetadataRead>(
            application.vocabulary(),
            &metadata_target,
            input(),
        )
        .unwrap();
    assert!(matches!(
        metadata.definition(),
        WorthQueryWorkflowDefinitionDiscovery::Current(_)
    ));
    let audited = read(&application, &reviewer, &scope, &observation, &original).unwrap();
    match audited {
        WorthQueryWorkflowInstanceRead::Live {
            observation: view, ..
        } => match view.audit() {
            WorthQueryWorkflowAudit::Visible {
                start,
                end,
                transitions,
            } => {
                let actor = start.entity_id();
                assert_eq!(actor.partition_value(), original_actor.partition_id());
                assert_eq!(actor.local_slot_value(), original_actor.local_slot());
                assert_eq!(actor.generation_value(), original_actor.generation());
                assert!(end.is_none());
                assert!(transitions.is_empty());
            }
            _ => panic!("the separately issued audit grant must disclose the original actor"),
        },
        _ => panic!("audited current target must remain live"),
    }
    let present = read(&application, &unauthorized, &scope, &observation, &original)
        .err()
        .unwrap();
    let absent = read(&application, &unauthorized, &scope, &observation, &missing)
        .err()
        .unwrap();
    match (present,absent) {
        (WorthQueryWorkflowObservationDenial::Read(WorthQueryWorkflowReadDenial::Authorization(a)),
         WorthQueryWorkflowObservationDenial::Read(WorthQueryWorkflowReadDenial::Authorization(b)))=>assert_eq!(a.causes(),b.causes()),
        other=>panic!("unauthorized present and missing must uniformly fail at capability admission: {other:?}"),
    }
    match read(&application, &operator, &scope, &observation, &missing).unwrap() {
        WorthQueryWorkflowInstanceRead::Missing { basis } => {
            assert_eq!(basis.selected_commit(), observation.selected_commit())
        }
        _ => panic!("an authentic unperformed start remains missing at the retained basis"),
    }
    assert_eq!(
        runtime.workflow_instance_progress_counters(),
        before,
        "read-only reconstruction never updates command progress"
    );
    assert_eq!(
        runtime
            .request(&operator, &scope)
            .retain_current_read()
            .unwrap()
            .selected_commit(),
        observation.selected_commit()
    );
    application
        .with_host_advancement(|phase| {
            assert!(
                !runtime
                    .has_equality_index_generation_for_test(
                        &phase,
                        runtime.current_world(),
                        DocumentRetentionField::reference()
                    )
                    .unwrap(),
                "subject/audit/exact lookup must leave the unrelated cold index unpublished"
            );
        })
        .unwrap();
    revoke(&application, "workflow-audit-read-grant", 91_503);
    let withheld = read(&application, &reviewer, &scope, &observation, &original).unwrap();
    match withheld {
        WorthQueryWorkflowInstanceRead::Historical {
            observation: view, ..
        } => {
            assert_eq!(view.selected_commit(), observation.selected_commit());
            assert!(matches!(view.audit(), WorthQueryWorkflowAudit::Withheld));
        }
        _ => panic!("revocation does not turn retained data into a current live reference"),
    }
    revoke(&application, "workflow-subject-read-grant", 91_504);
    assert!(matches!(
        read(&application, &reviewer, &scope, &observation, &original),
        Err(WorthQueryWorkflowObservationDenial::Read(
            WorthQueryWorkflowReadDenial::Authorization(_)
        ))
    ));
    let cancellation=worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    let cancelled = WorthQueryRequestScope::new(None, cancellation.token());
    cancellation.cancel();
    assert!(
        matches!(
            read(&application, &reviewer, &cancelled, &observation, &original),
            Err(WorthQueryWorkflowObservationDenial::ExecutionRequest(_))
        ),
        "technical request stop is not audit withholding or target absence"
    );
}
fn read(
    application: &GovernedRuntime,
    principal: &WorthQueryAuthenticatedExternalPrincipal<DocumentRetentionSchema>,
    scope: &WorthQueryRequestScope,
    observation: &WorthQueryApplicationReadObservation,
    original: &WorthQueryWorkflowInstanceDiscoveryIntent<DocumentRetentionSchema>,
) -> Result<WorthQueryWorkflowInstanceRead, WorthQueryWorkflowObservationDenial> {
    application
        .runtime()
        .request(principal, scope)
        .at(observation)
        .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
            application.vocabulary(),
            original,
            input(),
            input(),
        )
}
fn input() -> WorkflowReadInput {
    WorkflowReadInput {
        document_identity: DOCUMENT_IDENTITY.to_owned(),
    }
}

#[test]
fn borrowed_phase_revocation_refuses_foreign_custody_before_contacts_and_key_claim() {
    use worth_query_host::facade::{
        application_contribution::{
            WorthQueryAdvancementDenial, WorthQueryManagedComputationResourceDenial,
        },
        primary_graph::{
            installed_source_reads_on_this_thread_for_test as reads,
            WorthQueryExecutionObservation, WorthQueryMutationHandlerWork,
        },
    };
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let application = install();
    let foreign = install();
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let before = runtime
        .request(&principal, &scope)
        .retain_current_read()
        .unwrap();
    let key = 91_900_u64;
    let report = foreign
        .runtime()
        .with_application_advancement(&scope, |phase| {
            let contacts = reads();
            let report = runtime
                .request(&principal, &scope)
                .advanced()
                .mutate(WorkflowGrantStatusIntent {
                    input: WorkflowGrantStatusInput {
                        grant_identity: "workflow-metadata-read-grant".to_owned(),
                        status: "revoked".to_owned(),
                    },
                })
                .without_source()
                .idempotency(&key)
                .execute_in_program_report_in_advancement(&phase, application.program_runtime());
            assert_eq!(
                reads(),
                contacts,
                "foreign custody refuses before installed-source or branch contact"
            );
            report
        })
        .unwrap();
    assert!(matches!(
        report.decision_work(),
        WorthQueryMutationHandlerWork::NotStarted
    ));
    assert!(matches!(
        report.outcome(),
        Err(
            WorthQueryApplicationRequestMutationDenial::ExecutionRequest(
                WorthQueryAdvancementDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::ForeignAdvancementPhase
                )
            )
        )
    ));
    assert_eq!(
        runtime
            .request(&principal, &scope)
            .retain_current_read()
            .unwrap()
            .selected_commit(),
        before.selected_commit()
    );
    // Same intent and key must freshly commit on the real phase, never replay a foreign claim.
    revoke(&application, "workflow-metadata-read-grant", key);
    assert_ne!(
        runtime
            .request(&principal, &scope)
            .retain_current_read()
            .unwrap()
            .selected_commit(),
        before.selected_commit()
    );
}
