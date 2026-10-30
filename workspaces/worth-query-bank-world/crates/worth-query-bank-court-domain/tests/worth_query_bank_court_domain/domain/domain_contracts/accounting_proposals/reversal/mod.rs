pub(crate) use worth_query_bank_testkit_domain::facade::domain_contracts::accounting_proposals::reversal::*;

#[test]
fn reversal_is_exact_and_cannot_be_applied_twice() {
    let opened = create_personal(&fixture(), 1, "Daily", "create-1");
    let source = opened.primary_account(id(BankPrincipalId::new, 1)).unwrap();
    let funded = BankProposalEngine::prepare_opening_funding(
        &opened,
        binding(2),
        &key("fund"),
        &ApplyOpeningFunding {
            institution: id(InstitutionId::new, 1),
            account: source,
            amount: Money::from_minor(5_000).unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone();
    let original = funded.journal()[0].clone();
    let reversed = BankProposalEngine::prepare_reverse_journal(
        &funded,
        binding(4),
        &key("reverse"),
        &ReverseJournal {
            institution: id(InstitutionId::new, 1),
            journal: original.id(),
            reason: ReversalReason::OperatorCorrection,
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone();
    let reversal = reversed.journal().last().unwrap();
    for (original, opposite) in original.postings().iter().zip(reversal.postings()) {
        assert_eq!(original.account(), opposite.account());
        assert_eq!(
            original.amount().minor_units(),
            -opposite.amount().minor_units()
        );
    }
    assert_eq!(oracle_balance(&reversed, source), 0);
    assert_eq!(
        BankProposalEngine::prepare_reverse_journal(
            &reversed,
            binding(4),
            &key("reverse-again"),
            &ReverseJournal {
                institution: id(InstitutionId::new, 1),
                journal: original.id(),
                reason: ReversalReason::Duplicate,
            },
        )
        .err(),
        Some(BankProposalDenial::JournalAlreadyReversed(original.id()))
    );
}
