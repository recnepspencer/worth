//! A genuine second authentication event cannot replace the natively admitted approver.
use super::super::super::document_retention_model::{
    host::DocumentWorkflowRuntime,
    operator_identity::{block_on, secondary_signer_identity},
    schema::{Document, DocumentPrincipalBinding, DocumentRetentionSchema},
    workflow::{WorkflowAdvanceInput, WorkflowAdvanceOperation, WorkflowApprovalCapability},
};
use super::*;
use std::time::{Duration, SystemTime};
use worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventDenial;
use worth_query_host::facade::{
    admission::authenticated_principal::*, primary_graph::WorthQueryPrincipalResolutionMode,
};

struct SecondarySigner;
impl WorthQueryAuthenticationAdapter for SecondarySigner {
    type Credential = ();
    fn configuration_identity(&self) -> &str {
        "certification-secondary-signer.v1"
    }
    fn validate<'a>(
        &'a self,
        (): (),
        _: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        Box::pin(async {
            let now = SystemTime::now();
            WorthQueryValidatedExternalPrincipal::new(
                secondary_signer_identity(),
                WorthQueryAuthenticationAudience::new("document-retention-host").unwrap(),
                WorthQueryAuthenticationMethod::new("certification").unwrap(),
                now,
                now + Duration::from_secs(3600),
                vec![],
            )
            .map_err(|_| {
                WorthQueryAuthenticationAdapterFailure::new(
                    WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation,
                )
            })
        })
    }
}

type Prepared = PreparedWorkflowAdvance<
    DocumentRetentionSchema,
    WorkflowAdvanceOperation,
    WorkflowAdvanceInput,
    Document,
>;

fn prepare_as_operator(
    application: &DocumentWorkflowRuntime,
    external: &WorthQueryAuthenticatedExternalPrincipal<DocumentRetentionSchema>,
    scope: &WorthQueryRequestScope,
    instance: PublishedWorkflowInstanceRef,
    proposal: &PublishedWorkflowProposalRef,
    required: &RequiredWorkflowApproval,
) -> Prepared {
    let runtime = application.runtime();
    runtime
        .with_application_advancement(scope, |phase| {
            let selected = runtime.on_branch(instance.branch()).select(&phase).unwrap();
            let principal_binding = runtime
                .installed_schema()
                .principal_binding(DocumentPrincipalBinding::reference())
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &principal_binding,
                    external,
                    scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let operation = runtime
                .installed_schema()
                .installed_operation(WorkflowAdvanceOperation::reference())
                .unwrap();
            let capability = runtime
                .installed_schema()
                .capability(
                    WorkflowApprovalCapability::reference(),
                    WorkflowAdvanceOperation::reference(),
                )
                .unwrap();
            let access = selected
                .admit_capability_access(
                    &phase,
                    &principal,
                    &capability,
                    WorkflowAdvanceInput {
                        document_identity: DOCUMENT_IDENTITY.to_owned(),
                    },
                    scope,
                )
                .unwrap();
            let admission = runtime
                .authorize_capability_operation(&phase, access, &operation, Default::default())
                .unwrap();
            Adapter::prepare_approval::<_, WorkflowApprovalCapability, _, _, _, _>(
                &phase,
                &selected,
                application.vocabulary(),
                instance,
                required,
                proposal,
                WorkflowApprovalDecision::Approve,
                admission,
            )
            .unwrap()
        })
        .unwrap()
}

#[test]
fn advanced_approval_cannot_reauthenticate_as_a_different_admitted_signer() {
    let (application, _, instance, proposal, required, _) =
        approval_journey("signer-affinity", 25_000);
    let runtime = application.runtime();
    let scope = request_scope();
    let operator = authenticate_operator(runtime.installed_schema(), &scope);
    let secondary = block_on(
        admit_authentication_adapter(
            runtime.installed_schema(),
            WorthQueryAuthenticationAdapterAdmission::new(
                WorthQueryAuthenticationAudience::new("document-retention-host").unwrap(),
                WorthQueryAuthenticationMethod::new("certification").unwrap(),
            ),
            SecondarySigner,
        )
        .unwrap()
        .authenticate((), &scope),
    )
    .unwrap();
    let prepared = prepare_as_operator(
        &application,
        &operator,
        &scope,
        instance.clone(),
        &proposal,
        &required,
    );
    let intent = prepared.approval_authentication_intent().unwrap().clone();
    let event = block_on(application.authentication().authenticate(
        (),
        &secondary,
        intent.clone(),
        &scope,
    ))
    .unwrap();
    assert_eq!(
        prepared.sign_approval(&event, &secondary, &scope).err(),
        Some(WorthQueryAuthenticationEventDenial::PrincipalMismatch)
    );
    // Rejection occurs before consumption: this genuine event still has its one use.
    let unused = application
        .authentication()
        .consume_for_signing(&event, &secondary, &intent, &scope)
        .expect("substitution did not consume the other signer's event");
    drop(unused);
    assert!(matches!(
        advance_instance(&application, instance.clone(), 25_010),
        Ok(WorkflowProgressOutcome::AwaitingApproval(_))
    ));
    assert!(
        matches!(approve_instance(&application, instance.clone(), &required, &proposal, WorkflowApprovalDecision::Approve, 25_011), Ok(WorkflowProgressOutcome::Completed(performed)) if !performed.replayed())
    );
    assert!(
        matches!(approve_instance(&application, instance, &required, &proposal, WorkflowApprovalDecision::Approve, 25_011), Ok(WorkflowProgressOutcome::Completed(performed)) if performed.replayed())
    );
}
