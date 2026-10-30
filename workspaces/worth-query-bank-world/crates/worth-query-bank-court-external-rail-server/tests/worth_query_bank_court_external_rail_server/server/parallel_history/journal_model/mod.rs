pub(crate) use worth_query_bank_testkit_server::facade::parallel_history::journal_model::*;

#[test]
fn assessment_model_rejects_acceptance_before_settlement_and_preserves_retry_state() {
    use AssessmentRequest as R;
    let mut state = None;
    for (request, expected) in [
        (R::Start, Assessment::Started),
        (R::Propose, Assessment::Proposed),
        (R::Advance, Assessment::Awaiting),
        (R::Settle, Assessment::Settled),
        (R::RetrySettlement, Assessment::Settled),
        (R::Accept, Assessment::Accepted),
    ] {
        state = assessment_step(state, request);
        assert_eq!(state, Some(expected), "the hand-declared protocol");
    }
    assert_eq!(
        assessment_step(Some(Assessment::Awaiting), AssessmentRequest::Accept),
        None
    );
    assert_eq!(
        assessment_step(
            Some(Assessment::Settled),
            AssessmentRequest::RetrySettlement
        ),
        Some(Assessment::Settled)
    );
}

#[test]
fn journal_rules_conserve_cash_and_retry_without_another_posting() {
    let mut model = Journal::new(&[(11, 1), (12, 2)], 100, 11, 31);
    let transfer = Transfer {
        actor: 1,
        from: 11,
        to: 12,
        amount: 7,
        command: 1,
    };
    assert_eq!(model.apply(&transfer), Outcome::Committed(2));
    assert_eq!(model.apply(&transfer), Outcome::Committed(2));
    assert_eq!(
        model.apply(&Transfer {
            actor: 3,
            ..transfer.clone()
        }),
        Outcome::Unauthorized
    );
    assert_eq!(
        model.apply(&Transfer {
            amount: 25,
            command: 2,
            ..transfer
        }),
        Outcome::Insufficient
    );
    assert_eq!(model.postings.len(), 4);
    assert_eq!(model.balances[&11], 24);
    assert_eq!(model.balances[&12], 7);
    assert_eq!(
        model
            .postings
            .iter()
            .map(|posting| posting.amount)
            .sum::<i64>(),
        0
    );
}
