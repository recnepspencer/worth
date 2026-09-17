use worth_query_decl::facade::{
    application_operation::{
        ApplicationMutationOutputContract, ApplicationMutationOutputPosture,
        ApplicationMutationOutputRoleDescriptor,
    },
    application_schema::{NoApplicationUnit, ReadOnly},
    worth_query_mutation_binding, worth_query_structured_value_binding,
};

use crate::model::{AccountAuthorizationId, AccountId, BankPrincipalId, CustomerRole};
use crate::proposals::{
    BankIdempotencyKey, BankInvariantApprovedProposal, BankProposalDenial, CanonicalProposalPayload,
};

use super::create_personal_account_binding::client_key_identity;
use super::{
    GrantAccountAuthorization, GrantAccountAuthorizationInputBinding, RevokeAccountAuthorization,
    RevokeAccountAuthorizationInputBinding,
};
use crate::schema::{
    Account, AccountAuthorization, AccountIdentity, BankPrincipalBinding, BankPrincipalIdBinding,
    BankSchema, ExternalPrincipalMapping, GrantAccountAuthorizationOperation, Identity, Principal,
    RevokeAccountAuthorizationOperation,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccountAccessResult {
    pub authorization: AccountAuthorizationId,
}

worth_query_structured_value_binding!(
    pub AccountAccessResultBinding for AccountAccessResult {
        identity: "bank.operation.account-access.result.v1"
    }
);

worth_query_structured_value_binding!(
    pub AccountAccessDenialBinding for BankProposalDenial {
        identity: "bank.operation.account-access.denial.v1"
    }
);

pub struct GrantAccountAccessOutputs;
pub struct RevokeAccountAccessOutputs;

pub const ACCOUNT_ACCESS_OUTPUT_AUTHORIZATION: &str = "authorization";

impl ApplicationMutationOutputContract<BankSchema> for GrantAccountAccessOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[ApplicationMutationOutputRoleDescriptor::for_entity::<
            BankSchema,
            AccountAuthorization,
        >(
            ACCOUNT_ACCESS_OUTPUT_AUTHORIZATION,
            ApplicationMutationOutputPosture::Create,
        )];
}

impl ApplicationMutationOutputContract<BankSchema> for RevokeAccountAccessOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[ApplicationMutationOutputRoleDescriptor::for_entity::<
            BankSchema,
            AccountAuthorization,
        >(
            ACCOUNT_ACCESS_OUTPUT_AUTHORIZATION,
            ApplicationMutationOutputPosture::Retire,
        )];
}

fn grant_scope(input: &GrantAccountAuthorization) -> AccountId {
    input.account
}

fn revoke_scope(input: &RevokeAccountAuthorization) -> AccountId {
    input.account
}

fn role_identity(role: CustomerRole) -> u64 {
    match role {
        CustomerRole::PersonalOwner => 0,
        CustomerRole::BusinessOwner => 1,
        CustomerRole::Initiator => 2,
        CustomerRole::Approver => 3,
        CustomerRole::Viewer => 4,
    }
}

fn grant_input_identity(input: &GrantAccountAuthorization) -> [u8; 32] {
    *CanonicalProposalPayload::new("application-grant-account-authorization")
        .text("account", &input.account.canonical_text())
        .u64("principal", input.principal.get())
        .u64("role", role_identity(input.role))
        .derive_identity()
        .bytes()
}

fn revoke_input_identity(input: &RevokeAccountAuthorization) -> [u8; 32] {
    *CanonicalProposalPayload::new("application-revoke-account-authorization")
        .text("account", &input.account.canonical_text())
        .text("authorization", &input.authorization.canonical_text())
        .derive_identity()
        .bytes()
}

worth_query_mutation_binding!(
    pub GrantAccountAccessMutationBinding for GrantAccountAuthorization, schema BankSchema,
    identity "bank.operation.grant-account-authorization.mutation-binding.v1",
    input GrantAccountAuthorizationInputBinding,
    operation GrantAccountAuthorizationOperation,
    result AccountAccessResultBinding,
    idempotency BankIdempotencyKey,
        identity "bank.application-mutation-client-key.v1",
        key_identity client_key_identity,
        input_identity grant_input_identity,
    decision BankInvariantApprovedProposal,
    denial AccountAccessDenialBinding,
    handler identity "bank.operation.grant-account-authorization.handler.v1",
    program required,
    outputs GrantAccountAccessOutputs,
    principal BankPrincipalBinding,
        mapping ExternalPrincipalMapping,
        principal_entity Principal,
        principal_identity BankPrincipalId,
        identity_binding BankPrincipalIdBinding,
    scope Account,
        Identity,
        AccountIdentity,
        AccountId,
        ReadOnly,
        NoApplicationUnit,
    field AccountIdentity::reference(),
    value grant_scope,
    candidates creates 1, deletes 0, links 2, unlinks 0, writes 2, emits 0,
    resources retained_representation_bytes 8192, validator_work 101
);

worth_query_mutation_binding!(
    pub RevokeAccountAccessMutationBinding for RevokeAccountAuthorization, schema BankSchema,
    identity "bank.operation.revoke-account-authorization.mutation-binding.v1",
    input RevokeAccountAuthorizationInputBinding,
    operation RevokeAccountAuthorizationOperation,
    result AccountAccessResultBinding,
    idempotency BankIdempotencyKey,
        identity "bank.application-mutation-client-key.v1",
        key_identity client_key_identity,
        input_identity revoke_input_identity,
    decision BankInvariantApprovedProposal,
    denial AccountAccessDenialBinding,
    handler identity "bank.operation.revoke-account-authorization.handler.v1",
    program required,
    outputs RevokeAccountAccessOutputs,
    principal BankPrincipalBinding,
        mapping ExternalPrincipalMapping,
        principal_entity Principal,
        principal_identity BankPrincipalId,
        identity_binding BankPrincipalIdBinding,
    scope Account,
        Identity,
        AccountIdentity,
        AccountId,
        ReadOnly,
        NoApplicationUnit,
    field AccountIdentity::reference(),
    value revoke_scope,
    candidates creates 0, deletes 1, links 0, unlinks 2, writes 0, emits 0,
    resources retained_representation_bytes 8192, validator_work 99
);
