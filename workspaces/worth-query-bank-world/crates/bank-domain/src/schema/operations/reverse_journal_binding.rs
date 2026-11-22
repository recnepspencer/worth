use worth_query_decl::facade::{
    application_operation::NoApplicationMutationOutputs, worth_query_mutation_binding,
    worth_query_structured_value_binding,
};

use crate::model::InstitutionId;
use crate::proposals::{BankInvariantApprovedProposal, BankProposalDenial};

use super::{ReverseJournal, ReverseJournalInputBinding};
use crate::schema::{
    BankSchema, InstitutionIdentityField, MoneyMovementResultBinding, ReverseJournalOperation,
};

worth_query_structured_value_binding!(
    pub ReverseJournalDenialBinding for BankProposalDenial {
        identity: "bank.operation.reverse-journal.denial.v1"
    }
);

fn reversal_scope(input: &ReverseJournal) -> InstitutionId {
    input.institution
}

worth_query_mutation_binding!(
    pub ReverseJournalMutationBinding for ReverseJournal, schema BankSchema, context crate::schema::BankBindingContext,
    identity "bank.operation.reverse-journal.mutation-binding.v1",
    input ReverseJournalInputBinding,
    operation ReverseJournalOperation,
    result MoneyMovementResultBinding,
    decision BankInvariantApprovedProposal,
    denial ReverseJournalDenialBinding,
    handler identity "bank.operation.reverse-journal.handler.v1",
    program required,
    outputs NoApplicationMutationOutputs,
    scope InstitutionIdentityField,
    field InstitutionIdentityField::at("InstitutionIdentityField"),
    value reversal_scope,
);
