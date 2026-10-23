//! The journal created by an admitted SendMoney operation.
use worth_query_decl::facade::application_operation::{
    ApplicationMutationOutputContract, ApplicationMutationOutputRoleDescriptor,
    WorthQueryApplicationDeclaredOutputRole, WorthQueryApplicationOutputRole,
    WorthQueryCreateOutput, WorthQueryExactlyOneOutput,
};

use crate::schema::{BankSchema, JournalEntry};

pub struct SendMoneyOutputs;
pub struct SendMoneyJournal;

impl WorthQueryApplicationOutputRole for SendMoneyJournal {
    type Schema = BankSchema;
    type Contract = SendMoneyOutputs;
    type Entity = JournalEntry;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "sent-money-journal";
}

impl ApplicationMutationOutputContract<BankSchema> for SendMoneyOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[SendMoneyJournal::DESCRIPTOR];
}
