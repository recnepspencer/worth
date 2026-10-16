//! One installed Bank world exercises binding refusal, native close and vocabulary retry.
use super::*;
use bank_domain::{
    model::{AccountName, BankPrincipalId, InstitutionId},
    proposals::BankIdempotencyKey,
    schema::CreatePersonalAccount,
};
use crate::{
    application_definition::BankApplicationP1,
    identity_runtime::{installation, prepare_seeds},
    support::{
        authentication_configuration, block_on, request_scope, CausalBankAuthenticationAdapter,
        CausalCredential, DynamicIdentity,
    },
    BankApprovalAuthenticationConfiguration, BankAuthenticationBoundary, BankPrincipalSeed,
    PersonalAccountCreationAction,
};
use worth_query_host::facade::{
    admission::authenticated_principal::{
        admit_authentication_adapter, WorthQueryAuthenticationAdapterAdmission,
        WorthQueryAuthenticatedExternalPrincipal, WorthQueryCancellationSource, WorthQueryRequestScope,
    },
    application_entry::{WorthQueryApplicationRequestExt, WorthQueryOperationCloseDenial, WorthQuerySourceOnlyOperation},
    application_installation::WorthQueryWorkflowRuntimeBindingDenial,
    declaration::application_program::ApplicationSourceCompletion,
};

type UnexecutedAccount = WorthQuerySourceOnlyOperation<
    BankSchema, CreatePersonalAccount, BankApplication, PersonalAccountCreationAction,
    ApplicationSourceCompletion,
>;

#[test]
fn workflow_binding_refusal_preserves_inputs_until_real_operation_closes() {
    let identity = DynamicIdentity::new("workflow-binding-retry");
    let owner = BankPrincipalId::new(41).unwrap();
    // Setup failures are fatal fixture failures, not recoverable application outcomes.
    let seeds = prepare_seeds([BankPrincipalSeed::enabled(owner, identity.external())])
        .expect("the real installed principal seed is valid");
    let pending = installation::prepare_installation(
        seeds, None, installation::BankAuthorizationTimeInstallation::System,
        BankApprovalAuthenticationConfiguration::denying(),
    ).expect("the ordinary Bank installation reaches workflow binding");
    let revision = *pending.workflow_spec().program_revision();
    let signing_binding = pending.authentication.binding_identity().clone();
    let configuration = authentication_configuration();
    let authentication = BankAuthenticationBoundary::new(admit_authentication_adapter(
        pending.runtime.installed_schema(),
        WorthQueryAuthenticationAdapterAdmission::new(configuration.audience, configuration.method),
        CausalBankAuthenticationAdapter,
    ).expect("the real authentication adapter is admitted"));
    let scope = request_scope();
    let principal = block_on(authentication.authenticate(CausalCredential::for_identity(&identity), &scope))
        .expect("the installing principal authenticates");
    let operation = retain_unexecuted(&pending.runtime, &principal, &scope, owner, "binding");
    let pending = expect_binding_refusal(pending.bind(), WorthQueryWorkflowRuntimeBindingDenial::RuntimeKeepaliveRetained);
    assert_eq!(*pending.workflow_spec().program_revision(), revision);
    assert_eq!(pending.authentication.binding_identity(), &signing_binding);

    // An editable refusal preserves both the replaced original and the submitted successor.
    let successor = installation::install_payment_workflow_spec(
        pending.runtime.installed_schema(),
        pending.runtime.supported_program::<BankApplicationP1>().expect("P1 is genuinely rostered").installed_program(),
    ).expect("the successor vocabulary installs against actual rostered meaning");
    let mut pending = pending;
    let original = pending.replace_workflow_spec(successor);
    let mut pending = expect_binding_refusal(pending.bind(), WorthQueryWorkflowRuntimeBindingDenial::ForeignProgram);
    let successor = pending.replace_workflow_spec(original);
    assert_ne!(successor.program_revision(), pending.workflow_spec().program_revision());

    let cancellation = WorthQueryCancellationSource::new();
    let stopped_scope = WorthQueryRequestScope::new(None, cancellation.token());
    let stopped_principal = block_on(authentication.authenticate(CausalCredential::for_identity(&identity), &stopped_scope))
        .expect("authentication precedes cancellation");
    cancellation.cancel();
    let (operation, denial) = match operation.close(&pending.runtime.request(&stopped_principal, &stopped_scope)) {
        Err(refused) => refused,
        Ok(_) => panic!("cancelled close must preserve the real operation"),
    };
    assert!(matches!(denial, WorthQueryOperationCloseDenial::Execution(
        worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted(
            worth_query_host::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled
        )
    )));
    let pending = expect_binding_refusal(pending.bind(), WorthQueryWorkflowRuntimeBindingDenial::RuntimeKeepaliveRetained);
    let fresh_scope = request_scope();
    let fresh_principal = block_on(authentication.authenticate(CausalCredential::for_identity(&identity), &fresh_scope))
        .expect("lawful close receives freshly authenticated request context");
    operation.close(&pending.runtime.request(&fresh_principal, &fresh_scope))
        .unwrap_or_else(|(_, denial)| panic!("actual Initial owner closes: {denial:?}"));
    let mut bank = pending.bind().expect("releasing the genuine operation permits binding the same world");
    assert_eq!(*bank.approved_payment_workflow_spec().program_revision(), revision);
    assert_eq!(bank.runtime.authentication_owner().binding_identity(), &signing_binding);
    let admitted = block_on(bank.authenticate_with(&authentication, CausalCredential::for_identity(&identity), &fresh_scope))
        .expect("the returned Bank runtime retains its original principal binding");
    assert_eq!(admitted.principal_id(), owner);
    assert_eq!(bank.mapped_principal_count(), 1);

    // Borrowed support refusal returns the actual submitted spec for a later successful retry.
    let operation = retain_unexecuted(bank.application_program(), &fresh_principal, &fresh_scope, owner, "support");
    let (successor, denial) = match bank.runtime.support_workflow_spec(successor) {
        Err(refused) => refused,
        Ok(()) => panic!("a retained operation prevents vocabulary growth"),
    };
    assert_eq!(denial, WorthQueryWorkflowRuntimeBindingDenial::RuntimeKeepaliveRetained);
    operation.close(&bank.application_program().request(&fresh_principal, &fresh_scope))
        .unwrap_or_else(|(_, denial)| panic!("support keepalive closes lawfully: {denial:?}"));
    bank.runtime.support_workflow_spec(successor)
        .unwrap_or_else(|(_, denial)| panic!("the original supported vocabulary retries: {denial:?}"));
}

