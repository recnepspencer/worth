use super::*;
use worth_query_decl::facade::application_query::ApplicationQueryDefinitionDenial;

#[test]
fn continuation_ordering_must_be_a_direct_child_field() {
    let reversal = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        AccountActivityQuery,
        JournalEntry,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(JournalEntry::reference())
    .field(reversal_identity());
    let journal = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        AccountActivityQuery,
        JournalEntry,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(JournalEntry::reference())
    .field(journal_identity())
    .field(journal_purpose())
    .relation(journal_reversal(), reversal);
    let posting = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        AccountActivityQuery,
        Posting,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(Posting::reference())
    .field(posting_sequence())
    .field(posting_amount())
    .field(posting_purpose())
    .relation(posting_journal(), journal);
    let shape = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        AccountActivityQuery,
        Account,
        AccountActivityQueryResult,
        AccountActivityQueryResultBinding,
    >::new(Account::reference())
    .field(account_identity())
    .relation(account_postings(), posting)
    .build();
    let denied = ApplicationQueryDefinitionBuilder::declare(AccountActivityQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(3, 3, 8))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot().with_live())
        .requires_ability(ViewAccount::reference())
        .order_by(
            journal_purpose(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .continue_by(account_postings())
        .live_by::<Posting, AccountActivityLiveCause, _, _, _, _, _, _, _, _>(
            account_identity(),
            posting_identity(),
            ApplicationQueryLiveResourceContract::bounded(64, 2_048, 4_096),
        )
        .build()
        .unwrap_err();
    assert_eq!(
        denied,
        ApplicationQueryDefinitionDenial::ContinuationOrderingOutsideTarget
    );
}
