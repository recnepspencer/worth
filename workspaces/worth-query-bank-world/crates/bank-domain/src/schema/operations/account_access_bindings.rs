use worth_query_decl::facade::{
    application_operation::{
        ApplicationMutationOutputContract, ApplicationMutationOutputRoleDescriptor,
        WorthQueryApplicationDeclaredOutputRole, WorthQueryApplicationOutputRole,
        WorthQueryCreateOutput, WorthQueryExactlyOneOutput, WorthQueryRetireOutput,
    },
    application_schema::{NoApplicationUnit, ReadOnly},
    worth_query_mutation_binding, worth_query_structured_value_binding,
};

use crate::model::{AccountAuthorizationId, AccountId, BankPrincipalId};
use crate::proposals::{BankIdempotencyKey, BankInvariantApprovedProposal, BankProposalDenial};

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

/// The authorization a grant creates.
pub struct GrantedAccountAuthorizationOutput;

impl WorthQueryApplicationOutputRole for GrantedAccountAuthorizationOutput {
    type Schema = BankSchema;
    type Contract = GrantAccountAccessOutputs;
    type Entity = AccountAuthorization;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "authorization";
}

/// The authorization a revocation retires.
pub struct RevokedAccountAuthorizationOutput;

impl WorthQueryApplicationOutputRole for RevokedAccountAuthorizationOutput {
    type Schema = BankSchema;
    type Contract = RevokeAccountAccessOutputs;
    type Entity = AccountAuthorization;
    type Action = WorthQueryRetireOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "authorization";
}

impl ApplicationMutationOutputContract<BankSchema> for GrantAccountAccessOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        <GrantedAccountAuthorizationOutput as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
    ];
}

impl ApplicationMutationOutputContract<BankSchema> for RevokeAccountAccessOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        <RevokedAccountAuthorizationOutput as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
    ];
}

fn grant_scope(input: &GrantAccountAuthorization) -> AccountId {
    input.account
}

fn revoke_scope(input: &RevokeAccountAuthorization) -> AccountId {
    input.account
}

worth_query_mutation_binding!(
    pub GrantAccountAccessMutationBinding for GrantAccountAuthorization, schema BankSchema,
    identity "bank.operation.grant-account-authorization.mutation-binding.v1",
    input GrantAccountAuthorizationInputBinding,
    operation GrantAccountAuthorizationOperation,
    result AccountAccessResultBinding,
    idempotency BankIdempotencyKey,
        identity "bank.application-mutation-client-key.v1",
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
