use worth_query_decl::facade::{
    application_operation::{
        ApplicationMutationOutputContract, ApplicationMutationOutputPosture,
        ApplicationMutationOutputRoleDescriptor,
    },
    application_schema::{NoApplicationUnit, ReadOnly},
    worth_query_mutation_binding, worth_query_structured_value_binding,
};

use crate::model::{AccountId, BankPrincipalId, InstitutionId};
use crate::proposals::{
    BankIdempotencyKey, BankInvariantApprovedProposal, BankProposalDenial, CanonicalProposalPayload,
};

use super::create_personal_account_binding::client_key_identity;
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

pub const CREATE_BUSINESS_ACCOUNT_OUTPUT_ACCOUNT: &str = "account";

impl ApplicationMutationOutputContract<BankSchema> for CreateBusinessAccountOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[ApplicationMutationOutputRoleDescriptor::for_entity::<
            BankSchema,
            Account,
        >(
            CREATE_BUSINESS_ACCOUNT_OUTPUT_ACCOUNT,
            ApplicationMutationOutputPosture::Create,
        )];
}

fn institution_scope(input: &CreateBusinessAccount) -> InstitutionId {
    input.institution
}

fn input_identity(input: &CreateBusinessAccount) -> [u8; 32] {
    *CanonicalProposalPayload::new("application-create-business-account")
        .u64("institution", input.institution.get())
        .u64("business", input.business.get())
        .text("display-name", input.display_name.as_str())
        .derive_identity()
        .bytes()
}

worth_query_mutation_binding!(
    pub CreateBusinessAccountMutationBinding for CreateBusinessAccount, schema BankSchema,
    identity "bank.operation.create-business-account.mutation-binding.v1",
    input CreateBusinessAccountInputBinding,
    operation CreateBusinessAccountOperation,
    result CreateBusinessAccountResultBinding,
    idempotency BankIdempotencyKey,
        identity "bank.application-mutation-client-key.v1",
        key_identity client_key_identity,
        input_identity input_identity,
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
