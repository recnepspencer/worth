use worth_query_decl::facade::application_query::{
    ApplicationQueryResultShapeBuilder, TypedApplicationQueryResultShape,
};

use crate::reads::InstitutionAuditView;
use crate::schema::{Account, BankSchema, Institution, JournalEntry, Posting};

use super::fields::*;
use super::relations::*;
use super::InstitutionAuditQuery;

pub(super) fn institution_audit_shape() -> TypedApplicationQueryResultShape<
    BankSchema,
    InstitutionAuditQuery,
    Institution,
    InstitutionAuditView,
    super::InstitutionAuditQueryResultBinding,
> {
    let reversal = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        InstitutionAuditQuery,
        JournalEntry,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(JournalEntry::reference())
    .field(journal_identity::<ReversalIdentitySlot>("reversal_of"));
    let journal = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        InstitutionAuditQuery,
        JournalEntry,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(JournalEntry::reference())
    .field(journal_identity::<JournalIdentitySlot>("journal"))
    .field(journal_purpose())
    .relation(journal_reversal(), reversal);
    let posting = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        InstitutionAuditQuery,
        Posting,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(Posting::reference())
    .field(posting_identity())
    .field(posting_sequence())
    .field(posting_amount())
    .field(posting_purpose())
    .relation(posting_journal(), journal);
    let account = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        InstitutionAuditQuery,
        Account,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(Account::reference())
    .field(account_identity())
    .relation(account_postings(), posting);

    ApplicationQueryResultShapeBuilder::<
        BankSchema,
        InstitutionAuditQuery,
        Institution,
        InstitutionAuditView,
        super::InstitutionAuditQueryResultBinding,
    >::new(Institution::reference())
    .field(institution_identity())
    .relation(institution_accounts(), account)
    .build()
}
