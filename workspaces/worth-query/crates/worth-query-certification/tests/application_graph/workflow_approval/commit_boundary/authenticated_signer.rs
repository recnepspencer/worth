//! A genuine second authentication event cannot replace the natively admitted approver.
use super::super::super::document_retention_model::{
    operator_identity::{authenticate_reviewer, block_on, secondary_signer_identity},
    workflow::{WorkflowAdvanceInput, WorkflowApprovalIntent},
};
use super::*;
use std::time::{Duration, SystemTime};
use worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventDenial;
use worth_query_host::facade::{
    admission::authenticated_principal::*,
    application_entry::WorthQueryWorkflowAdvancePreparationDenial,
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

#[test]
fn approval_owner_rejects_authentication_from_another_principal() {
    let (application, _, instance, proposal, required, _) =
        approval_journey("signer-affinity", 25_000);
    let runtime = application.runtime();
    let scope = request_scope();
    let reviewer = authenticate_reviewer(runtime.installed_schema(), &scope);
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
    // The native owner supplies the original actor context and freshly admits
    // a reviewer distinct from the proposer; the test must not restate that admission.
    let key = 25_005;
    let signing = runtime
        .request(&reviewer, &scope)
        .on_branch(instance.branch())
        .mutate(WorkflowApprovalIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&key)
        .prepare_workflow_approval(
            &application,
            instance.clone(),
            &required,
            &proposal,
            WorkflowApprovalDecision::Approve,
        )
        .unwrap();
    let intent = signing.authentication_intent().unwrap().clone();
    let event = block_on(application.authentication().authenticate(
        (),
        &secondary,
        intent.clone(),
        &scope,
    ))
    .unwrap();
    assert!(matches!(
        signing.sign(&event).err(),
        Some(WorthQueryWorkflowAdvancePreparationDenial::Authentication(
            WorthQueryAuthenticationEventDenial::PrincipalMismatch
        ))
    ));
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
