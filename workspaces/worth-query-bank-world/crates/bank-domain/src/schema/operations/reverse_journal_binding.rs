use worth_query_decl::facade::{
    application_operation::NoApplicationMutationOutputs,
    application_schema::{NoApplicationUnit, ReadOnly},
    worth_query_mutation_binding, worth_query_structured_value_binding,
};

use crate::model::{BankPrincipalId, InstitutionId};
use crate::proposals::{
    BankIdempotencyKey, BankInvariantApprovedProposal, BankProposalDenial, CanonicalProposalPayload,
};

use super::create_personal_account_binding::client_key_identity;
use super::{ReversalReason, ReverseJournal, ReverseJournalInputBinding};
use crate::schema::{
    BankPrincipalBinding, BankPrincipalIdBinding, BankSchema, ExternalPrincipalMapping,
    Institution, InstitutionIdentity, InstitutionIdentityField, MoneyMovementResultBinding,
    Principal, ReverseJournalOperation,
};

worth_query_structured_value_binding!(
    pub ReverseJournalDenialBinding for BankProposalDenial {
        identity: "bank.operation.reverse-journal.denial.v1"
    }
);

fn reversal_scope(input: &ReverseJournal) -> InstitutionId {
    input.institution
}

fn reversal_identity(input: &ReverseJournal) -> [u8; 32] {
    let reason = match input.reason {
        ReversalReason::Duplicate => "duplicate",
        ReversalReason::OperatorCorrection => "operator-correction",
        ReversalReason::ExternalReturn => "external-return",
    };
    *CanonicalProposalPayload::new("application-reverse-journal")
        .u64("institution", input.institution.get())
        .text("journal", &input.journal.canonical_text())
        .text("reason", reason)
        .derive_identity()
        .bytes()
}

worth_query_mutation_binding!(
    pub ReverseJournalMutationBinding for ReverseJournal, schema BankSchema,
    identity "bank.operation.reverse-journal.mutation-binding.v1",
    input ReverseJournalInputBinding,
    operation ReverseJournalOperation,
    result MoneyMovementResultBinding,
    idempotency BankIdempotencyKey,
        identity "bank.application-mutation-client-key.v1",
        key_identity client_key_identity,
        input_identity reversal_identity,
    decision BankInvariantApprovedProposal,
    denial ReverseJournalDenialBinding,
    handler identity "bank.operation.reverse-journal.handler.v1",
    program required,
    outputs NoApplicationMutationOutputs,
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
    value reversal_scope,
    candidates creates 3, deletes 0, links 5, unlinks 0, writes 12, emits 2,
    resources retained_representation_bytes 32768, validator_work 16652
);