fn expect_binding_refusal(
    result: Result<BankIdentityRuntime, BankIdentityRuntimeBuildError>,
    expected: WorthQueryWorkflowRuntimeBindingDenial,
) -> BankIdentityRuntimeWorkflowBinding {
    match result {
        Err(BankIdentityRuntimeBuildError::WorkflowRuntimeBinding { binding, denial }) => {
            assert_eq!(denial, expected);
            *binding
        }
        Err(other) => panic!("wrong binding refusal cause: {other:?}"),
        Ok(_) => panic!("the installing world must remain recoverable on refusal"),
    }
}

fn retain_unexecuted(
    runtime: &WorthQueryProgramApplicationRuntime<BankSchema, BankApplication>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<BankSchema>,
    scope: &WorthQueryRequestScope,
    owner: BankPrincipalId,
    key: &str,
) -> UnexecutedAccount {
    // Initial custody is intentionally abandoned without executing a domain mutation.
    runtime.request(principal, scope).mutate(CreatePersonalAccount {
        institution: InstitutionId::new(1).unwrap(),
        owner,
        display_name: AccountName::new("Unexecuted binding owner").unwrap(),
    }).without_source().idempotency_owned(BankIdempotencyKey::new(format!("workflow-{key}")).unwrap())
        .unwrap_or_else(|(_, _, denial)| panic!("native input capture: {denial:?}"))
        .retain_operation::<BankApplication, PersonalAccountCreationAction>(runtime)
        .unwrap_or_else(|(_, denial)| panic!("actual declared Bank action construction: {denial:?}"))
}
