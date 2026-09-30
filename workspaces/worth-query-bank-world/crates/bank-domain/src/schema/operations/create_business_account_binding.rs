use worth_query_decl::facade::{
    application_operation::{
        ApplicationMutationOutputContract, ApplicationMutationOutputRoleDescriptor,
        WorthQueryApplicationOutputRole, WorthQueryCreateOutput,
    },
    application_schema::{NoApplicationUnit, ReadOnly},
    worth_query_mutation_binding, worth_query_structured_value_binding,
};

use crate::model::{AccountId, BankPrincipalId, InstitutionId};
use crate::proposals::{BankIdempotencyKey, BankInvariantApprovedProposal, BankProposalDenial};

use super::{CreateBusinessAccount, CreateBusinessAccountInputBinding};
use crate::schema::{
    Account, BankPrincipalBinding, BankPrincipalIdBinding, BankSchema,
    CreateBusinessAccountOperation, ExternalPrincipalMapping, Institution, InstitutionIdentity,
    InstitutionIdentityField, Principal,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CreateBusinessAccountResult {
    pub account: AccountId,
}

worth_query_structured_value_binding!(
    pub CreateBusinessAccountResultBinding for CreateBusinessAccountResult {
        identity: "bank.operation.create-business-account.result.v1"
    }
);

worth_query_structured_value_binding!(
    pub CreateBusinessAccountDenialBinding for BankProposalDenial {
        identity: "bank.operation.create-business-account.denial.v1"
    }
);

pub struct CreateBusinessAccountOutputs;

/// The business account the operation creates.
pub const CREATE_BUSINESS_ACCOUNT_OUTPUT_ACCOUNT: WorthQueryApplicationOutputRole<
    CreateBusinessAccountMutationBinding,
    Account,
    WorthQueryCreateOutput,
> = WorthQueryApplicationOutputRole::for_entity::<BankSchema>("account");

impl ApplicationMutationOutputContract<BankSchema> for CreateBusinessAccountOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[CREATE_BUSINESS_ACCOUNT_OUTPUT_ACCOUNT.descriptor()];
}

fn institution_scope(input: &CreateBusinessAccount) -> InstitutionId {
    input.institution
}

worth_query_mutation_binding!(
    pub CreateBusinessAccountMutationBinding for CreateBusinessAccount, schema BankSchema,
    identity "bank.operation.create-business-account.mutation-binding.v1",
    input CreateBusinessAccountInputBinding,
    operation CreateBusinessAccountOperation,
    result CreateBusinessAccountResultBinding,
    idempotency BankIdempotencyKey,
        identity "bank.application-mutation-client-key.v1",
    decision BankInvariantApprovedProposal,
    denial CreateBusinessAccountDenialBinding,
    handler identity "bank.operation.create-business-account.handler.v1",
    program required,
    outputs CreateBusinessAccountOutputs,
    principal BankPrincipalBinding,
        mapping ExternalPrincipalMapping,
        principal_entity Principal,
        principal_identity BankPrincipalId,
        identity_binding BankPrincipalIdBinding,
    scope Institution,
        InstitutionIdentity,
        InstitutionIdentityField,
        InstitutionId,
        ReadOnly,
        NoApplicationUnit,
    field InstitutionIdentityField::reference(),
    value institution_scope,
    candidates creates 1, deletes 0, links 2, unlinks 0, writes 5, emits 0,
    resources retained_representation_bytes 8192, validator_work 16
);
