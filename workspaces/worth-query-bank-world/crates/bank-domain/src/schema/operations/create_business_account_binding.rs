use worth_query_decl::facade::{
    application_operation::{
        ApplicationMutationOutputContract, ApplicationMutationOutputRoleDescriptor,
        WorthQueryApplicationDeclaredOutputRole, WorthQueryApplicationOutputRole,
        WorthQueryCreateOutput, WorthQueryExactlyOneOutput,
    },
    worth_query_mutation_binding, worth_query_structured_value_binding,
};

use crate::model::{AccountId, InstitutionId};
use crate::proposals::{BankInvariantApprovedProposal, BankProposalDenial};

use super::{CreateBusinessAccount, CreateBusinessAccountInputBinding};
use crate::schema::{
    Account, BankSchema, CreateBusinessAccountOperation, InstitutionIdentityField,
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
pub struct CreatedBusinessAccountOutput;

impl WorthQueryApplicationOutputRole for CreatedBusinessAccountOutput {
    type Schema = BankSchema;
    type Contract = CreateBusinessAccountOutputs;
    type Entity = Account;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "account";
}

impl ApplicationMutationOutputContract<BankSchema> for CreateBusinessAccountOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[<CreatedBusinessAccountOutput as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
}

fn institution_scope(input: &CreateBusinessAccount) -> InstitutionId {
    input.institution
}

worth_query_mutation_binding!(
    pub CreateBusinessAccountMutationBinding for CreateBusinessAccount, schema BankSchema, context crate::schema::BankBindingContext,
    identity "bank.operation.create-business-account.mutation-binding.v1",
    input CreateBusinessAccountInputBinding,
    operation CreateBusinessAccountOperation,
    result CreateBusinessAccountResultBinding,
    decision BankInvariantApprovedProposal,
    denial CreateBusinessAccountDenialBinding,
    handler identity "bank.operation.create-business-account.handler.v1",
    program required,
    outputs CreateBusinessAccountOutputs,
    scope InstitutionIdentityField,
    field InstitutionIdentityField::at("InstitutionIdentityField"),
    value institution_scope,
);
